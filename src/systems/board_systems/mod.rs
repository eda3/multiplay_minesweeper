/**
 * ボードシステムモジュール
 * 
 * ボード関連の処理を行うシステム群
 */

// サブモジュールをエクスポート
pub mod board_init_system;
pub mod cell_reveal_system;
pub mod flag_toggle_system;
pub mod win_condition_system;
// pub mod board_render_system; // 未実装モジュール
pub mod cell_reveal_handler_system;

// 公開するシステムをエクスポート
// board_init_systemは関数として実装されているため、構造体としてはインポートしない
pub use cell_reveal_system::CellRevealSystem;
pub use flag_toggle_system::FlagToggleSystem;
pub use win_condition_system::WinConditionSystem;
// pub use board_render_system::BoardRenderSystem; // 未実装モジュール
pub use cell_reveal_handler_system::CellRevealHandlerSystem; 