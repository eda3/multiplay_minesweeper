/**
 * システムモジュールのエントリポイント
 * 
 * ゲームロジックを実行するシステムの定義とエクスポート
 */

// エクスポートするサブモジュール
pub mod system_registry;
// pub mod system_manager; // 未実装
pub mod event_system_trait;
pub mod typed_event_system_trait; // 新しい型安全なイベントシステムトレイト

// 機能別システムのサブモジュール
pub mod board_systems;
pub mod input_systems;
// pub mod ui_systems; // 未実装
// pub mod game_systems; // 未実装
// pub mod network_systems; // 未実装
// pub mod util_systems; // 未実装

// プレイヤー関連システム
// pub mod player_systems; // 未実装

// AI関連システム
// pub mod ai_systems; // 未実装

// ユーティリティ関連システム
// pub mod utility_systems; // 未実装

// システムマネージャーとレジストリの再エクスポート
// pub use system_manager::SystemManager; // 未実装
pub use system_registry::SystemRegistry;
pub use crate::system::system_registry::SystemPhase;
pub use crate::system::SystemDefinition;
pub use crate::ecs::system::System;

// イベントシステムの再エクスポート
pub use event_system_trait::{EventSystemTrait, EventSystem};
pub use typed_event_system_trait::{TypedEventSystemTrait, TypedEventSystem}; // 型安全なイベントシステム

/**
 * システムの初期化関数
 */
pub fn init() -> SystemRegistry {
    let mut system_registry = SystemRegistry::new();
    
    // プリセットのシステムを登録
    board_systems::register_board_systems(&mut system_registry);
    
    system_registry
} 