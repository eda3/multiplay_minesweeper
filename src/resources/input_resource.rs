/**
 * 入力リソース
 * 
 * キーボードとマウスの入力状態を管理する
 */
use std::collections::{HashMap, HashSet, VecDeque};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{Event, MouseEvent, KeyboardEvent, WheelEvent, Touch, TouchEvent};

// どちらか一方だけを使用
// use crate::resources::Resource;
use super::resource_trait::Resource;
use super::mouse_state::MouseState;
use crate::systems::input_systems::{InputEvent, InputEventType, MouseButton};

/// 入力リソース
/// 
/// キーボードキーとマウスの状態を追跡し、入力イベントのキューを管理します。
#[derive(Debug, Clone)]
pub struct InputResource {
    /// 現在押されているキー
    pub keys: HashMap<String, bool>,
    /// 前のフレームで押されていたキー
    pub previous_keys: HashMap<String, bool>,
    /// マウスの状態
    pub mouse: MouseState,
    /// 入力イベントキュー
    pub event_queue: VecDeque<InputEvent>,
    /// 入力が有効かどうか
    pub enabled: bool,
    /// 入力感度設定（1.0がデフォルト、値が大きいほど感度が高い）
    pub sensitivity: f64,
    /// 押されているキーセット（キー名のセット、高速検索用）
    pub keys_down: HashSet<String>,
    /// アクティブなタッチIDのマップ（タッチIDから座標へのマップ）
    pub active_touches: HashMap<u32, (f64, f64)>,
}

