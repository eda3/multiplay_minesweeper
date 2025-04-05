/**
 * システムモジュール
 * 
 * ゲームの更新ロジックを担当するシステム群
 */

pub mod board_systems;
pub mod optimized;
pub mod input;
pub mod input_systems;

pub mod board_system;
pub mod network_system;
pub mod render_system;
pub mod ui_system;
pub mod update_system;
pub mod input_system;
pub mod system_registry;
pub mod event_system_trait;

// 標準システムをインポート
pub use board_system::board_system;
pub use network_system::network_system;
pub use render_system::render_system;
pub use ui_system::ui_system;
pub use update_system::update_system;
pub use input_system::input_system;

// 新しいシステムをインポート
pub use input::InputCollectionSystem;
pub use input::InputProcessingSystem;
pub use input::register_input_systems;
pub use input_systems::BoardClickSystem;
pub use board_systems::CellRevealHandlerSystem;

// システムレジストリをインポート
pub use system_registry::{SystemRegistry, SystemFn, DeltaTime};

// 必要な型をエクスポート
pub use event_system_trait::{EventSystemTrait, EventSystem};

/// イベント駆動システムをシステムレジストリに登録する
pub fn register_event_driven_systems(registry: &mut crate::system::system_registry::SystemRegistry) {
    // 入力システム
    let board_click_system = BoardClickSystem::new();
    registry.add_system(Box::new(board_click_system));
    
    // ボードシステム
    let cell_reveal_handler = CellRevealHandlerSystem::new();
    registry.add_system(Box::new(cell_reveal_handler));
    
    // その他のイベント駆動システムを追加
} 