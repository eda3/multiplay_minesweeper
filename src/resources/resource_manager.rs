/**
 * リソースマネージャー
 * 
 * ゲーム全体のリソースを管理するクラス
 */
use std::any::{Any, TypeId};
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::Debug;
use std::rc::Rc;

use super::resource_trait::Resource;

/// リソースの取得に失敗した場合のエラー
#[derive(Debug, Clone)]
pub enum ResourceError {
    /// リソースが見つからない
    NotFound(String),
    /// 型が一致しない
    WrongType(String),
    /// すでに存在する
    AlreadyExists(String),
}

impl std::fmt::Display for ResourceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ResourceError::NotFound(msg) => write!(f, "リソースが見つかりません: {}", msg),
            ResourceError::WrongType(msg) => write!(f, "リソースの型が違います: {}", msg),
            ResourceError::AlreadyExists(msg) => write!(f, "リソースはすでに存在します: {}", msg),
        }
    }
}

/// リソースの結果型
pub type ResourceResult<T> = Result<T, ResourceError>;

/// リソースマネージャー
/// 
/// アプリケーション全体で共有されるリソースを管理
#[derive(Debug, Default)]
pub struct ResourceManager {
    /// リソースマップ
    resources: HashMap<TypeId, Rc<RefCell<dyn Any>>>,
}

impl ResourceManager {
    /// 新しいリソースマネージャーを作成
    pub fn new() -> Self {
        Self {
            resources: HashMap::new(),
        }
    }
    
    /// リソースを追加
    pub fn add<R: Resource>(&mut self, resource: R) -> ResourceResult<()> {
        let type_id = TypeId::of::<R>();
        
        if self.resources.contains_key(&type_id) {
            return Err(ResourceError::AlreadyExists(
                std::any::type_name::<R>().to_string(),
            ));
        }
        
        self.resources.insert(type_id, Rc::new(RefCell::new(resource)));
        Ok(())
    }
    
    /// リソースを取得
    pub fn get<R: Resource>(&self) -> ResourceResult<Rc<RefCell<dyn Any>>> {
        let type_id = TypeId::of::<R>();
        
        self.resources
            .get(&type_id)
            .cloned()
            .ok_or_else(|| {
                ResourceError::NotFound(std::any::type_name::<R>().to_string())
            })
    }
    
    /// リソースを更新（既存のものを置き換え）
    pub fn update<R: Resource>(&mut self, resource: R) -> ResourceResult<()> {
        let type_id = TypeId::of::<R>();
        
        if !self.resources.contains_key(&type_id) {
            return Err(ResourceError::NotFound(
                std::any::type_name::<R>().to_string(),
            ));
        }
        
        self.resources.insert(type_id, Rc::new(RefCell::new(resource)));
        Ok(())
    }
    
    /// リソースを削除
    pub fn remove<R: Resource>(&mut self) -> Result<(), ResourceError> {
        let type_id = TypeId::of::<R>();
        if self.resources.remove(&type_id).is_some() {
            Ok(())
        } else {
            Err(ResourceError::NotFound(
                std::any::type_name::<R>().to_string(),
            ))
        }
    }
    
    /// リソースがあるかどうかを確認
    pub fn has<R: Resource>(&self) -> bool {
        let type_id = TypeId::of::<R>();
        self.resources.contains_key(&type_id)
    }
    
    /// すべてのリソースをクリア
    pub fn clear(&mut self) {
        self.resources.clear();
    }
    
    /// リソースの数を取得
    pub fn count(&self) -> usize {
        self.resources.len()
    }
    
    /// 新しいリソースを追加するか、既存のリソースを更新
    pub fn add_or_update<R: Resource>(&mut self, resource: R) {
        let type_id = TypeId::of::<R>();
        self.resources.insert(type_id, Rc::new(RefCell::new(resource)));
    }
    
    /// リソースを取得し、指定した型にダウンキャスト
    pub fn get_as<R: Resource + Clone>(&self) -> ResourceResult<Rc<RefCell<R>>> {
        let rc = self.get::<R>()?;
        
        // 型を確認し、適切な型のRc<RefCell<R>>を返す
        // これはリソースの型が正しいことを保証するために行う
        let borrowed = rc.borrow();
        if let Some(resource) = borrowed.downcast_ref::<R>() {
            // リソースをクローンして新しいRc<RefCell>を作成
            let resource_clone = resource.clone();
            return Ok(Rc::new(RefCell::new(resource_clone)));
        }
        
        Err(ResourceError::WrongType(
            format!(
                "リソースの型が一致しません。期待: {}, 実際: unknown",
                std::any::type_name::<R>()
            )
        ))
    }

    /// リソースが存在するかチェック
    pub fn contains<R: Resource>(&self) -> bool {
        let type_id = TypeId::of::<R>();
        self.resources.contains_key(&type_id)
    }

