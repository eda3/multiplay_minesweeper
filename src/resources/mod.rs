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
pub mod event_bus_resource;
pub mod typed_event_bus_resource;

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
pub use event_bus_resource::EventBusResource;
pub use typed_event_bus_resource::TypedEventBusResource;

// モデルをインポート
// pub use board_state::{CellState, CellValue, Board, BoardConfig}; // board_state からは CellState, BoardConfig のみ使うか、正しいパスからインポート
pub use board_state::{CellState, BoardConfig};

pub use crate::models::cell::CellValue;
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
pub use crate::systems::system_registry::DeltaTime; // コンパイラ提案の systems からインポート

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
pub struct ResourceBatch<'a> {
    pub resources: &'a ResourceManager,
}

impl<'a> ResourceBatch<'a> {
    /// 指定した型のリソースを取得
    pub fn get<T: Resource>(&self) -> Option<&T> {
        match self.resources.get::<T>() {
            Ok(rc) => {
                let borrowed = rc.borrow();
                borrowed.downcast_ref::<T>()
                    .map(|r| unsafe { std::mem::transmute::<&T, &T>(r) })
            }
            Err(_) => None
        }
    }
}

/// リソースのバッチ処理のためのラッパー（読み書き可能）
pub struct ResourceBatchMut<'a> {
    pub resources: &'a mut ResourceManager,
}

impl<'a> ResourceBatchMut<'a> {
    /// 指定した型のリソースを取得（読み取り専用）
    pub fn get<T: Resource>(&self) -> Option<&T> {
        match self.resources.get::<T>() {
            Ok(rc) => {
                let borrowed = rc.borrow();
                borrowed.downcast_ref::<T>()
                    .map(|r| unsafe { std::mem::transmute::<&T, &T>(r) })
            }
            Err(_) => None
        }
    }
    
    /// 指定した型のリソースを取得（読み書き可能）
    pub fn get_mut<T: Resource>(&mut self) -> Option<&mut T> {
        match self.resources.get_mut::<T>() {
            Ok(rc) => {
                let mut borrowed = rc.borrow_mut();
                borrowed.downcast_mut::<T>()
                    .map(|r| unsafe { std::mem::transmute::<&mut T, &mut T>(r) })
            }
            Err(_) => None
        }
    }
}

/// リソースシステムの初期化
pub fn init() -> ResourceManager {
    ResourceManager::new()
} 