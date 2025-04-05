use super::resource_trait::Resource;
// モジュールからのインポートのみ残す
use crate::resources::{
    GameStateResource,
    NetworkResource,
    BoardResource,
    TimeResource,
    PlayerStateResource,
    InputResource,
    RenderResource,
    MouseState
};

// 全てブランケット実装があるため個別の実装は不要 