    /// リソースを挿入
    pub fn insert<R: Resource>(&mut self, resource: R) {
        let type_id = TypeId::of::<R>();
        let boxed: Box<dyn Any> = Box::new(resource);
        let rc = Rc::new(RefCell::new(boxed));
        self.resources.insert(type_id, rc);
    }

    /// リソースの数を取得
    pub fn len(&self) -> usize {
        self.resources.len()
    }

    /// リソースマネージャーが空かどうかを取得
    pub fn is_empty(&self) -> bool {
        self.resources.is_empty()
    }

    /// エンティティの数を取得（EntityManagerとの互換性のため）
    pub fn get_entity_count(&self) -> usize {
        0 // 実際のエンティティマネージャーがなければ0を返す
    }

    /// リソースを取得（可変）
    pub fn get_mut<R: Resource>(&mut self) -> ResourceResult<Rc<RefCell<dyn Any>>> {
        self.get::<R>()
    }

    pub fn resources(&self) -> &HashMap<TypeId, Rc<RefCell<dyn Any>>> {
        &self.resources
    }

    pub fn get_by_type_id(&self, type_id: &TypeId) -> Option<Rc<RefCell<dyn Any>>> {
        self.resources.get(type_id).cloned()
    }
}

// テスト
#[cfg(test)]
mod tests {
    use super::*;
    
    // テスト用のダミーリソース
    #[derive(Debug, Clone, PartialEq)]
    struct TestResource {
        pub value: i32,
    }
    
    impl Resource for TestResource {}
    
    #[test]
    fn test_add_and_get_resource() {
        let mut manager = ResourceManager::new();
        let resource = TestResource { value: 42 };
        
        // リソースを追加
        assert!(manager.add(resource.clone()).is_ok());
        
        // リソースを取得
        let retrieved = manager.get::<TestResource>().unwrap();
        let borrowed = retrieved.borrow();
        let cast = borrowed.downcast_ref::<TestResource>().unwrap();
        assert_eq!(cast.value, 42);
    }
    
    #[test]
    fn test_add_duplicate_resource() {
        let mut manager = ResourceManager::new();
        let resource = TestResource { value: 42 };
        
        // 最初の追加は成功するはず
        assert!(manager.add(resource.clone()).is_ok());
        
        // 同じ型の2回目の追加は失敗するはず
        assert!(manager.add(resource.clone()).is_err());
    }
    
    #[test]
    fn test_update_resource() {
        let mut manager = ResourceManager::new();
        let resource1 = TestResource { value: 42 };
        let resource2 = TestResource { value: 43 };
        
        // リソースを追加
        assert!(manager.add(resource1).is_ok());
        
        // リソースを更新
        assert!(manager.update(resource2).is_ok());
        
        // 更新されたリソースを確認
        let retrieved = manager.get::<TestResource>().unwrap();
        let borrowed = retrieved.borrow();
        let cast = borrowed.downcast_ref::<TestResource>().unwrap();
        assert_eq!(cast.value, 43);
    }
    
    #[test]
    fn test_remove_resource() {
        let mut manager = ResourceManager::new();
        let resource = TestResource { value: 42 };
        
        // リソースを追加
        assert!(manager.add(resource).is_ok());
        assert!(manager.has::<TestResource>());
        
        // リソースを削除
        assert!(manager.remove::<TestResource>().is_ok());
        assert!(!manager.has::<TestResource>());
        
        // 存在しないリソースの削除は失敗するはず
        assert!(manager.remove::<TestResource>().is_err());
    }
    
    #[test]
    fn test_add_or_update() {
        let mut manager = ResourceManager::new();
        let resource1 = TestResource { value: 42 };
        let resource2 = TestResource { value: 43 };
        
        // 存在しないリソースの追加
        manager.add_or_update(resource1);
        assert!(manager.has::<TestResource>());
        
        // 既存のリソースの更新
        manager.add_or_update(resource2);
        
        // 更新されたリソースを確認
        let retrieved = manager.get::<TestResource>().unwrap();
        let borrowed = retrieved.borrow();
        let cast = borrowed.downcast_ref::<TestResource>().unwrap();
        assert_eq!(cast.value, 43);
    }
    
    #[test]
    fn test_clear_resources() {
        let mut manager = ResourceManager::new();
        
        // 複数のリソースを追加
        manager.add(TestResource { value: 42 }).unwrap();
        
        #[derive(Debug)]
        struct AnotherResource;
        impl Resource for AnotherResource {}
        
        manager.add(AnotherResource).unwrap();
        
        assert_eq!(manager.count(), 2);
        
        // すべてのリソースをクリア
        manager.clear();
        assert_eq!(manager.count(), 0);
        assert!(!manager.has::<TestResource>());
    }
} 