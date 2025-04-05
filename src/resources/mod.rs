/**
 * リソースモジュール
 * 
 * ECSパターンにおけるリソース（システム間で共有されるグローバルなデータ）を定義します
 */

// サブモジュールをエクスポート
pub mod board_resources;
pub mod resource_trait;
pub mod player_state;
pub mod time_resource;
pub mod event_queue_resource;
pub mod network_state;
pub mod render_state;
pub mod resource_manager;
pub mod draw_options;

// リソースを再エクスポート
pub use resource_trait::Resource;
pub use resource_manager::ResourceManager;
pub use board_resources::{BoardConfigResource, BoardStateResource};
pub use player_state::PlayerStateResource;
pub use time_resource::TimeResource;
pub use event_queue_resource::{EventQueueResource, GameEvent};
pub use network_state::NetworkResource;
pub use render_state::RenderResource;
pub use draw_options::DrawOptions;

// モデルをインポート
// pub use board_state::{CellState, CellValue, Board, BoardConfig}; // board_state からは CellState, BoardConfig のみ使うか、正しいパスからインポート
pub use board_state::{CellState, BoardConfig};

pub use crate::models::CellValue;
// pub use crate::board::Board; // BoardResource エイリアスを使うので不要か？

// 型エイリアス
pub type BoardResource = crate::board::Board;

// レガシーな定義（互換性のため）
pub mod board_state;
pub mod board_config;
pub mod core_game;
pub mod time;
pub mod input_resource;
pub mod mouse_state;
pub mod game_state;
pub mod game_config;
pub mod resource_impl;

// pub use board_state::{CellState, CellValue, Board, BoardConfig}; // 再度コメントアウト
// pub use board_config::BoardConfig as OldBoardConfig; // 削除
// pub use core_game::{GamePhase, CoreGameResource}; // CoreGameResource は core_game にない可能性
// pub use game_state::{GameStateResource, DifficultyLevel}; // game_state からは何も使わないか、core_game からインポート
// pub use core_game::{GameStateResource, DifficultyLevel}; // GameStateResource, DifficultyLevel も core_game にない可能性

// 正しいと思われるパスからインポート
pub use crate::resources::core_game::{GamePhase, DifficultyLevel}; // CoreGameResource を削除
pub use crate::resources::core_game::GameStateResource; 
// pub use crate::resources::core_game::CoreGameResource; // 削除

// pub use time::DeltaTime; // time からは何も使わないか、正しいパスからインポート
// pub use crate::resources::time_resource::DeltaTime; // time_resource にもない可能性
pub use crate::systems::DeltaTime; // コンパイラ提案の systems からインポート

pub use input_resource::InputResource;
pub use mouse_state::MouseState;
pub use game_config::GameConfigResource;

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