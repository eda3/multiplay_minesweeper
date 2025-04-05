/**
 * システム初期化ユーティリティ
 * 
 * システムの初期化に必要な共通処理を行う
 */
use crate::resources::ResourceManager;
use crate::resources::EventBusResource;
use crate::entities::EntityManager;

/// システムの初期化を行う
pub fn initialize_systems(resource_manager: &mut ResourceManager, entity_manager: &mut EntityManager) {
    // 各種リソースの初期化
    initialize_resources(resource_manager);
    
    // エンティティの初期化
    initialize_entities(entity_manager, resource_manager);
}

/// リソースの初期化
pub fn initialize_resources(resource_manager: &mut ResourceManager) {
    // 基本リソースがまだ登録されていなければ登録
    if !resource_manager.has::<EventBusResource>() {
        let event_bus = EventBusResource::new();
        resource_manager.add(event_bus).unwrap_or_else(|_| {
            log::warn!("EventBusResourceの登録に失敗しました");
        });
    }
    
    // その他の基本リソースの初期化
    // ...
}

/// エンティティの初期化
pub fn initialize_entities(entity_manager: &mut EntityManager, resource_manager: &mut ResourceManager) {
    // 基本エンティティの作成など
    // ...
} 