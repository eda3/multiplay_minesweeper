/**
 * マウス状態リソース
 * 
 * マウスの現在位置とボタンの状態を管理する
 */
use super::resource_trait::Resource;
use std::any::Any;

/// マウスの状態を表すリソース
#[derive(Debug, Clone)]
pub struct MouseState {
    /// X座標
    pub x: f64,
    /// Y座標
    pub y: f64,
    /// 左ボタンが押されているか
    pub left_button: bool,
    /// 右ボタンが押されているか
    pub right_button: bool,
    /// 中央ボタンが押されているか
    pub middle_button: bool,
    /// 前回のX座標
    pub prev_x: f64,
    /// 前回のY座標
    pub prev_y: f64,
    /// クリックされたか（一時的なフラグ、1フレームのみtrue）
    pub clicked: bool,
    /// 右クリックされたか（一時的なフラグ、1フレームのみtrue）
    pub right_clicked: bool,
}

impl Default for MouseState {
    fn default() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            left_button: false,
            right_button: false,
            middle_button: false,
            prev_x: 0.0,
            prev_y: 0.0,
            clicked: false,
            right_clicked: false,
        }
    }
}

impl MouseState {
    /// 新しいマウス状態を作成
    pub fn new() -> Self {
        Self::default()
    }
    
    /// マウス位置を更新
    pub fn update_position(&mut self, x: f64, y: f64) {
        self.prev_x = self.x;
        self.prev_y = self.y;
        self.x = x;
        self.y = y;
    }
    
    /// マウスボタン状態を更新
    pub fn update_buttons(&mut self, left: bool, right: bool, middle: bool) {
        // クリックイベントの検出（押されてない状態から押された状態への変化）
        self.clicked = !self.left_button && left;
        self.right_clicked = !self.right_button && right;
        
        // ボタン状態の更新
        self.left_button = left;
        self.right_button = right;
        self.middle_button = middle;
    }
    
    /// マウスの移動量を取得
    pub fn get_movement(&self) -> (f64, f64) {
        (self.x - self.prev_x, self.y - self.prev_y)
    }
    
    /// クリックフラグをリセット（フレーム終了時に呼ぶ）
    pub fn reset_click_flags(&mut self) {
        self.clicked = false;
        self.right_clicked = false;
    }

    // 互換性のための定数
    pub const LeftDown: &'static str = "LeftDown";
    pub const RightDown: &'static str = "RightDown";
    pub const MiddleDown: &'static str = "MiddleDown";
    pub const Up: &'static str = "Up";
    
    /// マウスの状態文字列を取得（互換性のため）
    pub fn get_state(&self) -> &'static str {
        if self.left_button {
            Self::LeftDown
        } else if self.right_button {
            Self::RightDown
        } else if self.middle_button {
            Self::MiddleDown
        } else {
            Self::Up
        }
    }
} 