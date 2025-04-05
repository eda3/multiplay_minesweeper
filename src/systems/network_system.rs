/**
 * ネットワークシステム
 * 
 * ウェブソケット通信を処理するシステム
 */
use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
use wasm_bindgen::JsValue;
use serde::{Serialize, Deserialize};
use web_sys::CanvasRenderingContext2d;
use std::any::Any;
use wasm_bindgen::JsCast;
use web_sys::{WebSocket, MessageEvent, CloseEvent};
use web_sys::console;
use serde_json::Value;

use crate::entities::{EntityManager, Entity, EntityId};
use crate::systems::system_registry::DeltaTime;
use crate::resources::{
    NetworkResource, 
    PlayerStateResource, 
    BoardResource, 
    GameStateResource
};
use crate::components::{Position, player::Player};
use crate::resources::Resource;

/// ネットワークメッセージタイプ
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum NetworkMessageType {
    PlayerJoin,
    PlayerLeave,
    PlayerMove,
    BoardUpdate,
    RevealCell,
    ToggleFlag,
    GameStart,
    GameOver,
    GameWin,
    Chat,
}

/// ネットワークメッセージ
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NetworkMessage {
    pub msg_type: NetworkMessageType,
    pub player_id: Option<String>,
    pub data: Option<String>,
    pub x: Option<f64>,
    pub y: Option<f64>,
}

/// リソースに対するdowncast_mutメソッドを追加する拡張トレイト
trait ResourceExt {
    fn downcast_mut<T: 'static>(&mut self) -> Option<&mut T>;
}

impl ResourceExt for dyn Resource {
    fn downcast_mut<T: 'static>(&mut self) -> Option<&mut T> {
        // ResourceトレイトからAnyトレイトへの変換
        // as_any_mutメソッドを使用してAnyトレイトにキャスト
        self.as_any_mut().downcast_mut::<T>()
    }
}

// WebSocketインスタンスを保持する静的変数
static mut WEBSOCKET_INSTANCE: Option<WebSocket> = None;

/// ネットワークシステム関数
pub fn network_system(mut resources: &mut dyn Resource, _delta_time: f64) -> Result<(), JsValue> {
    // 各リソースから必要な情報を個別に抽出
    let needs_connection;
    let should_reconnect;
    let connect_attempts;
    let is_multiplayer;
    
    // 一時的なスコープで情報を取得
    {
        let network = resources.downcast_mut::<NetworkResource>().unwrap_or_else(|| {
            crate::js_bindings::log("NetworkResource not found");
            panic!("NetworkResource not found");
        });
        
        connect_attempts = network.connect_attempts;
        should_reconnect = network.should_reconnect();
        
        let is_connected = network.is_connected;
        
        // PlayerStateResourceから情報を取得 - 別のスコープで
        {
            let player = resources.downcast_mut::<PlayerStateResource>().unwrap_or_else(|| {
                crate::js_bindings::log("PlayerStateResource not found");
                panic!("PlayerStateResource not found");
            });
            
            is_multiplayer = player.is_multiplayer;
        }
        
        needs_connection = is_multiplayer && !is_connected;
    }
    
    // 接続が必要な場合
    if needs_connection {
        if connect_attempts == 0 {
            web_sys::console::log_1(&JsValue::from_str("接続を試みます"));
            
            // NetworkResourceの更新
            {
                let network = resources.downcast_mut::<NetworkResource>().unwrap_or_else(|| {
                    crate::js_bindings::log("NetworkResource not found");
                    panic!("NetworkResource not found");
                });
                
                network.connect_attempts += 1;
            }
            
            // WebSocket接続を試みる
            match setup_websocket() {
                Ok(ws) => {
                    // NetworkResourceを更新
                    let network = resources.downcast_mut::<NetworkResource>().unwrap_or_else(|| {
                        crate::js_bindings::log("NetworkResource not found");
                        panic!("NetworkResource not found");
                    });
                    
                    // 接続状態をtrueに設定
                    network.set_connected(true);
                    
                    // WebSocketインスタンスを保存
                    unsafe {
                        WEBSOCKET_INSTANCE = Some(ws);
                    }
                },
                Err(e) => {
                    web_sys::console::error_1(&JsValue::from_str(&format!("WebSocket接続エラー: {:?}", e)));
                    
                    let network = resources.downcast_mut::<NetworkResource>().unwrap_or_else(|| {
                        crate::js_bindings::log("NetworkResource not found");
                        panic!("NetworkResource not found");
                    });
                    
                    network.is_connected = false;
                },
            }
        } else if should_reconnect {
            web_sys::console::log_1(&JsValue::from_str("再接続を試みます"));
            
            // NetworkResourceの更新
            {
                let network = resources.downcast_mut::<NetworkResource>().unwrap_or_else(|| {
                    crate::js_bindings::log("NetworkResource not found");
                    panic!("NetworkResource not found");
                });
                
                network.connect_attempts = 0;
            }
            
            // WebSocket再接続
            match setup_websocket() {
                Ok(ws) => {
                    let network = resources.downcast_mut::<NetworkResource>().unwrap_or_else(|| {
                        crate::js_bindings::log("NetworkResource not found");
                        panic!("NetworkResource not found");
                    });
                    
                    // 接続状態をtrueに設定
                    network.set_connected(true);
                    
                    // WebSocketインスタンスを保存
                    unsafe {
                        WEBSOCKET_INSTANCE = Some(ws);
                    }
                },
                Err(e) => {
                    web_sys::console::error_1(&JsValue::from_str(&format!("WebSocket再接続エラー: {:?}", e)));
                },
            }
        }
    }
    
    // メッセージキュー処理は別途必要であれば実装
    
    Ok(())
}

