/**
 * システムモジュール
 * 
 * ゲームの各システムを定義するモジュール
 */

// 最適化されたECS実装
pub mod optimized;

// 各システムのモジュール
pub mod input_system;
pub mod render_system;
pub mod update_system;
pub mod network_system;
pub mod ui_system;

// システムをエクスポート
pub use input_system::input_system;
pub use render_system::render_system;
pub use update_system::{update_system, game_tick_system};
pub use network_system::network_system;
pub use ui_system::ui_system;

// システムレジストリ
pub mod system_registry;

// ボードシステム
pub mod board_system;

// ボード関連の新しいシステム
pub mod board_systems;

// システムレジストリ関連の型を再エクスポート
pub use system_registry::{SystemRegistry, SystemFn, DeltaTime}; 