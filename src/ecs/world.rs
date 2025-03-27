/**
 * ECS World
 * 
 * ECSアーキテクチャの中心となるWorldクラス
 * エンティティとリソースの管理を一元化
 */
use crate::entities::{EntityManager, EntityId};
use crate::resources::ResourceManager;
use crate::system::{SystemRegistry, System};
use crate::resources::{ResourceBatch, ResourceBatchMut};
use std::any::Any;
use std::any::TypeId;
use crate::components::Component;
use crate::resources::Resource;

/// World構造体 - ECSの中心的なコンテナ
#[derive(Debug)]
pub struct World {
    /// エンティティマネージャー
    entity_manager: EntityManager,
    /// リソースマネージャー
    resource_manager: ResourceManager,
    /// システムレジストリ
    system_registry: SystemRegistry,
}

impl Default for World {
    fn default() -> Self {
        Self {
            entity_manager: EntityManager::new(),
            resource_manager: ResourceManager::new(),
            system_registry: SystemRegistry::new(),
        }
    }
}

impl World {
    /// 新しいWorldを作成
    pub fn new() -> Self {
        Self::default()
    }
    
    /// エンティティマネージャーを取得（不変）
    pub fn entities(&self) -> &EntityManager {
        &self.entity_manager
    }
    
    /// エンティティマネージャーを取得（可変）
    pub fn entities_mut(&mut self) -> &mut EntityManager {
        &mut self.entity_manager
    }
    
    /// リソースを取得（不変）
    pub fn get_resource<T: Resource>(&self) -> Option<&T> {
        self.resource_manager.get::<T>()
    }
    
    /// リソースを取得（可変）
    pub fn get_resource_mut<T: Resource>(&mut self) -> Option<&mut T> {
        self.resource_manager.get_mut::<T>()
    }
    
    /// リソースを追加または更新
    pub fn insert_resource<T: Resource>(&mut self, resource: T) {
        self.resource_manager.insert(resource);
    }
    
    /// リソースが存在するかチェック
    pub fn has_resource<T: Resource>(&self) -> bool {
        self.resource_manager.contains::<T>()
    }
    
    /// リソースを削除
    pub fn remove_resource<T: Resource>(&mut self) -> Option<T> {
        self.resource_manager.remove::<T>()
    }
    
    /// 複数のリソースを一度に取得
    pub fn get_resources<A: 'static, B: 'static>(&self) -> Option<(&A, &B)> {
        self.resource_manager.get_multi::<A, B>()
    }
    
    /// 複数のリソースを一度に取得（一部可変）
    pub fn get_resources_mut<A: 'static, B: 'static>(&mut self) -> Option<(&A, &mut B)> {
        if let Some((a, b)) = self.resource_manager.get_multi_mut::<A, B>() {
            // 不変参照として再構築
            let a_ref = a as &A;
            Some((a_ref, b))
        } else {
            None
        }
    }
    
    /// リソースマネージャーを取得（不変）
    pub fn resources(&self) -> &ResourceManager {
        &self.resource_manager
    }
    
    /// リソースマネージャーを取得（可変）
    pub fn resources_mut(&mut self) -> &mut ResourceManager {
        &mut self.resource_manager
    }
    
