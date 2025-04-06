/**
 * システムモジュールのエントリポイント
 * 
 * ゲームロジックを実行するシステムの定義とエクスポート
 */

// エクスポートするサブモジュール
pub mod system_registry;
pub mod event_system_trait;
pub mod typed_event_system_trait;
pub mod base_event_system;

// 機能別システムのサブモジュール
pub mod board_systems;
pub mod input_systems;
pub mod render_system;
pub mod board_system;
pub mod network_system;
pub mod ui_system;
pub mod update_system;
pub mod input_system;

// WASM向け設定
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

// システムレジストリの再エクスポート
pub use system_registry::{SystemRegistry, SystemPriority, DeltaTime};
pub use crate::ecs::system::System;

// イベントシステムの再エクスポート
pub use event_system_trait::{EventSystemTrait, EventSystem};
pub use typed_event_system_trait::{TypedEventSystemTrait, TypedEventSystem};
pub use base_event_system::{BaseEventSystem, EventQueue, EventRequest, EventProcessingState};

// 入力システムの型とコンポーネントの再エクスポート
pub use input_systems::{MouseButton, InputEventType, InputEvent};
pub use input_systems::{
    InputCollectionSystem,
    InputProcessingSystem,
    UIInputSystem,
    GameplayInputSystem,
    board_click_system::BoardClickSystem
};

/**
 * システムの初期化関数
 */
pub fn init() -> SystemRegistry {
    let mut system_registry = SystemRegistry::new();
    
    // プリセットのシステムを登録
    board_systems::register_board_systems(&mut system_registry);
    
    // 新しい入力システムの登録
    // system_registry.register_system("UIInputSystem", UIInputSystem::new(), SystemPriority::Input);
    // system_registry.register_system("GameplayInputSystem", GameplayInputSystem::new(), SystemPriority::Input);
    
    system_registry
} 