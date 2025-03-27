/**
 * ECS World
 * 
 * ECSアーキテクチャの中心となるWorldクラス
 * エンティティとリソースの管理を一元化
 */
use crate::entities::EntityManager;
use crate::resources::ResourceManager;

/// World構造体 - ECSの中心的なコンテナ
#[derive(Debug)]
pub struct World {
    /// エンティティマネージャー
    entity_manager: EntityManager,
    /// リソースマネージャー
    resource_manager: ResourceManager,
}

impl Default for World {
    fn default() -> Self {
        Self {
            entity_manager: EntityManager::new(),
            resource_manager: ResourceManager::new(),
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
    pub fn get_resource<T: 'static>(&self) -> Option<&T> {
        self.resource_manager.get::<T>()
    }
    
    /// リソースを取得（可変）
    pub fn get_resource_mut<T: 'static>(&mut self) -> Option<&mut T> {
        self.resource_manager.get_mut::<T>()
    }
    
    /// リソースを追加または更新
    pub fn insert_resource<T: 'static>(&mut self, resource: T) {
        self.resource_manager.insert(resource);
    }
    
    /// リソースが存在するかチェック
    pub fn has_resource<T: 'static>(&self) -> bool {
        self.resource_manager.contains::<T>()
    }
    
    /// リソースを削除
    pub fn remove_resource<T: 'static>(&mut self) -> Option<T> {
        self.resource_manager.remove::<T>()
    }
    
    /// 複数のリソースを一度に取得
    pub fn get_resources<A: 'static, B: 'static>(&self) -> Option<(&A, &B)> {
        self.resource_manager.get_multi::<A, B>()
    }
    
    /// 複数のリソースを一度に取得（一部可変）
    pub fn get_resources_mut<A: 'static, B: 'static>(&mut self) -> Option<(&A, &mut B)> {
        self.resource_manager.get_multi_mut::<A, B>()
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
        F: FnOnce(&crate::resources::ResourceBatch) -> R,
    {
        self.resource_manager.batch(f)
    }
    
    /// リソースバッチ処理（読み書き）
    pub fn with_resources_mut<F, R>(&mut self, f: F) -> R
    where
        F: FnOnce(&mut crate::resources::ResourceBatchMut) -> R,
    {
        self.resource_manager.batch_mut(f)
    }
    
    /// 初期リソースを追加
    pub fn setup_default_resources(&mut self) {
        use crate::resources::{
            CoreGameResource, 
            TimeResource, 
            PlayerStateResource, 
            GameConfigResource
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
    }
} 