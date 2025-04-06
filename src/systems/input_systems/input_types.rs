//! 入力イベント関連の型定義
//! 
//! ECSシステムで使用する入力イベントの型定義モジュール。

use std::collections::{VecDeque, HashSet, HashMap};
use serde::{Serialize, Deserialize};

/// マウスボタンの種類
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MouseButton {
    /// 左ボタン
    Left,
    /// 中ボタン
    Middle,
    /// 右ボタン
    Right,
}

/// 入力イベントの種類
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum InputEventType {
    /// マウス移動イベント (x, y)
    MouseMove(f64, f64),
    /// マウスクリックイベント (x, y, ボタン)
    MouseClick(f64, f64, MouseButton),
    /// マウスボタン押下イベント (x, y, ボタン)
    MouseDown(f64, f64, MouseButton),
    /// マウスボタン解放イベント (x, y, ボタン)
    MouseUp(f64, f64, MouseButton),
    /// キー押下イベント (キーコード)
    KeyPress(String),
    /// キー押下状態イベント (キーコード)
    KeyDown(String),
    /// キー解放イベント (キーコード)
    KeyUp(String),
    /// タッチイベント (x, y, タッチID)
    Touch(f64, f64, u32),
    /// ゲームパッドボタンイベント (ボタンID, ゲームパッドID)
    GamepadButton(u32, u32),
    /// ホイールイベント (deltaX, deltaY)
    Wheel(f64, f64),
}

/// 入力イベント
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InputEvent {
    /// イベントの種類
    pub event_type: InputEventType,
    /// イベント発生タイムスタンプ (ミリ秒)
    pub timestamp: f64,
    /// イベント処理済みフラグ
    pub handled: bool,
}

impl InputEvent {
    /// 新しい入力イベントを作成
    pub fn new(event_type: InputEventType, timestamp: f64) -> Self {
        Self {
            event_type,
            timestamp,
            handled: false,
        }
    }

    /// イベントを処理済みとしてマーク
    pub fn mark_handled(&mut self) {
        self.handled = true;
    }

    /// イベントが処理済みかどうか
    pub fn is_handled(&self) -> bool {
        self.handled
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_input_event_creation() {
        let event = InputEvent::new(InputEventType::MouseClick(100.0, 200.0, MouseButton::Left), 123.45);
        assert_eq!(event.timestamp, 123.45);
        assert!(!event.handled);
        
        match event.event_type {
            InputEventType::MouseClick(x, y, button) => {
                assert_eq!(x, 100.0);
                assert_eq!(y, 200.0);
                assert_eq!(button, MouseButton::Left);
            },
            _ => panic!("不正なイベントタイプ"),
        }
    }

    #[test]
    fn test_mark_handled() {
        let mut event = InputEvent::new(InputEventType::KeyPress("Enter".to_string()), 0.0);
        assert!(!event.is_handled());
        
        event.mark_handled();
        assert!(event.is_handled());
    }
} 