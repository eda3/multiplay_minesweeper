/**
 * プレイヤーエンティティの定義
 * 
 * マルチプレイヤーゲームにおけるプレイヤーをエンティティとして表現
 */
use crate::components::{PlayerComponent, Position};
use crate::entities::entity::{Entity, EntityId};
use crate::entities::entity_manager::EntityBuilder;

/// プレイヤーエンティティのタグ
pub const PLAYER_TAG: &str = "player";
pub const LOCAL_PLAYER_TAG: &str = "local_player";
pub const REMOTE_PLAYER_TAG: &str = "remote_player";

/// プレイヤーエンティティの種類を表す型
#[derive(Debug, Clone, PartialEq)]
pub enum PlayerEntity {
    /// ローカルプレイヤー（自分自身）
    LocalPlayer(EntityId),
    /// リモートプレイヤー（他のプレイヤー）
    RemotePlayer(EntityId),
}

impl PlayerEntity {
    /// エンティティIDを取得
    pub fn id(&self) -> EntityId {
        match self {
            PlayerEntity::LocalPlayer(id) => *id,
            PlayerEntity::RemotePlayer(id) => *id,
        }
    }
    
    /// ローカルプレイヤーかどうか
    pub fn is_local(&self) -> bool {
        matches!(self, PlayerEntity::LocalPlayer(_))
    }
}

/// ローカルプレイヤーエンティティを作成
pub fn create_local_player(builder: EntityBuilder, id: String, x: f64, y: f64) -> Entity {
    builder
        .with_component(PlayerComponent::local(id))
        .with_component(Position::new(x, y))
        .with_tag(PLAYER_TAG)
        .with_tag(LOCAL_PLAYER_TAG)
        .build()
}

/// リモートプレイヤーエンティティを作成
pub fn create_remote_player(builder: EntityBuilder, id: String, color: String, x: f64, y: f64) -> Entity {
    builder
        .with_component(PlayerComponent::remote(id, color))
        .with_component(Position::new(x, y))
        .with_tag(PLAYER_TAG)
        .with_tag(REMOTE_PLAYER_TAG)
        .build()
}

/// 汎用プレイヤーエンティティを作成（ローカル/リモートを自動判断）
pub fn create_player_entity(builder: EntityBuilder, id: String, color: String, x: f64, y: f64, is_local: bool) -> Entity {
    if is_local {
        create_local_player(builder, id, x, y)
    } else {
        create_remote_player(builder, id, color, x, y)
    }
}

/// プレイヤーエンティティに対する操作
/// 実際のエンティティマネージャーとエンティティIDを使用してプレイヤーを操作
pub mod player_operations {
    use super::*;
    use crate::entities::entity_manager::EntityManager;
    
    /// プレイヤーの位置を更新
    pub fn update_player_position(manager: &mut EntityManager, id: EntityId, x: f64, y: f64) -> bool {
        if let Some(entity) = manager.get_entity_mut(id) {
            if let Some(position) = entity.get_component_mut::<Position>() {
                position.x = x;
                position.y = y;
                
                // プレイヤーコンポーネントの最終更新時間も更新
                if let Some(player) = entity.get_component_mut::<PlayerComponent>() {
                    player.update_action_time();
                }
                
                return true;
            }
        }
        
        false
    }
    
    /// プレイヤーの位置を取得
    pub fn get_player_position(manager: &EntityManager, id: EntityId) -> Option<(f64, f64)> {
        manager.get_entity(id)
            .and_then(|entity| entity.get_component::<Position>())
            .map(|pos| (pos.x, pos.y))
    }
    
    /// プレイヤー情報を取得
    pub fn get_player_info(manager: &EntityManager, id: EntityId) -> Option<PlayerComponent> {
        manager.get_entity(id)
            .and_then(|entity| entity.get_component::<PlayerComponent>())
            .cloned()
    }
    
    /// ローカルプレイヤーのエンティティIDを取得
    pub fn get_local_player_id(manager: &EntityManager) -> Option<EntityId> {
        manager.get_entities_with_tag(LOCAL_PLAYER_TAG)
            .first()
            .copied()
    }
    
    /// 非アクティブなプレイヤーを検出（一定時間操作がないプレイヤー）
    pub fn find_inactive_players(manager: &EntityManager, timeout_ms: f64) -> Vec<EntityId> {
        let current_time = js_sys::Date::now();
        
        manager.get_entities_with_tag(PLAYER_TAG)
            .into_iter()
            .filter(|id| {
                if let Some(entity) = manager.get_entity(*id) {
                    if let Some(player) = entity.get_component::<PlayerComponent>() {
                        // タイムアウト時間より長く操作がないプレイヤーを検出
                        return current_time - player.last_action_time > timeout_ms;
                    }
                }
                false
            })
            .collect()
    }
} 