/// WebSocketイベントハンドラの設定
fn setup_event_handlers(
    ws: web_sys::WebSocket,
    network: &mut NetworkResource,
    player: &mut PlayerStateResource,
    board: &mut BoardResource
) -> Result<(), Box<dyn std::error::Error>> {
    // 安全でないcode block - 静的参照を使用（実際の実装では避けるべき）
    unsafe {
        static mut NETWORK: Option<*mut NetworkResource> = None;
        static mut PLAYER: Option<*mut PlayerStateResource> = None;
        static mut BOARD: Option<*mut BoardResource> = None;
        
        NETWORK = Some(network as *mut NetworkResource);
        PLAYER = Some(player as *mut PlayerStateResource);
        BOARD = Some(board as *mut BoardResource);
        
        // メッセージ受信ハンドラ
        let on_message = js_sys::Function::new_with_args(
            "e",
            r#"
            if (window.wasmNetworkHandlers && window.wasmNetworkHandlers.onMessage) {
                window.wasmNetworkHandlers.onMessage(e);
            }
            "#,
        );
        
        // 接続成功ハンドラ
        let on_open = js_sys::Function::new_no_args(
            r#"
            if (window.wasmNetworkHandlers && window.wasmNetworkHandlers.onOpen) {
                window.wasmNetworkHandlers.onOpen();
            }
            "#,
        );
        
        // 接続終了ハンドラ
        let on_close = js_sys::Function::new_with_args(
            "e",
            r#"
            if (window.wasmNetworkHandlers && window.wasmNetworkHandlers.onClose) {
                window.wasmNetworkHandlers.onClose(e);
            }
            "#,
        );
        
        // エラーハンドラ
        let on_error = js_sys::Function::new_with_args(
            "e",
            r#"
            if (window.wasmNetworkHandlers && window.wasmNetworkHandlers.onError) {
                window.wasmNetworkHandlers.onError(e);
            }
            "#,
        );
        
        // ハンドラを設定
        network.setup_event_handlers(&ws, on_message, on_open, on_close, on_error);
        
        // JavaScript側にハンドラを設定
        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();
        
        // スクリプトを作成
        let script = document.create_element("script").map_err(|_| "Failed to create script element")?;
        script.set_attribute("type", "text/javascript").map_err(|_| "Failed to set attribute")?;
        script.set_text_content(Some(r#"
            window.wasmNetworkHandlers = {
                onMessage: function(e) {
                    if (e.data) {
                        try {
                            const data = JSON.parse(e.data);
                            window.wasmBindings.handleNetworkMessage(data);
                        } catch (err) {
                            console.error("Error parsing message:", err);
                        }
                    }
                },
                onOpen: function() {
                    window.wasmBindings.handleNetworkConnected();
                },
                onClose: function(e) {
                    window.wasmBindings.handleNetworkDisconnected(e.code, e.reason);
                },
                onError: function(e) {
                    window.wasmBindings.handleNetworkError(e.message || "Unknown error");
                }
            };
            
            // WASM Bindings
            window.wasmBindings = {
                handleNetworkMessage: function(data) {
                    // 実装はRust側から呼び出される
                },
                handleNetworkConnected: function() {
                    // 実装はRust側から呼び出される
                },
                handleNetworkDisconnected: function(code, reason) {
                    // 実装はRust側から呼び出される
                },
                handleNetworkError: function(message) {
                    // 実装はRust側から呼び出される
                }
            };
        "#));
        
        // headタグが存在すればそこに追加、なければbodyに追加
        let head_elements = document.get_element_by_id("head");
        if let Some(head) = head_elements {
            head.append_child(&script).map_err(|_| "Failed to append script to head")?;
        } else {
            // headが見つからない場合はbodyを探す
            let body_elements = document.get_element_by_id("body");
            if let Some(body) = body_elements {
                body.append_child(&script).map_err(|_| "Failed to append script to body")?;
            } else {
                // bodyも見つからない場合はdocumentに直接追加
                document.append_child(&script).map_err(|_| "Failed to append script to document")?;
            }
        }
        
        // ネットワークの接続状態を更新
        network.set_connected(true);
    }
    Ok(())
}

/// メッセージキューの処理
fn process_message_queue(
    network: &mut NetworkResource,
    player: &mut PlayerStateResource,
    board: &mut BoardResource
) {
    let messages = std::mem::take(&mut network.message_queue);
    
    for message in messages {
        // メッセージを処理
        process_message(network, player, board, &message);
    }
}

/// メッセージを処理
fn process_message(network: &NetworkResource, player: &mut PlayerStateResource, board: &mut BoardResource, message: &str) {
    let json: wasm_bindgen::JsValue = js_sys::JSON::parse(message).unwrap_or_else(|_| wasm_bindgen::JsValue::NULL);
    
    if json.is_null() {
        return; // 不正なJSONメッセージ
    }
    
    // Objectに変換
    let obj = js_sys::Object::from(json.clone());
    
    // メッセージタイプを取得
    let message_type: String = match js_sys::Reflect::get(&json, &"type".into()) {
        Ok(value) => value.as_string().unwrap_or_default(),
        Err(_) => return,
    };
    
    // メッセージタイプに応じて処理
    match message_type.as_str() {
        "init" => handle_init_message(&obj, player),
        "player_joined" => handle_player_joined(&obj, player),
        "player_left" => handle_player_left(&obj, player),
        "cells_revealed" => handle_cells_revealed(&obj, board),
        "flag_toggled" => handle_flag_toggled(&obj, board),
        "game_over" => handle_game_over(&obj, board),
        "game_reset" => handle_game_reset(&obj, board),
        _ => web_sys::console::warn_1(&format!("不明なメッセージタイプ: {}", message_type).into()),
    }
}

/// 初期化メッセージの処理
fn handle_init_message(data: &js_sys::Object, player: &mut PlayerStateResource) {
    if let Some(player_id) = js_sys::Reflect::get(&data, &"playerId".into()).ok().and_then(|v| v.as_string()) {
        player.set_player_id(player_id);
        player.has_joined = true;
    }
}

/// プレイヤー参加メッセージの処理
fn handle_player_joined(data: &js_sys::Object, player: &mut PlayerStateResource) {
    if let Some(player_name) = js_sys::Reflect::get(&data, &"name".into()).ok().and_then(|v| v.as_string()) {
        web_sys::console::log_1(&format!("Player joined: {}", player_name).into());
        // ここでプレイヤーリストなどを更新する
    }
}

/// プレイヤー退出メッセージの処理
fn handle_player_left(data: &js_sys::Object, player: &mut PlayerStateResource) {
    if let Some(player_id) = js_sys::Reflect::get(&data, &"playerId".into()).ok().and_then(|v| v.as_string()) {
        web_sys::console::log_1(&format!("Player left: {}", player_id).into());
        // ここでプレイヤーリストなどを更新する
    }
}

/// セル公開メッセージの処理
fn handle_cells_revealed(data: &js_sys::Object, board: &mut BoardResource) {
    if let Some(cells) = js_sys::Reflect::get(&data, &"cells".into()).ok() {
        if let Some(cells_array) = cells.dyn_into::<js_sys::Array>().ok() {
            let len = cells_array.length();
            for i in 0..len {
                if let Some(cell_index) = cells_array.get(i).as_f64() {
                    // セルを公開
                    board.reveal_cell(cell_index as usize);
                }
            }
        }
    }
}

/// フラグ切り替えメッセージの処理
fn handle_flag_toggled(data: &js_sys::Object, board: &mut BoardResource) {
    if let Some(index) = js_sys::Reflect::get(&data, &"index".into()).ok().and_then(|v| v.as_f64()) {
        // フラグを切り替え
        board.toggle_flag(index as usize);
    }
}

/// ゲームオーバーメッセージの処理
fn handle_game_over(data: &js_sys::Object, board: &mut BoardResource) {
    if let Some(win) = js_sys::Reflect::get(&data, &"win".into()).ok().and_then(|v| v.as_bool()) {
        if win {
            // 勝利処理
            web_sys::console::log_1(&"Game won!".into());
        } else {
            // 敗北処理
            web_sys::console::log_1(&"Game lost!".into());
            
            // すべての地雷を表示
            board.reveal_all_mines();
        }
    }
}

/// ゲームリセットメッセージの処理
fn handle_game_reset(data: &js_sys::Object, board: &mut BoardResource) {
    // ボード設定を取得
    if let Some(width) = js_sys::Reflect::get(&data, &"width".into()).ok().and_then(|v| v.as_f64()) {
        if let Some(height) = js_sys::Reflect::get(&data, &"height".into()).ok().and_then(|v| v.as_f64()) {
            if let Some(mines) = js_sys::Reflect::get(&data, &"mines".into()).ok().and_then(|v| v.as_f64()) {
                // ボードをリセット
                let config = crate::resources::board_state::BoardConfig::custom(
                    width as usize,
                    height as usize,
                    mines as usize,
                );
                *board = BoardResource::new(config);
            }
        }
    }
}

/// プレイヤー位置の更新
pub fn update_player_position(
    entity_manager: &mut EntityManager,
    player_id: &String,
    x: f64,
    y: f64,
) -> Result<(), String> {
    // プレイヤーのエンティティを探す
    let player_entities = entity_manager.find_entities_with_component::<Player>();
    let mut found = false;
    
    for entity_id in player_entities {
        if let Some(player) = entity_manager.get_component::<Player>(entity_id) {
            if player.id == *player_id {
                // 既存のプレイヤーを見つけた
                if let Some(pos) = entity_manager.get_component_mut::<Position>(entity_id) {
                    pos.x = x;
                    pos.y = y;
                    found = true;
                    break;
                }
            }
        }
    }
    
    // プレイヤーが見つからなかった場合、新規作成
    if !found {
        let entity_id = entity_manager.create_entity();
        entity_manager.add_component(entity_id, Position { x, y }).unwrap();
        entity_manager.add_component(entity_id, Player {
            id: player_id.clone(),
            name: format!("Player {}", player_id),
            color: "#FF0000".to_string(),
            last_action_time: js_sys::Date::now(),
            is_local: false,
        }).unwrap();
    }
    
    Ok(())
}

/// WebSocketの設定を行う
fn setup_websocket() -> Result<WebSocket, JsValue> {
    WebSocket::new("wss://api.eda3.net/ws")
} 