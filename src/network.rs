/**
 * WebSocket通信を管理するモジュール
 * 
 * サーバーとの通信機能を提供します。
 */
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{WebSocket, MessageEvent};
use std::collections::HashMap;
use serde_json::{json, Value};

use crate::js_bindings::{log, update_connection_status, update_player_count, get_websocket_url};
use crate::models::Player;

/**
 * WebSocket通信を管理する構造体
 */
pub struct NetworkManager {
    /// WebSocketインスタンス
    pub websocket: Option<WebSocket>,
    /// 接続状態
    pub is_connected: bool,
    /// ローカルプレイヤーID
    pub local_player_id: Option<String>,
}

/// CallbackType: GameStateのメソッドをコールバックとして使用するための型
pub type MessageCallback = Box<dyn Fn(&serde_json::Value) -> Result<(), JsValue>>;

impl NetworkManager {
    /**
     * 新しいNetworkManagerを作成
     */
    pub fn new() -> Self {
        Self {
            websocket: None,
            is_connected: false,
            local_player_id: None,
        }
    }
    
    /**
     * WebSocketサーバーに接続する
     * 
     * サーバーとの通信を確立し、各種イベントハンドラを設定します。
     * - onopen: 接続成功時の処理
     * - onmessage: メッセージ受信時の処理
     * - onerror: エラー発生時の処理
     * - onclose: 接続終了時の処理
     * 
     * @return 成功した場合はOk(()), エラーの場合はErr(JsValue)
     */
    pub fn connect(&mut self, message_callback: MessageCallback) -> Result<(), JsValue> {
        // WebSocketの作成
        let server_url = get_websocket_url();
        log(&format!("Connecting to WebSocket server at: {}", server_url));
        
        let ws = WebSocket::new(&server_url)?;
        let this = self as *mut NetworkManager;

        // onopen: 接続成功時のコールバック
        let onopen_callback = Closure::wrap(Box::new(move || {
            log("WebSocket connected!");
            update_connection_status(true);
            unsafe {
                (*this).is_connected = true;  // 接続状態を更新
            }
        }) as Box<dyn FnMut()>);
        ws.set_onopen(Some(onopen_callback.as_ref().unchecked_ref()));
        onopen_callback.forget();

        // onmessage: メッセージ受信時のコールバック
        let callback = message_callback;
        let onmessage_callback = Closure::wrap(Box::new(move |e: MessageEvent| {
            if let Ok(txt) = e.data().dyn_into::<js_sys::JsString>() {
                let message = String::from(txt);
                log(&format!("Message received: {}", message));
                
                // JSONをパース
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&message) {
                    // メッセージを処理するコールバックを呼び出す
                    if let Err(e) = callback(&json) {
                        log(&format!("Error processing message: {:?}", e));
                    }
                }
            }
        }) as Box<dyn FnMut(MessageEvent)>);
        ws.set_onmessage(Some(onmessage_callback.as_ref().unchecked_ref()));
        onmessage_callback.forget();

        // onerror: エラー発生時のコールバック
        let onerror_callback = Closure::wrap(Box::new(move |e: web_sys::Event| {
            log(&format!("WebSocket error: {:?}", e));
        }) as Box<dyn FnMut(web_sys::Event)>);
        ws.set_onerror(Some(onerror_callback.as_ref().unchecked_ref()));
        onerror_callback.forget();

        // onclose: 接続終了時のコールバック
        let network_manager = this;
        let onclose_callback = Closure::wrap(Box::new(move |e: web_sys::CloseEvent| {
            log(&format!("WebSocket closed: code={}, reason={}", e.code(), e.reason()));
            update_connection_status(false);
            unsafe {
                (*network_manager).is_connected = false;  // 接続状態を更新
            }
        }) as Box<dyn FnMut(web_sys::CloseEvent)>);
        ws.set_onclose(Some(onclose_callback.as_ref().unchecked_ref()));
        onclose_callback.forget();

        // WebSocketをフィールドに保存
        self.websocket = Some(ws);
        
        Ok(())
    }
    
    /**
     * メッセージを送信する
     * 
     * @param message 送信するJSONメッセージ
     * @return 成功した場合はOk(()), エラーの場合はErr(JsValue)
     */
    pub fn send_message(&self, message: &serde_json::Value) -> Result<(), JsValue> {
        if let Some(ws) = &self.websocket {
            if ws.ready_state() == web_sys::WebSocket::OPEN {
                let json_string = serde_json::to_string(message).unwrap();
                ws.send_with_str(&json_string)?;
                Ok(())
            } else {
                Err(JsValue::from_str("WebSocket is not open"))
            }
        } else {
            Err(JsValue::from_str("WebSocket is not initialized"))
        }
    }
    
    /**
     * プレイヤーの位置情報を送信する
     * 
     * @param x X座標
     * @param y Y座標
     * @return 成功した場合はOk(()), エラーの場合はErr(JsValue)
     */
    pub fn send_position_update(&self, x: f64, y: f64) -> Result<(), JsValue> {
        // 自分のIDがなければ送信しない
        if self.local_player_id.is_none() {
            return Ok(());
        }
        
        let message = json!({
            "type": "player_move",
            "x": x,
            "y": y
        });
        
        self.send_message(&message)
    }
    
    /**
     * セルを開く要求を送信する
     * 
     * @param index 開くセルのインデックス
     * @return 成功した場合はOk(()), エラーの場合はErr(JsValue)
     */
    pub fn send_reveal_cell(&self, index: usize) -> Result<(), JsValue> {
        let message = json!({
            "type": "reveal_cell",
            "index": index
        });
        
        self.send_message(&message)
    }
    
    /**
     * フラグをトグルする要求を送信する
     * 
     * @param index フラグを設定/解除するセルのインデックス
     * @return 成功した場合はOk(()), エラーの場合はErr(JsValue)
     */
    pub fn send_toggle_flag(&self, index: usize) -> Result<(), JsValue> {
        let message = json!({
            "type": "toggle_flag",
            "index": index
        });
        
        self.send_message(&message)
    }
    
    /**
     * ゲームをリセットする要求を送信する
     * 
     * @return 成功した場合はOk(()), エラーの場合はErr(JsValue)
     */
    pub fn send_reset_game(&self) -> Result<(), JsValue> {
        let message = json!({
            "type": "reset_game"
        });
        
        self.send_message(&message)
    }
    
    /**
     * ローカルプレイヤーIDを設定する
     * 
     * @param id プレイヤーID
     */
    pub fn set_local_player_id(&mut self, id: String) {
        self.local_player_id = Some(id);
    }
} 