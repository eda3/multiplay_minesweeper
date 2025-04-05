/**
 * システムモジュール
 * 
 * ゲームの更新ロジックを担当するシステム群
 */

pub mod board_systems;
pub mod optimized;
pub mod input;

pub mod board_system;
pub mod network_system;
pub mod render_system;
pub mod ui_system;
pub mod update_system;
pub mod input_system;
pub mod system_registry;

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

// システムレジストリをインポート
pub use system_registry::{SystemRegistry, SystemFn, DeltaTime}; 