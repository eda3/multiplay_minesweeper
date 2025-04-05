/**
 * 入力リソース
 * 
 * キーボードとマウスの入力状態を管理する
 */
use std::collections::HashMap;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{Event, MouseEvent};

// どちらか一方だけを使用
// use crate::resources::Resource;
use super::resource_trait::Resource;
use super::mouse_state::MouseState;

/// 入力リソース
/// 
/// キーボードキーとマウスの状態を追跡
#[derive(Debug, Clone)]
pub struct InputResource {
    /// 現在押されているキー
    pub keys: HashMap<String, bool>,
    /// 前のフレームで押されていたキー
    pub previous_keys: HashMap<String, bool>,
    /// マウスの状態
    pub mouse: MouseState,
    /// 入力が有効かどうか
    pub enabled: bool,
}

impl Default for InputResource {
    fn default() -> Self {
        Self {
            keys: HashMap::new(),
            previous_keys: HashMap::new(),
            mouse: MouseState::default(),
            enabled: true,
        }
    }
}

impl InputResource {
    /// 新しい入力リソースを作成
    pub fn new() -> Self {
        Self::default()
    }
    
    /// フレーム更新
    pub fn update(&mut self) {
        // 前のフレームのキー状態を保存
        self.previous_keys = self.keys.clone();
    }
    
    /// キーが押されているかどうか
    pub fn is_key_down(&self, key: &str) -> bool {
        if !self.enabled {
            return false;
        }
        
        self.keys.get(key).copied().unwrap_or(false)
    }
    
    /// キーが押されたかどうか（このフレームで）
    pub fn is_key_pressed(&self, key: &str) -> bool {
        if !self.enabled {
            return false;
        }
        
        let current = self.keys.get(key).copied().unwrap_or(false);
        let previous = self.previous_keys.get(key).copied().unwrap_or(false);
        
        current && !previous
    }
    
    /// キーが離されたかどうか（このフレームで）
    pub fn is_key_released(&self, key: &str) -> bool {
        if !self.enabled {
            return false;
        }
        
        let current = self.keys.get(key).copied().unwrap_or(false);
        let previous = self.previous_keys.get(key).copied().unwrap_or(false);
        
        !current && previous
    }
    
    /// マウスが押されているかどうか
    pub fn is_mouse_down(&self, button: u16) -> bool {
        if !self.enabled {
            return false;
        }
        
        match button {
            0 => self.mouse.left_button,
            2 => self.mouse.right_button,
            1 => self.mouse.middle_button,
            _ => false,
        }
    }
    
    /// マウスが押されたかどうか（このフレームで）
    pub fn is_mouse_pressed(&self, button: u16) -> bool {
        if !self.enabled {
            return false;
        }
        
        match button {
            0 => self.mouse.clicked,
            2 => self.mouse.right_clicked,
            _ => false,
        }
    }
    
    /// マウス位置を取得
    pub fn get_mouse_position(&self) -> (i32, i32) {
        (self.mouse.x as i32, self.mouse.y as i32)
    }
    
    /// キーダウンイベントを処理
    pub fn handle_key_down(&mut self, event: &Event) {
        if !self.enabled {
            return;
        }
        
        // キーのコードを取得
        let code = js_sys::Reflect::get(event, &JsValue::from_str("code"))
            .unwrap_or(JsValue::from_str(""))
            .as_string()
            .unwrap_or_default();
        
        self.keys.insert(code, true);
    }
    
    /// キーアップイベントを処理
    pub fn handle_key_up(&mut self, event: &Event) {
        if !self.enabled {
            return;
        }
        
        // キーのコードを取得
        let code = js_sys::Reflect::get(event, &JsValue::from_str("code"))
            .unwrap_or(JsValue::from_str(""))
            .as_string()
            .unwrap_or_default();
        
        self.keys.insert(code, false);
    }
    
    /// マウス移動イベントを処理
    pub fn handle_mouse_move(&mut self, x: i32, y: i32) {
        if !self.enabled {
            return;
        }
        
        self.mouse.update_position(x as f64, y as f64);
    }
    
    /// マウスダウンイベントを処理
    pub fn handle_mouse_down(&mut self, button: u16) {
        if !self.enabled {
            return;
        }
        
        // ボタン状態を更新
        match button {
            0 => self.mouse.update_buttons(true, self.mouse.right_button, self.mouse.middle_button),
            2 => self.mouse.update_buttons(self.mouse.left_button, true, self.mouse.middle_button),
            1 => self.mouse.update_buttons(self.mouse.left_button, self.mouse.right_button, true),
            _ => {}
        }
    }
    
    /// マウスアップイベントを処理
    pub fn handle_mouse_up(&mut self, button: u16) {
        if !self.enabled {
            return;
        }
        
        // ボタン状態を更新
        match button {
            0 => self.mouse.update_buttons(false, self.mouse.right_button, self.mouse.middle_button),
            2 => self.mouse.update_buttons(self.mouse.left_button, false, self.mouse.middle_button),
            1 => self.mouse.update_buttons(self.mouse.left_button, self.mouse.right_button, false),
            _ => {}
        }
    }
    
    /// 入力を有効化
    pub fn enable(&mut self) {
        self.enabled = true;
    }
    
    /// 入力を無効化
    pub fn disable(&mut self) {
        self.enabled = false;
    }
    
    /// すべての入力状態をリセット
    pub fn reset(&mut self) {
        self.keys.clear();
        self.previous_keys.clear();
        self.mouse = MouseState::default();
    }
} 