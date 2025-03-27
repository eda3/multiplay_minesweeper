/**
 * ネットワーク状態リソース
 * 
 * WebSocket通信に関する状態を管理するリソース
 */
use wasm_bindgen::prelude::*;
use js_sys::{Function, Object, Reflect, JSON};
use web_sys::{WebSocket, MessageEvent, CloseEvent};
use std::collections::HashMap;

/// メッセージの種類
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub enum MessageType {
    /// 初期化メッセージ
    Init,
    /// プレイヤー参加
    PlayerJoined,
    /// プレイヤー退出
    PlayerLeft,
    /// プレイヤー移動
    PlayerMoved,
    /// セル開示
    CellsRevealed,
    /// ゲームオーバー
    GameOver,
    /// フラグ切り替え
    FlagToggled,
    /// ゲームリセット
    GameReset,
    /// エラー
    Error,
    /// 不明
    Unknown,
}

/// メッセージハンドラ
pub type MessageHandler = js_sys::Function;

/// ネットワーク状態リソース
#[derive(Debug)]
pub struct NetworkState {
    /// WebSocketインスタンス
    websocket: Option<WebSocket>,
    /// 接続状態
    pub is_connected: bool,
    /// ローカルプレイヤーID
    pub local_player_id: Option<String>,
    /// 最後のエラーメッセージ
    pub last_error: Option<String>,
    /// サーバーURL
    pub server_url: String,
    /// メッセージハンドラー
    message_handlers: HashMap<MessageType, Vec<MessageHandler>>,
    /// 送信キュー（未接続時に蓄積する）
    message_queue: Vec<String>,
    /// 再接続試行回数
    reconnect_attempts: u32,
    /// 最後に送信した位置更新時間
    pub last_position_update: f64,
}

impl Default for NetworkState {
    fn default() -> Self {
        Self {
            websocket: None,
            is_connected: false,
            local_player_id: None,
            last_error: None,
            server_url: "wss://minesweeper-server.example.com".to_string(),
            message_handlers: HashMap::new(),
            message_queue: Vec::new(),
            reconnect_attempts: 0,
            last_position_update: 0.0,
        }
    }
}

impl NetworkState {
    /// 新しいネットワーク状態を作成
    pub fn new(server_url: &str) -> Self {
        Self {
            server_url: server_url.to_string(),
            ..Default::default()
        }
    }
    
    /// WebSocketサーバーに接続
    pub fn connect(&mut self) -> Result<(), JsValue> {
        // すでに接続中なら何もしない
        if self.is_connected {
            return Ok(());
        }
        
        // WebSocketを作成
        let ws = WebSocket::new(&self.server_url)?;
        
        // バイナリ型を指定
        ws.set_binary_type(web_sys::BinaryType::Arraybuffer);
        
        // 接続時の処理
        let onopen_callback = Closure::wrap(Box::new(move || {
            web_sys::console::log_1(&"WebSocket接続しました！".into());
        }) as Box<dyn FnMut()>);
        ws.set_onopen(Some(onopen_callback.as_ref().unchecked_ref()));
        onopen_callback.forget();
        
        // メッセージ受信時の処理
        let this_clone = self as *mut NetworkState;
        let onmessage_callback = Closure::wrap(Box::new(move |e: MessageEvent| {
            let network_state = unsafe { &mut *this_clone };
            
            // テキストメッセージを処理
            if let Ok(txt) = e.data().dyn_into::<js_sys::JsString>() {
                let txt_str = String::from(txt);
                match network_state.handle_message(&txt_str) {
                    Ok(_) => {},
                    Err(err) => {
                        web_sys::console::error_1(&format!("メッセージ処理エラー: {:?}", err).into());
                        network_state.last_error = Some(format!("メッセージ処理エラー: {:?}", err));
                    }
                }
            }
        }) as Box<dyn FnMut(MessageEvent)>);
        ws.set_onmessage(Some(onmessage_callback.as_ref().unchecked_ref()));
        onmessage_callback.forget();
        
        // エラー発生時の処理
        let this_clone = self as *mut NetworkState;
        let onerror_callback = Closure::wrap(Box::new(move |_e: web_sys::Event| {
            let network_state = unsafe { &mut *this_clone };
            web_sys::console::error_1(&"WebSocketエラーが発生しました".into());
            network_state.is_connected = false;
            network_state.last_error = Some("WebSocketエラーが発生しました".to_string());
        }) as Box<dyn FnMut(web_sys::Event)>);
        ws.set_onerror(Some(onerror_callback.as_ref().unchecked_ref()));
        onerror_callback.forget();
        
        // 切断時の処理
        let this_clone = self as *mut NetworkState;
        let onclose_callback = Closure::wrap(Box::new(move |e: CloseEvent| {
            let network_state = unsafe { &mut *this_clone };
            web_sys::console::log_1(&format!("WebSocket切断: コード={}, 理由={}", e.code(), e.reason()).into());
            network_state.is_connected = false;
            network_state.websocket = None;
            
            // 自動再接続
            if network_state.reconnect_attempts < 3 {
                network_state.reconnect_attempts += 1;
                web_sys::console::log_1(&format!("再接続を試みます ({}/3)...", network_state.reconnect_attempts).into());
                
                // 1秒後に再接続
                let this_clone = network_state as *mut NetworkState;
                let reconnect_callback = Closure::once(Box::new(move || {
                    let network_state = unsafe { &mut *this_clone };
                    if let Err(err) = network_state.connect() {
                        web_sys::console::error_1(&format!("再接続エラー: {:?}", err).into());
                    }
                }) as Box<dyn FnOnce()>);
                
                web_sys::window()
                    .unwrap()
                    .set_timeout_with_callback_and_timeout_and_arguments_0(
                        reconnect_callback.as_ref().unchecked_ref(),
                        1000,
                    )
                    .unwrap();
                reconnect_callback.forget();
            }
        }) as Box<dyn FnMut(CloseEvent)>);
        ws.set_onclose(Some(onclose_callback.as_ref().unchecked_ref()));
        onclose_callback.forget();
        
        // WebSocketを保存
        self.websocket = Some(ws);
        self.is_connected = true;
        self.reconnect_attempts = 0;
        
        // 接続確立後、キューに溜まったメッセージを送信
        if !self.message_queue.is_empty() {
            let mut queue = std::mem::take(&mut self.message_queue);
            for msg in queue.drain(..) {
                self.send_raw(&msg)?;
            }
        }
        
        Ok(())
    }
    
