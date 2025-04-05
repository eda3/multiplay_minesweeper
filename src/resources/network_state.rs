/**
 * ネットワークリソース
 * 
 * WebSocket接続とネットワークメッセージを管理する
 */
use super::resource_trait::Resource;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{WebSocket, MessageEvent, CloseEvent, Event};
use js_sys::Function;
use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;
use serde_json::Value;
use std::any::Any;

/// ネットワーク状態を管理するリソース
#[derive(Debug, Clone)]
pub struct NetworkResource {
    /// WebSocketの接続状態
    pub is_connected: bool,
    /// サーバURL
    pub server_url: String,
    /// 受信メッセージのキュー
    pub message_queue: Vec<String>,
    /// 最終接続試行時間
    pub last_connect_attempt: f64,
    /// 接続試行回数
    pub connect_attempts: u32,
    /// 自動再接続するかどうか
    pub auto_reconnect: bool,
}

impl Default for NetworkResource {
    fn default() -> Self {
        Self {
            is_connected: false,
            server_url: "ws://localhost:8080".to_string(),
            message_queue: Vec::new(),
            last_connect_attempt: 0.0,
            connect_attempts: 0,
            auto_reconnect: true,
        }
    }
}

impl NetworkResource {
    /// 新しいネットワークリソースを作成
    pub fn new() -> Self {
        Self::default()
    }
    
    /// サーバURLを設定
    pub fn set_server_url(&mut self, url: &str) {
        self.server_url = url.to_string();
    }
    
    /// WebSocket接続を確立
    pub fn connect(&mut self) -> Result<WebSocket, JsValue> {
        // 現在時間を記録
        self.last_connect_attempt = js_sys::Date::now();
        self.connect_attempts += 1;
        
        // WebSocket接続を作成
        let ws = WebSocket::new(&self.server_url)?;
        
        // バイナリ型をArrayBufferに設定
        ws.set_binary_type(web_sys::BinaryType::Arraybuffer);
        
        self.is_connected = false;
        
        Ok(ws)
    }
    
    /// メッセージを送信（WebSocketインスタンスが必要）
    pub fn send_message(&self, ws: &WebSocket, message: &str) -> Result<(), JsValue> {
        if !self.is_connected {
            return Err(JsValue::from_str("Not connected to server"));
        }
        
        ws.send_with_str(message)
    }
    
    /// メッセージをキューに追加
    pub fn queue_message(&mut self, message: &str) {
        self.message_queue.push(message.to_string());
    }
    
    /// 接続状態を設定
    pub fn set_connected(&mut self, connected: bool) {
        self.is_connected = connected;
        
        if connected {
            // 接続に成功したらカウンタをリセット
            self.connect_attempts = 0;
        }
    }
    
    /// 再接続を試みるべきかどうかを判定
    pub fn should_reconnect(&self) -> bool {
        if !self.auto_reconnect || self.is_connected {
            return false;
        }
        
        // 最後の接続試行から5秒以上経過している、かつ試行回数が20未満
        let now = js_sys::Date::now();
        let elapsed = now - self.last_connect_attempt;
        
        elapsed > 5000.0 && self.connect_attempts < 20
    }
    
    /// WebSocketイベントハンドラを設定
    pub fn setup_event_handlers(&self, ws: &WebSocket, 
                               on_message: js_sys::Function, 
                               on_open: js_sys::Function, 
                               on_close: js_sys::Function, 
                               on_error: js_sys::Function) {
        let msg_fn = on_message.clone();
        let open_fn = on_open.clone();
        let close_fn = on_close.clone();
        let err_fn = on_error.clone();
        
        // メッセージ受信ハンドラ
        let onmessage_callback = Closure::wrap(
            Box::new(move |e: MessageEvent| {
                let _ = msg_fn.call1(&JsValue::NULL, &e);
            }) as Box<dyn FnMut(MessageEvent)>
        );
        ws.set_onmessage(Some(onmessage_callback.as_ref().unchecked_ref()));
        onmessage_callback.forget();
        
        // 接続成功ハンドラ
        let onopen_callback = Closure::wrap(
            Box::new(move |_| {
                let _ = open_fn.call0(&JsValue::NULL);
            }) as Box<dyn FnMut(JsValue)>
        );
        ws.set_onopen(Some(onopen_callback.as_ref().unchecked_ref()));
        onopen_callback.forget();
        
        // 接続終了ハンドラ
        let onclose_callback = Closure::wrap(
            Box::new(move |e: CloseEvent| {
                let _ = close_fn.call1(&JsValue::NULL, &e);
            }) as Box<dyn FnMut(CloseEvent)>
        );
        ws.set_onclose(Some(onclose_callback.as_ref().unchecked_ref()));
        onclose_callback.forget();
        
        // エラーハンドラ
        let connected = self.is_connected;
        let error_callback = Closure::wrap(
            Box::new(move |e: Event| {
                let _ = err_fn.call1(&JsValue::NULL, &e);
            }) as Box<dyn FnMut(Event)>
        );
        ws.set_onerror(Some(error_callback.as_ref().unchecked_ref()));
        error_callback.forget();
    }
} 