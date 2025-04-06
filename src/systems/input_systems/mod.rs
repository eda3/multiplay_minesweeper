/**
 * 入力システムモジュール
 * 
 * ユーザー入力の処理を行うシステム群
 */

// サブモジュールをエクスポート
// pub mod keyboard_input_system; // 未実装モジュール
// pub mod mouse_input_system; // 未実装モジュール
pub mod board_click_system;
pub mod input_types;
pub mod input_collection_system;
pub mod input_processing_system;
pub mod ui_input_system;
pub mod gameplay_input_system;

// WASM向け設定
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

// 公開するシステムをエクスポート
// pub use keyboard_input_system::KeyboardInputSystem; // 未実装モジュール
// pub use mouse_input_system::MouseInputSystem; // 未実装モジュール
pub use board_click_system::BoardClickSystem;
pub use input_types::{MouseButton, InputEventType, InputEvent};
pub use input_collection_system::InputCollectionSystem;
pub use input_processing_system::InputProcessingSystem;
pub use ui_input_system::UIInputSystem;
pub use gameplay_input_system::GameplayInputSystem; 