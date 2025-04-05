use super::resource_trait::Resource;
// モジュールからのインポートのみ残す
use crate::resources::{ 
    NetworkResource,
    BoardResource,
    TimeResource,
    PlayerStateResource,
    InputResource,
    RenderResource,
    MouseState
};
use super::core_game::GameStateResource;
// use super::player_state::PlayerStateResource; // 重複インポートのためコメントアウト

// 全てブランケット実装があるため個別の実装は不要 

#[derive(Debug)]
pub enum GameResource {
    GameStateResource,
    PlayerStateResource,
} 