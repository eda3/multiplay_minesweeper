/**
 * リソースモジュール
 * 
 * ゲームのグローバルな状態を表すリソースの定義
 */

// サブモジュールをエクスポート
pub mod resource_trait;
pub mod resource_manager;
pub mod core_game;
pub mod time_resource;
pub mod player_state;
pub mod board_state;
pub mod render_state;
pub mod network_state;
pub mod mouse_state;
pub mod input_resource;
pub mod resource_impl;  // Resourceトレイト実装モジュール
pub mod board_config;
pub mod game_config;

// 公開するモジュールと型
pub use resource_trait::Resource;
pub use resource_manager::ResourceManager;
pub use core_game::{GameStateResource, GamePhase, DifficultyLevel, GameMode};
pub use time_resource::TimeResource;
pub use player_state::PlayerStateResource;
pub use board_state::{BoardResource, BoardConfig, Cell, CellState};
pub use board_state::BoardResource as BoardStateResource;
pub use board_config::BoardConfig as BoardConfigResource;
pub use game_config::GameConfigResource;
pub use render_state::RenderResource;
pub use network_state::NetworkResource;
pub use mouse_state::MouseState;
pub use input_resource::InputResource;
// GameStateResourceをCoreGameResourceのエイリアスとして公開
pub use core_game::GameStateResource as CoreGameResource;

// リソース関連のエラー
#[derive(Debug, Clone)]
pub enum ResourceError {
    NotFound(String),
    WrongType(String),
    AlreadyExists(String),
}

impl std::fmt::Display for ResourceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ResourceError::NotFound(name) => write!(f, "Resource not found: {}", name),
            ResourceError::WrongType(name) => write!(f, "Resource type mismatch: {}", name),
            ResourceError::AlreadyExists(name) => write!(f, "Resource already exists: {}", name),
        }
    }
}

/// リソースのバッチ処理のためのラッパー（読み取り専用）
pub struct ResourceBatch<T: ?Sized + 'static> {
    pub resource: &'static T,
}

/// リソースのバッチ処理のためのラッパー（読み書き可能）
pub struct ResourceBatchMut<T: ?Sized + 'static> {
    pub resource: &'static mut T,
} 