impl Default for InputResource {
    fn default() -> Self {
        Self {
            keys: HashMap::new(),
            previous_keys: HashMap::new(),
            mouse: MouseState::default(),
            event_queue: VecDeque::with_capacity(32), // 適切な初期サイズを設定
            enabled: true,
            sensitivity: 1.0,
            keys_down: HashSet::new(),
            active_touches: HashMap::new(),
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
        
        // マウスのクリックフラグをリセット
        self.mouse.reset_click_flags();
    }
    
    /// イベントキューをクリア
    pub fn clear_event_queue(&mut self) {
        self.event_queue.clear();
    }
    
    /// イベントをキューに追加
    pub fn push_event(&mut self, event: InputEvent) {
        if !self.enabled {
            return;
        }
        
        self.event_queue.push_back(event);
    }
    
    /// イベントをキューから取り出す
    pub fn poll_event(&mut self) -> Option<InputEvent> {
        self.event_queue.pop_front()
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
    
    /// 型安全なマウスボタン列挙型を使用したマウスチェック
    pub fn is_mouse_button_down(&self, button: MouseButton) -> bool {
        if !self.enabled {
            return false;
        }
        
        match button {
            MouseButton::Left => self.mouse.left_button,
            MouseButton::Right => self.mouse.right_button,
            MouseButton::Middle => self.mouse.middle_button,
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
    
    /// 型安全なマウスボタン列挙型を使用したマウスプレスチェック
    pub fn is_mouse_button_pressed(&self, button: MouseButton) -> bool {
        if !self.enabled {
            return false;
        }
        
        match button {
            MouseButton::Left => self.mouse.clicked,
            MouseButton::Right => self.mouse.right_clicked,
            MouseButton::Middle => false, // 現在はサポートされていない
        }
    }
    
    /// マウス位置を取得
    pub fn get_mouse_position(&self) -> (i32, i32) {
        (self.mouse.x as i32, self.mouse.y as i32)
    }
    
    /// マウス位置を取得（f64）
    pub fn get_mouse_position_f64(&self) -> (f64, f64) {
        (self.mouse.x, self.mouse.y)
    }
    
    /// キーダウンイベントを処理
    pub fn handle_key_down(&mut self, event: &Event) {
        if !self.enabled {
            return;
        }
        
        // イベントをKeyboardEventにキャスト
        let keyboard_event = event.dyn_ref::<KeyboardEvent>();
        if let Some(key_event) = keyboard_event {
            let code = key_event.code();
            
            // キー状態を更新
            self.keys.insert(code.clone(), true);
            self.keys_down.insert(code.clone());
            
            // イベントキューにイベントを追加
            let timestamp = js_sys::Date::now();
            let input_event = InputEvent::new(InputEventType::KeyDown(code), timestamp);
            self.push_event(input_event);
        }
    }
    
    /// キーアップイベントを処理
    pub fn handle_key_up(&mut self, event: &Event) {
        if !self.enabled {
            return;
        }
        
        // イベントをKeyboardEventにキャスト
        let keyboard_event = event.dyn_ref::<KeyboardEvent>();
        if let Some(key_event) = keyboard_event {
            let code = key_event.code();
            
            // キー状態を更新
            self.keys.insert(code.clone(), false);
            self.keys_down.remove(&code);
            
            // イベントキューにイベントを追加
            let timestamp = js_sys::Date::now();
            let input_event = InputEvent::new(InputEventType::KeyUp(code), timestamp);
            self.push_event(input_event);
        }
    }
    
    /// マウス移動イベントを処理
    pub fn handle_mouse_move(&mut self, event: &Event) {
        if !self.enabled {
            return;
        }
        
        // イベントをMouseEventにキャスト
        let mouse_event = event.dyn_ref::<MouseEvent>();
        if let Some(mouse_event) = mouse_event {
            let x = mouse_event.client_x() as f64;
            let y = mouse_event.client_y() as f64;
            
            // マウス位置を更新
            self.mouse.update_position(x, y);
            
            // イベントキューにイベントを追加
            let timestamp = js_sys::Date::now();
            let input_event = InputEvent::new(InputEventType::MouseMove(x, y), timestamp);
            self.push_event(input_event);
        }
    }
    
    /// 従来のインターフェースとの互換性のためのメソッド
    pub fn handle_mouse_move_xy(&mut self, x: i32, y: i32) {
        if !self.enabled {
            return;
        }
        
        let x_f64 = x as f64;
        let y_f64 = y as f64;
        
        // マウス位置を更新
        self.mouse.update_position(x_f64, y_f64);
        
        // イベントキューにイベントを追加
        let timestamp = js_sys::Date::now();
        let input_event = InputEvent::new(InputEventType::MouseMove(x_f64, y_f64), timestamp);
        self.push_event(input_event);
    }
    
    /// マウスダウンイベントを処理
    pub fn handle_mouse_down(&mut self, event: &Event) {
        if !self.enabled {
            return;
        }
        
        // イベントをMouseEventにキャスト
        let mouse_event = event.dyn_ref::<MouseEvent>();
        if let Some(mouse_event) = mouse_event {
            // WASM環境では型の互換性のためにi16->u16へキャスト
            // buttonは0:左, 1:中, 2:右 の値を取るので、符号なし整数が適切
            let button = mouse_event.button() as u16;
            let x = mouse_event.client_x() as f64;
            let y = mouse_event.client_y() as f64;
            
            // マウスボタンを更新
            self.update_mouse_button_state(button, true);
            
            // 入力イベントを作成
            let timestamp = js_sys::Date::now();
            let mouse_button = match button {
                0 => MouseButton::Left,
                1 => MouseButton::Middle,
                2 => MouseButton::Right,
                _ => return, // 未サポートのボタン
            };
            
            let input_event = InputEvent::new(InputEventType::MouseDown(x, y, mouse_button), timestamp);
            self.push_event(input_event);
        }
    }
    
    /// 従来のインターフェースとの互換性のためのメソッド
    pub fn handle_mouse_down_button(&mut self, button: u16) {
        if !self.enabled {
            return;
        }
        
        // ボタン状態を更新
        self.update_mouse_button_state(button, true);
        
        // マウスイベントを作成
        let (x, y) = (self.mouse.x, self.mouse.y);
        let timestamp = js_sys::Date::now();
        
        let mouse_button = match button {
            0 => MouseButton::Left,
            1 => MouseButton::Middle,
            2 => MouseButton::Right,
            _ => return, // 未サポートのボタン
        };
        
        let input_event = InputEvent::new(InputEventType::MouseDown(x, y, mouse_button), timestamp);
        self.push_event(input_event);
    }
    
    /// マウスアップイベントを処理
    pub fn handle_mouse_up(&mut self, event: &Event) {
        if !self.enabled {
            return;
        }
        
        // イベントをMouseEventにキャスト
        let mouse_event = event.dyn_ref::<MouseEvent>();
        if let Some(mouse_event) = mouse_event {
            // WASM環境では型の互換性のためにi16->u16へキャスト
            // buttonは0:左, 1:中, 2:右 の値を取るので、符号なし整数が適切
            let button = mouse_event.button() as u16;
            let x = mouse_event.client_x() as f64;
            let y = mouse_event.client_y() as f64;
            
            // マウスボタンを更新
            self.update_mouse_button_state(button, false);
            
            // 入力イベントを作成
            let timestamp = js_sys::Date::now();
            let mouse_button = match button {
                0 => MouseButton::Left,
                1 => MouseButton::Middle,
                2 => MouseButton::Right,
                _ => return, // 未サポートのボタン
            };
            
            let input_event = InputEvent::new(InputEventType::MouseUp(x, y, mouse_button), timestamp);
            self.push_event(input_event);
            
            // クリックイベントも発行（短いダウン->アップシーケンス）
            if self.is_click_gesture(x, y) {
                let click_event = InputEvent::new(InputEventType::MouseClick(x, y, mouse_button), timestamp);
                self.push_event(click_event);
            }
        }
    }
    
    /// 従来のインターフェースとの互換性のためのメソッド
    pub fn handle_mouse_up_button(&mut self, button: u16) {
        if !self.enabled {
            return;
        }
        
        // ボタン状態を更新
        self.update_mouse_button_state(button, false);
        
        // マウスイベントを作成
        let (x, y) = (self.mouse.x, self.mouse.y);
        let timestamp = js_sys::Date::now();
        
        let mouse_button = match button {
            0 => MouseButton::Left,
            1 => MouseButton::Middle,
            2 => MouseButton::Right,
            _ => return, // 未サポートのボタン
        };
        
        let input_event = InputEvent::new(InputEventType::MouseUp(x, y, mouse_button), timestamp);
        self.push_event(input_event);
    }
    
    /// マウスホイールイベントを処理
    pub fn handle_wheel(&mut self, event: &Event) {
        if !self.enabled {
            return;
        }
        
        // イベントをWheelEventにキャスト
        let wheel_event = event.dyn_ref::<WheelEvent>();
        if let Some(wheel_event) = wheel_event {
            let delta_x = wheel_event.delta_x();
            let delta_y = wheel_event.delta_y();
            
            // 感度を適用
            let scaled_delta_x = delta_x * self.sensitivity;
            let scaled_delta_y = delta_y * self.sensitivity;
            
            // イベントキューにイベントを追加
            let timestamp = js_sys::Date::now();
            let input_event = InputEvent::new(InputEventType::Wheel(scaled_delta_x, scaled_delta_y), timestamp);
            self.push_event(input_event);
        }
    }
    
    /// タッチスタートイベントを処理
    pub fn handle_touch_start(&mut self, event: &Event) {
        if !self.enabled {
            return;
        }
        
        // イベントをTouchEventにキャスト
        let touch_event = event.dyn_ref::<TouchEvent>();
        if let Some(touch_event) = touch_event {
            let touches = touch_event.changed_touches();
            for i in 0..touches.length() {
                if let Some(touch) = touches.get(i) {
                    // WASM環境では型の互換性のためにi32->u32へキャスト
                    // タッチIDは常に非負整数なので、符号なし整数が適切
                    let id = touch.identifier() as u32;
                    let x = touch.client_x() as f64;
                    let y = touch.client_y() as f64;
                    
                    // タッチ情報を保存
                    self.active_touches.insert(id, (x, y));
                    
                    // イベントキューにイベントを追加
                    let timestamp = js_sys::Date::now();
                    let input_event = InputEvent::new(InputEventType::Touch(x, y, id), timestamp);
                    self.push_event(input_event);
                }
            }
        }
    }
    
    /// タッチ移動イベントを処理
    pub fn handle_touch_move(&mut self, event: &Event) {
        if !self.enabled {
            return;
        }
        
        // イベントをTouchEventにキャスト
        let touch_event = event.dyn_ref::<TouchEvent>();
        if let Some(touch_event) = touch_event {
            let touches = touch_event.changed_touches();
            for i in 0..touches.length() {
                if let Some(touch) = touches.get(i) {
                    // WASM環境では型の互換性のためにi32->u32へキャスト
                    // タッチIDは常に非負整数なので、符号なし整数が適切
                    let id = touch.identifier() as u32;
                    let x = touch.client_x() as f64;
                    let y = touch.client_y() as f64;
                    
                    // タッチ情報を更新
                    self.active_touches.insert(id, (x, y));
                    
                    // イベントキューにイベントを追加
                    let timestamp = js_sys::Date::now();
                    let input_event = InputEvent::new(InputEventType::Touch(x, y, id), timestamp);
                    self.push_event(input_event);
                }
            }
        }
    }
    
    /// タッチエンドイベントを処理
    pub fn handle_touch_end(&mut self, event: &Event) {
        if !self.enabled {
            return;
        }
        
        // イベントをTouchEventにキャスト
        let touch_event = event.dyn_ref::<TouchEvent>();
        if let Some(touch_event) = touch_event {
            let touches = touch_event.changed_touches();
            for i in 0..touches.length() {
                if let Some(touch) = touches.get(i) {
                    // WASM環境では型の互換性のためにi32->u32へキャスト
                    // タッチIDは常に非負整数なので、符号なし整数が適切
                    let id = touch.identifier() as u32;
                    
                    // タッチ情報を削除
                    self.active_touches.remove(&id);
                }
            }
        }
    }
    
    /// 型安全なマウスボタン状態の更新ロジック
    fn update_mouse_button_state(&mut self, button: u16, state: bool) {
        match button {
            0 => self.mouse.update_buttons(state, self.mouse.right_button, self.mouse.middle_button),
            2 => self.mouse.update_buttons(self.mouse.left_button, state, self.mouse.middle_button),
            1 => self.mouse.update_buttons(self.mouse.left_button, self.mouse.right_button, state),
            _ => {}
        }
    }
    
    /// クリックジェスチャかどうかを判定
    fn is_click_gesture(&self, x: f64, y: f64) -> bool {
        // クリックと見なす最大距離（ピクセル単位）
        const MAX_CLICK_DISTANCE: f64 = 10.0;
        
        // マウスの動きが小さい場合はクリックとして扱う
        let dx = x - self.mouse.prev_x;
        let dy = y - self.mouse.prev_y;
        let distance_squared = dx * dx + dy * dy;
        
        distance_squared <= MAX_CLICK_DISTANCE * MAX_CLICK_DISTANCE
    }
    
    /// 入力を有効化
    pub fn enable(&mut self) {
        self.enabled = true;
    }
    
    /// 入力を無効化
    pub fn disable(&mut self) {
        self.enabled = false;
    }
    
    /// 入力感度を設定
    pub fn set_sensitivity(&mut self, sensitivity: f64) {
        self.sensitivity = sensitivity.max(0.1).min(5.0); // 感度を範囲内に制限
    }
    
    /// すべての入力状態をリセット
    pub fn reset(&mut self) {
        self.keys.clear();
        self.previous_keys.clear();
        self.mouse = MouseState::default();
        self.event_queue.clear();
        self.keys_down.clear();
        self.active_touches.clear();
    }
} 