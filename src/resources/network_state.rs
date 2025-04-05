/**
 * ネットワーク状態リソース
 * 
 * マルチプレイヤー機能のためのネットワーク状態を管理するリソース
 */

use wasm_bindgen::JsValue;
use wasm_bindgen::JsCast;
use web_sys::{WebSocket, MessageEvent};
use std::collections::VecDeque;

/// ネットワークメッセージ型
#[derive(Debug, Clone)]
pub enum NetworkMessage {
    Connect(String),      // プレイヤーID
    Disconnect(String),   // プレイヤーID
    Position(String, f64, f64), // プレイヤーID, x, y
    RevealCell(String, usize, usize), // プレイヤーID, row, col
    FlagCell(String, usize, usize),   // プレイヤーID, row, col
    ResetGame(String),    // プレイヤーID
    ChatMessage(String, String), // プレイヤーID, メッセージ
    Error(String),        // エラーメッセージ
    Raw(String),          // 生のメッセージ
}

/// ネットワーク状態リソース
#[derive(Debug)]
pub struct NetworkResource {
    /// WebSocketインスタンス
    pub socket: Option<WebSocket>,
    /// 接続状態
    pub connected: bool,
    /// セッションID
    pub session_id: Option<String>,
    /// プレイヤーID
    pub player_id: Option<String>,
    /// 受信メッセージキュー
    pub message_queue: VecDeque<NetworkMessage>,
    /// 最後のエラー
    pub last_error: Option<String>,
}

impl Default for NetworkResource {
    fn default() -> Self {
        Self {
            socket: None,
            connected: false,
            session_id: None,
            player_id: None,
            message_queue: VecDeque::new(),
            last_error: None,
        }
    }
}

impl NetworkResource {
    /// 新しいネットワークリソースを作成
    pub fn new() -> Self {
        Self::default()
    }
    
    /// WebSocketを接続
    pub fn connect(&mut self, url: &str) -> Result<(), JsValue> {
        // 既存の接続を閉じる
        if let Some(socket) = &self.socket {
            socket.close()?;
        }
        
        // 新しいWebSocketを作成
        let socket = WebSocket::new(url)?;
        
        // イベントリスナーを設定する必要があるが、
        // Rust側からはクロージャを渡す必要があるため、
        // 実際の実装ではJS側でイベントをハンドリングすることが多い
        
        self.socket = Some(socket);
        self.connected = false; // open イベントが発生するまで false
        
        Ok(())
    }
    
    /// 接続を閉じる
    pub fn disconnect(&mut self) -> Result<(), JsValue> {
        if let Some(socket) = &self.socket {
            socket.close()?;
        }
        
        self.socket = None;
        self.connected = false;
        self.session_id = None;
        
        Ok(())
    }
    
    /// メッセージを送信
    pub fn send_message(&self, message: &str) -> Result<(), JsValue> {
        if let Some(socket) = &self.socket {
            if self.connected {
                socket.send_with_str(message)?;
                return Ok(());
            }
        }
        
        Err(JsValue::from_str("WebSocket not connected"))
    }
    
    /// プレイヤー位置を送信
    pub fn send_position(&self, x: f64, y: f64) -> Result<(), JsValue> {
        if let Some(player_id) = &self.player_id {
            let message = format!("{{\"type\":\"position\",\"id\":\"{}\",\"x\":{},\"y\":{}}}", player_id, x, y);
            self.send_message(&message)
        } else {
            Err(JsValue::from_str("Player ID not set"))
        }
    }
    
    /// セル公開リクエストを送信
    pub fn send_reveal_cell(&self, row: usize, col: usize) -> Result<(), JsValue> {
        if let Some(player_id) = &self.player_id {
            let message = format!("{{\"type\":\"reveal\",\"id\":\"{}\",\"row\":{},\"col\":{}}}", player_id, row, col);
            self.send_message(&message)
        } else {
            Err(JsValue::from_str("Player ID not set"))
        }
    }
    
    /// フラグトグルリクエストを送信
    pub fn send_flag_cell(&self, row: usize, col: usize) -> Result<(), JsValue> {
        if let Some(player_id) = &self.player_id {
            let message = format!("{{\"type\":\"flag\",\"id\":\"{}\",\"row\":{},\"col\":{}}}", player_id, row, col);
            self.send_message(&message)
        } else {
            Err(JsValue::from_str("Player ID not set"))
        }
    }
    
    /// リセットリクエストを送信
    pub fn send_reset_game(&self) -> Result<(), JsValue> {
        if let Some(player_id) = &self.player_id {
            let message = format!("{{\"type\":\"reset\",\"id\":\"{}\"}}", player_id);
            self.send_message(&message)
        } else {
            Err(JsValue::from_str("Player ID not set"))
        }
    }
    
    /// チャットメッセージを送信
    pub fn send_chat_message(&self, text: &str) -> Result<(), JsValue> {
        if let Some(player_id) = &self.player_id {
            let message = format!("{{\"type\":\"chat\",\"id\":\"{}\",\"text\":\"{}\"}}", player_id, text);
            self.send_message(&message)
        } else {
            Err(JsValue::from_str("Player ID not set"))
        }
    }
    
    /// メッセージを受信（JS側から呼び出される）
    pub fn receive_message(&mut self, event: MessageEvent) -> Result<(), JsValue> {
        let data = event.data();
        
        // テキストメッセージの場合
        if let Ok(text) = data.dyn_into::<js_sys::JsString>() {
            let text = String::from(text);
            
            // 一旦生メッセージとしてキューに追加
            self.message_queue.push_back(NetworkMessage::Raw(text));
        }
        
        Ok(())
    }
    
    /// メッセージキューからメッセージを取得
    pub fn poll_message(&mut self) -> Option<NetworkMessage> {
        self.message_queue.pop_front()
    }
    
    /// エラーを設定
    pub fn set_error(&mut self, error: String) {
        self.last_error = Some(error);
    }
} 