    /// メッセージを送信
    pub fn send_message(&mut self, msg_type: &str, data: &JsValue) -> Result<(), JsValue> {
        // メッセージオブジェクトを作成
        let obj = Object::new();
        Reflect::set(&obj, &"type".into(), &msg_type.into())?;
        Reflect::set(&obj, &"data".into(), data)?;
        
        // JSON文字列に変換
        let json_str = JSON::stringify(&obj)?;
        
        // 送信
        self.send_raw(&String::from(json_str))
    }
    
    /// 生のメッセージを送信
    fn send_raw(&mut self, message: &str) -> Result<(), JsValue> {
        match &self.websocket {
            Some(ws) if self.is_connected => {
                ws.send_with_str(message)?;
                Ok(())
            },
            _ => {
                // 未接続の場合はキューに入れる
                self.message_queue.push(message.to_string());
                Ok(())
            }
        }
    }
    
    /// メッセージを処理
    fn handle_message(&mut self, message: &str) -> Result<(), JsValue> {
        // JSONをパース
        let json = JSON::parse(message)?;
        
        // メッセージタイプを取得
        let msg_type = if let Some(t) = Reflect::get(&json, &"type".into())
            .ok()
            .and_then(|v| v.as_string()) {
            match t.as_str() {
                "init" => MessageType::Init,
                "player_joined" => MessageType::PlayerJoined,
                "player_left" => MessageType::PlayerLeft,
                "player_moved" => MessageType::PlayerMoved,
                "cells_revealed" => MessageType::CellsRevealed,
                "game_over" => MessageType::GameOver,
                "flag_toggled" => MessageType::FlagToggled,
                "game_reset" => MessageType::GameReset,
                "error" => MessageType::Error,
                _ => MessageType::Unknown,
            }
        } else {
            MessageType::Unknown
        };
        
        // 登録されたハンドラを呼び出す
        if let Some(handlers) = self.message_handlers.get(&msg_type) {
            for handler in handlers {
                handler.call1(&JsValue::NULL, &json)?;
            }
        }
        
        // 初期化メッセージの場合はプレイヤーIDを保存
        if matches!(msg_type, MessageType::Init) {
            if let Some(player_id) = Reflect::get(&json, &"playerId".into())
                .ok()
                .and_then(|v| v.as_string()) {
                self.local_player_id = Some(player_id);
            }
        }
        
        Ok(())
    }
    
    /// メッセージハンドラを登録
    pub fn add_message_handler(&mut self, msg_type: MessageType, handler: MessageHandler) {
        self.message_handlers
            .entry(msg_type)
            .or_insert_with(Vec::new)
            .push(handler);
    }
    
    /// プレイヤーの位置を送信
    pub fn send_position_update(&mut self, x: f64, y: f64) -> Result<(), JsValue> {
        // 現在時刻を取得
        let now = js_sys::Date::now();
        
        // 前回の更新から一定時間（100ms）経過していれば送信
        if now - self.last_position_update > 100.0 {
            self.last_position_update = now;
            
            // 位置情報をオブジェクトにする
            let obj = Object::new();
            Reflect::set(&obj, &"x".into(), &x.into())?;
            Reflect::set(&obj, &"y".into(), &y.into())?;
            
            // 送信
            self.send_message("move", &obj)
        } else {
            Ok(())
        }
    }
    
    /// セルを開く操作を送信
    pub fn send_reveal_cell(&mut self, index: usize) -> Result<(), JsValue> {
        let obj = Object::new();
        Reflect::set(&obj, &"index".into(), &(index as u32).into())?;
        
        self.send_message("reveal", &obj)
    }
    
    /// フラグを切り替える操作を送信
    pub fn send_toggle_flag(&mut self, index: usize) -> Result<(), JsValue> {
        let obj = Object::new();
        Reflect::set(&obj, &"index".into(), &(index as u32).into())?;
        
        self.send_message("toggleFlag", &obj)
    }
    
    /// ゲームのリセットを送信
    pub fn send_reset_game(&mut self) -> Result<(), JsValue> {
        self.send_message("resetGame", &JsValue::NULL)
    }
    
    /// 接続を閉じる
    pub fn close(&mut self) -> Result<(), JsValue> {
        if let Some(ws) = &self.websocket {
            ws.close()?;
        }
        
        self.is_connected = false;
        self.websocket = None;
        
        Ok(())
    }
    
    /// 接続状態をチェック
    pub fn check_connection(&self) -> bool {
        self.is_connected
    }
    
    /// ローカルプレイヤーIDを設定
    pub fn set_local_player_id(&mut self, id: String) {
        self.local_player_id = Some(id);
    }
} 