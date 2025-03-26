/**
 * ネットワークシステム
 * 
 * WebSocket通信を担当するシステム
 */
use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
use wasm_bindgen::JsValue;
use serde::{Serialize, Deserialize};

use crate::entities::EntityManager;
use crate::systems::system_registry::DeltaTime;
use crate::resources::{NetworkResource, PlayerResource, BoardResource};
use crate::components::{Position, Player};

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

/// ネットワークシステム - WebSocket通信を処理
pub fn network_system(
    entity_manager: &mut EntityManager,
    resources: &mut HashMap<&'static str, Rc<RefCell<dyn std::any::Any>>>,
    _delta_time: DeltaTime,
) -> Result<(), JsValue> {
    // ネットワークリソースを取得
    let network_resource = resources.get("network").and_then(|res| {
        res.clone().borrow_mut().downcast_mut::<NetworkResource>().map(|r| r.clone())
    });
    
    if let Some(network) = network_resource {
        // 新しいメッセージをチェック
        if !network.message_queue.is_empty() {
            // メッセージキューをローカルにコピー
            let messages = network.message_queue.clone();
            
            // リソース内のキューをクリア
            if let Some(network_rc) = resources.get("network") {
                if let Some(mut network_res) = network_rc.borrow_mut().downcast_mut::<NetworkResource>() {
                    network_res.message_queue.clear();
                }
            }
            
            // 各メッセージを処理
            for message_str in messages {
                process_message(entity_manager, resources, &message_str)?;
            }
        }
        
        // 定期的に位置情報を送信
        if let Some(player_resource) = resources.get("player").and_then(|res| {
            res.clone().borrow_mut().downcast_mut::<PlayerResource>().map(|r| r.clone())
        }) {
            // プレイヤーIDがある場合のみ送信
            if let Some(player_id) = &player_resource.player_id {
                // プレイヤーエンティティを検索
                let player_entities = entity_manager.find_entities_with_component::<Player>();
                
                for entity_id in player_entities {
                    // プレイヤーコンポーネントを取得して、現在のプレイヤーかチェック
                    if let Some(player) = entity_manager.get_component::<Player>(entity_id) {
                        if player.id == *player_id {
                            // 位置コンポーネントを取得
                            if let Some(position) = entity_manager.get_component::<Position>(entity_id) {
                                // 移動メッセージを作成
                                let move_msg = NetworkMessage {
                                    msg_type: NetworkMessageType::PlayerMove,
                                    player_id: Some(player_id.clone()),
                                    data: None,
                                    x: Some(position.x),
                                    y: Some(position.y),
                                };
                                
                                // JSONに変換
                                let move_json = serde_json::to_string(&move_msg).unwrap_or_default();
                                
                                // WebSocketで送信
                                if let Some(network_rc) = resources.get("network") {
                                    if let Some(mut network_res) = network_rc.borrow_mut().downcast_mut::<NetworkResource>() {
                                        network_res.send_message(&move_json);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    
    Ok(())
}

/// ネットワークメッセージを処理
fn process_message(
    entity_manager: &mut EntityManager,
    resources: &mut HashMap<&'static str, Rc<RefCell<dyn std::any::Any>>>,
    message_str: &str,
) -> Result<(), JsValue> {
    // メッセージをパース
    if let Ok(message) = serde_json::from_str::<NetworkMessage>(message_str) {
        match message.msg_type {
            NetworkMessageType::PlayerJoin => {
                // プレイヤー参加処理
                if let (Some(player_id), Some(data)) = (&message.player_id, &message.data) {
                    handle_player_join(entity_manager, player_id, data)?;
                }
            },
            
            NetworkMessageType::PlayerLeave => {
                // プレイヤー退出処理
                if let Some(player_id) = &message.player_id {
                    handle_player_leave(entity_manager, player_id)?;
                }
            },
            
            NetworkMessageType::PlayerMove => {
                // プレイヤー移動処理
                if let (Some(player_id), Some(x), Some(y)) = (&message.player_id, message.x, message.y) {
                    handle_player_move(entity_manager, player_id, x, y)?;
                }
            },
            
            NetworkMessageType::BoardUpdate => {
                // ボード更新処理
                if let Some(data) = &message.data {
                    handle_board_update(resources, data)?;
                }
            },
            
            NetworkMessageType::RevealCell => {
                // セル公開処理
                if let (Some(x), Some(y)) = (message.x, message.y) {
                    handle_reveal_cell(resources, x as usize, y as usize)?;
                }
            },
            
            NetworkMessageType::ToggleFlag => {
                // フラグ切替処理
                if let (Some(x), Some(y)) = (message.x, message.y) {
                    handle_toggle_flag(resources, x as usize, y as usize)?;
                }
            },
            
            NetworkMessageType::GameStart => {
                // ゲーム開始処理
                if let Some(data) = &message.data {
                    handle_game_start(resources, data)?;
                }
            },
            
            NetworkMessageType::GameOver => {
                // ゲームオーバー処理
                handle_game_over(resources)?;
            },
            
            NetworkMessageType::GameWin => {
                // ゲーム勝利処理
                handle_game_win(resources)?;
            },
            
            NetworkMessageType::Chat => {
                // チャットメッセージ処理
                if let (Some(player_id), Some(data)) = (&message.player_id, &message.data) {
                    handle_chat_message(entity_manager, player_id, data)?;
                }
            },
        }
    }
    
    Ok(())
}

/// プレイヤー参加処理
fn handle_player_join(
    entity_manager: &mut EntityManager,
    player_id: &str,
    player_name: &str,
) -> Result<(), JsValue> {
    // プレイヤーエンティティの作成
    let entity_id = entity_manager.create_entity();
    
    // プレイヤーコンポーネントの追加
    entity_manager.add_component(entity_id, Player {
        id: player_id.to_string(),
        name: player_name.to_string(),
        color: "#FF0000".to_string(),  // デフォルトカラー
    });
    
    // 位置コンポーネントの追加（初期位置）
    entity_manager.add_component(entity_id, Position {
        x: 100.0,
        y: 100.0,
    });
    
    Ok(())
}

/// プレイヤー退出処理
fn handle_player_leave(
    entity_manager: &mut EntityManager,
    player_id: &str,
) -> Result<(), JsValue> {
    // 指定されたIDのプレイヤーエンティティを検索
    let player_entities = entity_manager.find_entities_with_component::<Player>();
    
    for entity_id in player_entities {
        if let Some(player) = entity_manager.get_component::<Player>(entity_id) {
            if player.id == player_id {
                // エンティティを削除
                entity_manager.remove_entity(entity_id);
                break;
            }
        }
    }
    
    Ok(())
}

/// プレイヤー移動処理
fn handle_player_move(
    entity_manager: &mut EntityManager,
    player_id: &str,
    x: f64,
    y: f64,
) -> Result<(), JsValue> {
    // 指定されたIDのプレイヤーエンティティを検索
    let player_entities = entity_manager.find_entities_with_component::<Player>();
    
    for entity_id in player_entities {
        if let Some(player) = entity_manager.get_component::<Player>(entity_id) {
            if player.id == player_id {
                // 位置を更新
                if let Some(mut position) = entity_manager.get_component_mut::<Position>(entity_id) {
                    position.x = x;
                    position.y = y;
                }
                break;
            }
        }
    }
    
    Ok(())
}

/// ボード更新処理
fn handle_board_update(
    resources: &mut HashMap<&'static str, Rc<RefCell<dyn std::any::Any>>>,
    data: &str,
) -> Result<(), JsValue> {
    // ボードデータをパース
    if let Ok(board_data) = serde_json::from_str::<BoardResource>(data) {
        if let Some(board_rc) = resources.get("board") {
            if let Some(mut board) = board_rc.borrow_mut().downcast_mut::<BoardResource>() {
                // ボードデータを更新
                *board = board_data;
                board.is_updated = true;
            }
        }
    }
    
    Ok(())
}

/// セル公開処理
fn handle_reveal_cell(
    resources: &mut HashMap<&'static str, Rc<RefCell<dyn std::any::Any>>>,
    x: usize,
    y: usize,
) -> Result<(), JsValue> {
    if let Some(board_rc) = resources.get("board") {
        if let Some(mut board) = board_rc.borrow_mut().downcast_mut::<BoardResource>() {
            // セルを公開
            board.reveal_cell(x, y);
        }
    }
    
    Ok(())
}

/// フラグ切替処理
fn handle_toggle_flag(
    resources: &mut HashMap<&'static str, Rc<RefCell<dyn std::any::Any>>>,
    x: usize,
    y: usize,
) -> Result<(), JsValue> {
    if let Some(board_rc) = resources.get("board") {
        if let Some(mut board) = board_rc.borrow_mut().downcast_mut::<BoardResource>() {
            // フラグを切り替え
            board.toggle_flag(x, y);
        }
    }
    
    Ok(())
}

/// ゲーム開始処理
fn handle_game_start(
    resources: &mut HashMap<&'static str, Rc<RefCell<dyn std::any::Any>>>,
    data: &str,
) -> Result<(), JsValue> {
    // ゲーム初期データをパース
    if let Ok(board_data) = serde_json::from_str::<BoardResource>(data) {
        if let Some(board_rc) = resources.get("board") {
            if let Some(mut board) = board_rc.borrow_mut().downcast_mut::<BoardResource>() {
                // ボードデータをリセットして初期状態に
                *board = board_data;
                board.game_started = true;
                board.game_over = false;
                board.game_won = false;
                board.is_updated = true;
            }
        }
    }
    
    Ok(())
}

/// ゲームオーバー処理
fn handle_game_over(
    resources: &mut HashMap<&'static str, Rc<RefCell<dyn std::any::Any>>>,
) -> Result<(), JsValue> {
    if let Some(board_rc) = resources.get("board") {
        if let Some(mut board) = board_rc.borrow_mut().downcast_mut::<BoardResource>() {
            // ゲームオーバー状態に設定
            board.game_over = true;
            board.is_updated = true;
            
            // すべての地雷を公開
            for y in 0..board.height {
                for x in 0..board.width {
                    let idx = y * board.width + x;
                    if board.cells[idx] == -1 {
                        board.revealed[idx] = true;
                    }
                }
            }
        }
    }
    
    Ok(())
}

/// ゲーム勝利処理
fn handle_game_win(
    resources: &mut HashMap<&'static str, Rc<RefCell<dyn std::any::Any>>>,
) -> Result<(), JsValue> {
    if let Some(board_rc) = resources.get("board") {
        if let Some(mut board) = board_rc.borrow_mut().downcast_mut::<BoardResource>() {
            // 勝利状態に設定
            board.game_won = true;
            board.is_updated = true;
            
            // すべての地雷にフラグを立てる
            for y in 0..board.height {
                for x in 0..board.width {
                    let idx = y * board.width + x;
                    if board.cells[idx] == -1 {
                        board.flagged[idx] = true;
                    }
                }
            }
        }
    }
    
    Ok(())
}

/// チャットメッセージ処理
fn handle_chat_message(
    _entity_manager: &mut EntityManager,
    _player_id: &str,
    _message: &str,
) -> Result<(), JsValue> {
    // チャットメッセージを処理するロジックを実装
    // 必要に応じてUIリソースにメッセージを追加するなど
    
    Ok(())
} 