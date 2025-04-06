/**
 * 入力処理システム
 * 
 * InputResourceからイベントを取り出して処理する
 */

use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use web_sys::console;

use crate::resources::input_resource::InputResource;
use crate::systems::input_systems::{InputEvent, InputEventType, MouseButton};
use crate::ecs::system::{System, SystemResult};
use crate::resources::ResourceManager;
use crate::entities::EntityManager;

// WASMで使用するためのマーカー属性
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn warn(s: &str);
}

/// 入力処理システム
#[derive(Default)]
pub struct InputProcessingSystem {
    /// 処理済みイベント数のカウンター（デバッグ/統計用）
    processed_events: usize,
    /// デバッグモードフラグ
    debug_mode: bool,
    /// システムが有効かどうか
    enabled: bool,
}

impl InputProcessingSystem {
    /// 新しい入力処理システムを作成
    pub fn new() -> Self {
        Self {
            processed_events: 0,
            debug_mode: false,
            enabled: true,
        }
    }
    
    /// デバッグモードを有効にする
    pub fn with_debug(mut self, debug: bool) -> Self {
        self.debug_mode = debug;
        self
    }
    
    /// 単一の入力イベントを処理
    fn process_input_event(&mut self, event: &mut InputEvent) -> Result<(), String> {
        // すでに処理済みのイベントはスキップ
        if event.is_handled() {
            return Ok(());
        }
        
        // 必要に応じてイベント処理をデバッグログに出力
        if self.debug_mode {
            self.log_event(event);
        }
        
        // ここで特定の入力イベントに対する処理を実装
        // 例: ゲーム状態の更新、UI要素の更新など
        match &event.event_type {
            InputEventType::KeyDown(key) => {
                // 特定のキー入力に対する処理を実装
                // 例: スペースキーでゲーム一時停止など
                if key == "Escape" {
                    // ESCキーの処理例
                    console::log_1(&"Escキーが押されました".into());
                }
            },
            InputEventType::MouseClick(x, y, button) => {
                // マウスクリックの処理
                // 例: 左クリックでセルを開く、右クリックでフラグを立てるなど
                match button {
                    MouseButton::Left => {
                        // 左クリック処理
                        if self.debug_mode {
                            console::log_1(&format!("左クリック: ({}, {})", x, y).into());
                        }
                    },
                    MouseButton::Right => {
                        // 右クリック処理
                        if self.debug_mode {
                            console::log_1(&format!("右クリック: ({}, {})", x, y).into());
                        }
                    },
                    _ => {}
                }
            },
            // 他のイベントタイプに対する処理を追加
            _ => {
                // デフォルト処理または無視
            }
        }
        
        // イベントを処理済みとしてマーク
        event.mark_handled();
        self.processed_events += 1;
        
        Ok(())
    }
    
    /// イベントをデバッグログに出力
    fn log_event(&self, event: &InputEvent) {
        match &event.event_type {
            InputEventType::KeyDown(key) => {
                console::log_1(&format!("キー押下: {}", key).into());
            },
            InputEventType::KeyUp(key) => {
                console::log_1(&format!("キー解放: {}", key).into());
            },
            InputEventType::MouseMove(x, y) => {
                // マウス移動イベントは多すぎるので、通常はログに出力しない
                // console::log_1(&format!("マウス移動: ({}, {})", x, y).into());
            },
            InputEventType::MouseClick(x, y, button) => {
                let button_str = match button {
                    MouseButton::Left => "左",
                    MouseButton::Right => "右",
                    MouseButton::Middle => "中",
                };
                console::log_1(&format!("マウスクリック: {} ({}, {})", button_str, x, y).into());
            },
            InputEventType::MouseDown(x, y, button) => {
                let button_str = match button {
                    MouseButton::Left => "左",
                    MouseButton::Right => "右",
                    MouseButton::Middle => "中",
                };
                console::log_1(&format!("マウス押下: {} ({}, {})", button_str, x, y).into());
            },
            InputEventType::MouseUp(x, y, button) => {
                let button_str = match button {
                    MouseButton::Left => "左",
                    MouseButton::Right => "右",
                    MouseButton::Middle => "中",
                };
                console::log_1(&format!("マウス解放: {} ({}, {})", button_str, x, y).into());
            },
            InputEventType::Wheel(dx, dy) => {
                console::log_1(&format!("ホイール: dx={}, dy={}", dx, dy).into());
            },
            InputEventType::Touch(x, y, id) => {
                console::log_1(&format!("タッチ: ID={} ({}, {})", id, x, y).into());
            },
            InputEventType::GamepadButton(button, controller) => {
                console::log_1(&format!("ゲームパッドボタン: コントローラ={}, ボタン={}", controller, button).into());
            },
            _ => {
                console::log_1(&format!("その他の入力イベント: {:?}", event.event_type).into());
            }
        }
    }
    
    /// 統計情報をリセット
    pub fn reset_stats(&mut self) {
        self.processed_events = 0;
    }
    
    /// 処理済みイベント数を取得
    pub fn get_processed_events(&self) -> usize {
        self.processed_events
    }
}

impl System for InputProcessingSystem {
    fn update(&mut self, _entity_manager: &mut EntityManager, resources: &mut ResourceManager) -> SystemResult {
        if !self.enabled {
            return SystemResult::Ok;
        }
        
        // InputResourceを探してイベントを処理
        if let Some(mut input) = resources.get_resource_mut::<InputResource>("") {
            // イベントキューからすべてのイベントを処理
            while let Some(mut event) = input.poll_event() {
                // イベントを処理
                if let Err(e) = self.process_input_event(&mut event) {
                    // エラーが発生した場合はログに出力するが、処理は続行
                    console::error_1(&format!("イベント処理エラー: {}", e).into());
                }
            }
            
            // 入力状態の更新（前フレームのキー状態の保存など）
            input.update();
            
            SystemResult::Ok
        } else {
            // InputResourceが見つからない場合
            console::warn_1(&"InputResourceが見つかりません".into());
            SystemResult::Error
        }
    }
    
    fn initialize(&mut self, _entity_manager: &mut EntityManager, _resources: &mut ResourceManager) -> SystemResult {
        // 初期化処理（特に必要なければ空でOK）
        self.reset_stats();
        SystemResult::Ok
    }
    
    fn cleanup(&mut self, _entity_manager: &mut EntityManager, _resources: &mut ResourceManager) -> SystemResult {
        // 終了処理（特に必要なければ空でOK）
        if self.debug_mode {
            console::log_1(&format!("InputProcessingSystem: 合計処理イベント数: {}", self.processed_events).into());
        }
        SystemResult::Ok
    }
    
    fn name(&self) -> &str {
        "InputProcessingSystem"
    }
    
    fn enabled(&self) -> bool {
        self.enabled
    }
    
    fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_system_creation() {
        let system = InputProcessingSystem::new();
        assert_eq!(system.name(), "InputProcessingSystem");
        assert_eq!(system.processed_events, 0);
        assert!(system.enabled());
    }
    
    #[test]
    fn test_with_debug() {
        let system = InputProcessingSystem::new().with_debug(true);
        assert!(system.debug_mode);
    }
} 