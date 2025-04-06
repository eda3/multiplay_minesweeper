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
use crate::resources::{
    GameStateResource, 
    GameConfigResource,
    BoardConfigResource,
    BoardStateResource
};
use crate::resources::{
    BoardResource
};
use wasm_bindgen::prelude::*;
use std::rc::Rc;
use std::cell::RefCell;
use crate::resources::{
    TimeResource, PlayerStateResource
};
use std::collections::HashMap;
use crate::system::system_registry::SystemPhase;
use crate::resources::InputResource;
use crate::resources::RenderResource;
use crate::resources::{
    EventBusResource,
    TypedEventBusResource
};

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
        match self.resource_manager.get::<T>() {
            Ok(rc) => {
                let borrowed = rc.borrow();
                borrowed.downcast_ref::<T>()
                    .map(|r| unsafe { std::mem::transmute::<&T, &T>(r) })
            }
            Err(_) => None
        }
    }
    
    /// リソースを取得（可変）
    pub fn get_resource_mut<T: Resource>(&mut self) -> Option<&mut T> {
        match self.resource_manager.get_mut::<T>() {
            Ok(rc) => {
                let mut borrowed = rc.borrow_mut();
                borrowed.downcast_mut::<T>()
                    .map(|r| unsafe { std::mem::transmute::<&mut T, &mut T>(r) })
            }
            Err(_) => None
        }
    }
    
    /// リソースを追加または更新
    pub fn insert_resource<T: Resource>(&mut self, resource: T) {
        if self.resource_manager.has::<T>() {
            let _ = self.resource_manager.update(resource);
        } else {
            let _ = self.resource_manager.add(resource);
        }
    }
    
    /// リソースが存在するかチェック
    pub fn has_resource<T: Resource>(&self) -> bool {
        self.resource_manager.has::<T>()
    }
    
    /// リソースを削除
    pub fn remove_resource<T: Resource>(&mut self) -> Option<T> {
        let _ = self.resource_manager.remove::<T>();
        None
    }
    
    /// 複数のリソースを一度に取得
    pub fn get_resources<A: Resource, B: Resource>(&self) -> Option<(&A, &B)> {
        let a = self.get_resource::<A>()?;
        let b = self.get_resource::<B>()?;
        Some((a, b))
    }
    
    /// 3つのリソースを一度に取得
    pub fn get_resources3<A: Resource, B: Resource, C: Resource>(&self) -> Option<(&A, &B, &C)> {
        let a = self.get_resource::<A>()?;
        let b = self.get_resource::<B>()?;
        let c = self.get_resource::<C>()?;
        Some((a, b, c))
    }
    
    /// 複数のリソースを一度に取得（一部可変）
    pub fn get_resources_mut<A: Resource, B: Resource>(&mut self) -> Option<(&A, &mut B)> {
        let a_ptr = match self.resource_manager.get::<A>() {
            Ok(rc) => {
                let borrowed = rc.borrow();
                borrowed.downcast_ref::<A>()
                    .map(|r| r as *const A)
            }
            Err(_) => None
        }?;
        
        let b_ptr = match self.resource_manager.get_mut::<B>() {
            Ok(rc) => {
                let mut borrowed = rc.borrow_mut();
                borrowed.downcast_mut::<B>()
                    .map(|r| r as *mut B)
            }
            Err(_) => None
        }?;
        
        // 安全性チェック：AとBが異なる型である必要がある
        if std::any::TypeId::of::<A>() == std::any::TypeId::of::<B>() {
            return None;
        }
        
        // ポインタを安全に参照に変換
        unsafe {
            Some((&*a_ptr, &mut *b_ptr))
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
        F: FnOnce(&ResourceBatch) -> R,
    {
        // リソースバッチを作成
        let batch = ResourceBatch {
            resources: &self.resource_manager
        };
        
        // 関数に渡して実行
        f(&batch)
    }
    
    /// リソースバッチ処理（読み書き）
    pub fn with_resources_mut<F, R>(&mut self, f: F) -> R
    where
        F: FnOnce(&mut ResourceBatchMut) -> R,
    {
        // 可変リソースバッチを作成
        let mut batch = ResourceBatchMut {
            resources: &mut self.resource_manager
        };
        
        // 関数に渡して実行
        f(&mut batch)
    }
    
    /// 初期リソースを追加
    pub fn setup_default_resources(&mut self) {
        use crate::resources::{
            GameStateResource,
            TimeResource, 
            PlayerStateResource, 
            GameConfigResource,
            BoardConfigResource,
            BoardStateResource,
            EventQueueResource,
            EventBusResource,
            TypedEventBusResource
        };
        
        // コアゲームリソース → GameStateResourceに置き換え
        if !self.has_resource::<GameStateResource>() {
            self.insert_resource(GameStateResource::new());
        }
        
        // 時間リソース
        if !self.has_resource::<TimeResource>() {
            self.insert_resource(TimeResource::new());
        }
        
        // 基本リソースを登録
        if !self.has_resource::<PlayerStateResource>() {
            self.insert_resource(PlayerStateResource::new("Player"));
        }
        
        // ゲーム設定リソース
        if !self.has_resource::<GameConfigResource>() {
            self.insert_resource(GameConfigResource::new());
        }
        
        // ボード設定リソース
        if !self.has_resource::<BoardConfigResource>() {
            self.insert_resource(BoardConfigResource::default());
        }
        
        // ボード関連リソースを登録
        if !self.has_resource::<BoardStateResource>() {
            self.insert_resource(BoardStateResource::new());
        }
        
        // イベントキューリソース
        if !self.has_resource::<EventQueueResource>() {
            self.insert_resource(EventQueueResource::new());
        }
        
        // イベントバスリソース
        if !self.has_resource::<EventBusResource>() {
            self.insert_resource(EventBusResource::new());
        }
        
        // 型安全なイベントバスリソース
        if !self.has_resource::<TypedEventBusResource>() {
            self.insert_resource(TypedEventBusResource::new());
        }
        
        self.insert_resource(RenderResource::default());
        self.insert_resource(GameConfigResource::default());
        self.insert_resource(InputResource::default());
        self.insert_resource(GameConfigResource::default());
    }
    
    /// システムを追加
    pub fn add_system<S>(&mut self, system: S) -> usize
    where
        S: 'static + System,
    {
        // SystemトレイトをSystemDefinitionに変換することはできないので、
        // ここではとりあえずインデックスを返すようにします
        // (あとでもっと良い解決策を実装する必要があります)
        
        // 今はsystems配列のサイズを返す（不完全な実装）
        let system_id = self.system_registry.len();
        
        // システムの名前を取得
        let name = system.name();
        
        // 実際には適切な変換関数を実装するか、
        // システム登録の仕組みをより詳細に設計する必要があります
        
        system_id
    }
    
    /// スタートアップフェーズのシステムを実行
    pub fn run_startup(&mut self) {
        self.run_phase(SystemPhase::Startup);
    }
    
    /// 全フェーズのシステムを実行
    pub fn run_systems(&mut self) {
        self.system_registry.run_all_phases(&mut self.resource_manager);
    }
    
    /// 特定のフェーズのシステムのみを実行
    pub fn run_phase(&mut self, phase: SystemPhase) {
        // システムレジストリに対応するフェーズのSystemPriorityに変換して実行
        let priority = match phase {
            SystemPhase::Startup => crate::system::system_registry::SystemPriority::First,
            SystemPhase::Input => crate::system::system_registry::SystemPriority::Input,
            SystemPhase::Update => crate::system::system_registry::SystemPriority::Update,
            SystemPhase::Render => crate::system::system_registry::SystemPriority::Render,
            SystemPhase::Cleanup => crate::system::system_registry::SystemPriority::Last,
        };
        
        self.system_registry.run_phase(&mut self.resource_manager, priority);
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
    pub fn get_system(&self, id: &'static str) -> Option<&dyn Any> {
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
    
    /// 3つのリソースを一度に取得（1つのみ可変）
    pub fn get_resources3_mut<A: Resource, B: Resource, C: Resource>(&mut self) -> Option<(&A, &B, &mut C)> {
        // リソースの型IDを取得
        let a_id = std::any::TypeId::of::<A>();
        let b_id = std::any::TypeId::of::<B>();
        let c_id = std::any::TypeId::of::<C>();
        
        // 同じ型が含まれていないかチェック
        if a_id == b_id || a_id == c_id || b_id == c_id {
            return None;
        }
        
        // 各リソースを取得
        let a_ptr = match self.resource_manager.get::<A>() {
            Ok(rc) => {
                let borrowed = rc.borrow();
                borrowed.downcast_ref::<A>()
                    .map(|r| r as *const A)
            }
            Err(_) => None
        }?;
        
        let b_ptr = match self.resource_manager.get::<B>() {
            Ok(rc) => {
                let borrowed = rc.borrow();
                borrowed.downcast_ref::<B>()
                    .map(|r| r as *const B)
            }
            Err(_) => None
        }?;
        
        let c_ptr = match self.resource_manager.get_mut::<C>() {
            Ok(rc) => {
                let mut borrowed = rc.borrow_mut();
                borrowed.downcast_mut::<C>()
                    .map(|r| r as *mut C)
            }
            Err(_) => None
        }?;
        
        // ポインタを安全に参照に変換
        unsafe {
            Some((&*a_ptr, &*b_ptr, &mut *c_ptr))
        }
    }
    
    /// 3つのリソースを一度に取得（2つが可変）
    pub fn get_resources3_mut2<A: Resource, B: Resource, C: Resource>(&mut self) -> Option<(&A, &mut B, &mut C)> {
        // リソースの型IDを取得
        let a_id = std::any::TypeId::of::<A>();
        let b_id = std::any::TypeId::of::<B>();
        let c_id = std::any::TypeId::of::<C>();
        
        // 同じ型が含まれていないかチェック
        if a_id == b_id || a_id == c_id || b_id == c_id {
            return None;
        }
        
        // 各リソースを取得
        let a_ptr = match self.resource_manager.get::<A>() {
            Ok(rc) => {
                let borrowed = rc.borrow();
                borrowed.downcast_ref::<A>()
                    .map(|r| r as *const A)
            }
            Err(_) => None
        }?;
        
        // 可変リソースを取得
        let b_ptr = match self.resource_manager.get_mut::<B>() {
            Ok(rc) => {
                let mut borrowed = rc.borrow_mut();
                borrowed.downcast_mut::<B>()
                    .map(|r| r as *mut B)
            }
            Err(_) => None
        }?;
        
        // 可変リソースを取得
        let c_ptr = match self.resource_manager.get_mut::<C>() {
            Ok(rc) => {
                let mut borrowed = rc.borrow_mut();
                borrowed.downcast_mut::<C>()
                    .map(|r| r as *mut C)
            }
            Err(_) => None
        }?;
        
        // ポインタを安全に参照に変換
        unsafe {
            Some((&*a_ptr, &mut *b_ptr, &mut *c_ptr))
        }
    }
}

// ダミーリソース（一時的な実装用）
struct DummyResource;

struct VoidResource; 