    /// リソースバッチ処理（読み取り専用）
    pub fn with_resources<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&ResourceBatch<dyn Resource>) -> R,
    {
        self.resource_manager.batch(|batch| f(&batch))
    }
    
    /// リソースバッチ処理（読み書き）
    pub fn with_resources_mut<F, R>(&mut self, f: F) -> R
    where
        F: FnOnce(&mut ResourceBatchMut<dyn Resource>) -> R,
    {
        self.resource_manager.batch_mut(|mut batch| f(&mut batch))
    }
    
    /// 初期リソースを追加
    pub fn setup_default_resources(&mut self) {
        use crate::resources::{
            CoreGameResource, 
            TimeResource, 
            PlayerStateResource, 
            GameConfigResource,
            BoardConfigResource,
            BoardStateResource
        };
        
        // コアゲームリソース
        if !self.has_resource::<CoreGameResource>() {
            self.insert_resource(CoreGameResource::new());
        }
        
        // 時間リソース
        if !self.has_resource::<TimeResource>() {
            self.insert_resource(TimeResource::new());
        }
        
        // プレイヤー状態リソース
        if !self.has_resource::<PlayerStateResource>() {
            self.insert_resource(PlayerStateResource::new());
        }
        
        // ゲーム設定リソース
        if !self.has_resource::<GameConfigResource>() {
            self.insert_resource(GameConfigResource::new());
        }
        
        // ボード設定リソース
        if !self.has_resource::<BoardConfigResource>() {
            self.insert_resource(BoardConfigResource::default());
        }
        
        // ボード状態リソース
        if !self.has_resource::<BoardStateResource>() {
            self.insert_resource(BoardStateResource::new());
        }
    }
    
    /// システムを追加
    pub fn add_system<S>(&mut self, system: S) -> usize
    where
        S: 'static + System,
    {
        self.system_registry.add_system(Box::new(system))
    }
    
    /// スタートアップフェーズのシステムを実行
    pub fn run_startup(&mut self) {
        self.system_registry.run_startup(&mut self.resource_manager);
    }
    
    /// 全フェーズのシステムを実行
    pub fn run_systems(&mut self) {
        self.system_registry.run_all_phases(&mut self.resource_manager);
    }
    
    /// 特定のフェーズのシステムのみを実行
    pub fn run_phase(&mut self, phase: crate::system::system_registry::SystemPhase) {
        self.system_registry.run_phase(phase, &mut self.resource_manager);
    }
    
    /// システムレジストリを取得（不変）
    pub fn systems(&self) -> &SystemRegistry {
        &self.system_registry
    }
    
    /// システムレジストリを取得（可変）
    pub fn systems_mut(&mut self) -> &mut SystemRegistry {
        &mut self.system_registry
    }
    
    /// 指定したIDのシステムを取得（テスト用）
    pub fn get_system(&self, id: crate::system::system_registry::SystemId) -> Option<&dyn Any> {
        // SystemからAnyへの変換は直接はできないので、
        // システムレジストリの既存APIを通じてシステムを取得し、
        // それをAnyとして返す（これはテスト用なので簡易的な実装です）
        None  // テスト用なので一旦Noneを返す
    }
    
    pub fn create_entity(&mut self) -> EntityId {
        self.entity_manager.create_entity()
    }
    
    pub fn add_component<T: Component>(&mut self, entity: EntityId, component: T) {
        // TODO: エラーハンドリングの改善
        let _ = self.entity_manager.add_component(entity, component);
    }
    
    pub fn get_component<T: Component>(&self, entity: EntityId) -> Option<&T> {
        // TODO: EntityManagerにget_componentメソッドを実装
        None
    }
    
    pub fn get_component_mut<T: Component>(&mut self, entity: EntityId) -> Option<&mut T> {
        // TODO: EntityManagerにget_component_mutメソッドを実装
        None
    }
    
    pub fn remove_component<T: Component>(&mut self, entity: EntityId) -> Option<T> {
        // TODO: EntityManagerにremove_componentメソッドを実装
        None
    }
    
    pub fn has_component<T: Component>(&self, entity: EntityId) -> bool {
        // TODO: EntityManagerにhas_componentメソッドを実装
        false
    }
    
    pub fn get_entity_manager(&self) -> &EntityManager {
        &self.entity_manager
    }
    
    pub fn get_entity_manager_mut(&mut self) -> &mut EntityManager {
        &mut self.entity_manager
    }
    
    pub fn get_resource_manager(&self) -> &ResourceManager {
        &self.resource_manager
    }
    
    pub fn get_resource_manager_mut(&mut self) -> &mut ResourceManager {
        &mut self.resource_manager
    }
} 