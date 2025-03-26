/**
 * ECSパターンのシステム定義
 * マインスイーパーゲームのシステム実装
 */

// システムレジストリ
pub mod system_registry;

// 入力システム
pub mod input_system;

// 更新システム
pub mod update_system;

// 描画システム
pub mod render_system;

// ボードシステム
pub mod board_system;

// ネットワークシステム
pub mod network_system;

// UIシステム
pub mod ui_system;

// システム関連の型を再エクスポート
pub use system_registry::{SystemRegistry, SystemFn, DeltaTime};
pub use input_system::{process_input, process_mouse_move, process_mouse_click};
pub use update_system::{update_game_state, update_players, update_animations};
pub use render_system::{render_game, render_board, render_ui, render_players};
pub use board_system::{init_board, reveal_cell, toggle_flag, check_win_condition};
pub use network_system::{process_network_messages, send_player_updates};
pub use ui_system::{process_ui_interactions, update_ui_elements}; 