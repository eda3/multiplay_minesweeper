/**
 * リソースマネージャー
 * 
 * ECSパターンでグローバルに共有されるリソースを管理します。
 * 型ごとに一意のリソースインスタンスを保持します。
 */
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::fmt::Debug;
use std::cell::{RefCell, Ref, RefMut};
use std::marker::PhantomData;

use super::resource_trait::Resource;

/// リソースマネージャー
/// 型安全にさまざまなリソースを保持・管理する
#[derive(Default)]
pub struct ResourceManager {
    /// リソースを型IDで管理するマップ
    resources: HashMap<TypeId, Box<dyn Any + Send + Sync>>,
}

impl Debug for ResourceManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ResourceManager")
            .field("resource_count", &self.resources.len())
            .field("resource_type_ids", &self.resources.keys().collect::<Vec<_>>())
            .finish()
    }
}

impl ResourceManager {
    /// 新しいリソースマネージャーを作成
    pub fn new() -> Self {
        Self {
            resources: HashMap::new(),
        }
    }
    
    /// リソースを追加または更新
    pub fn insert<R: Resource>(&mut self, resource: R) {
        let type_id = TypeId::of::<R>();
        self.resources.insert(type_id, Box::new(resource));
    }
    
    /// リソースへの不変参照を取得
    pub fn get<R: Resource>(&self) -> Option<&R> {
        let type_id = TypeId::of::<R>();
        self.resources
            .get(&type_id)
            .and_then(|boxed| boxed.downcast_ref::<R>())
    }
    
    /// リソースへの可変参照を取得
    pub fn get_mut<R: Resource>(&mut self) -> Option<&mut R> {
        let type_id = TypeId::of::<R>();
        self.resources
            .get_mut(&type_id)
            .and_then(|boxed| boxed.downcast_mut::<R>())
    }
    
    /// リソースが存在するかチェック
    pub fn contains<R: Resource>(&self) -> bool {
        let type_id = TypeId::of::<R>();
        self.resources.contains_key(&type_id)
    }
    
    /// リソースを削除し、削除したリソースを返す
    pub fn remove<R: Resource>(&mut self) -> Option<R> {
        let type_id = TypeId::of::<R>();
        self.resources
            .remove(&type_id)
            .and_then(|boxed| boxed.downcast::<R>().ok())
            .map(|boxed| *boxed)
    }
    
    /// 全リソースをクリア
    pub fn clear(&mut self) {
        self.resources.clear();
    }
    
    /// リソースの数を取得
    pub fn len(&self) -> usize {
        self.resources.len()
    }
    
    /// リソースが空かどうか
    pub fn is_empty(&self) -> bool {
        self.resources.is_empty()
    }
    
    /// 複数のリソースへの参照を同時に取得（2つのケース）
    pub fn get_multi<A: 'static + Resource, B: 'static + Resource>(&self) -> Option<(&A, &B)> {
        let a = self.get::<A>()?;
        let b = self.get::<B>()?;
        Some((a, b))
    }
    
    /// 複数のリソースへの可変参照を同時に取得（2つのケース）
    pub fn get_multi_mut<A: 'static + Resource, B: 'static + Resource>(&mut self) -> Option<(&mut A, &mut B)> {
        // 同じ型への可変参照を防ぐ
        if TypeId::of::<A>() == TypeId::of::<B>() {
            return None;
        }
        
        // 安全でない方法で複数の可変参照を取得
        // この実装は例示的なもので、実際には内部可変性や追加のチェックが必要
        unsafe {
            let a_ptr = self.get_mut::<A>()? as *mut A;
            let b_ptr = self.get_mut::<B>()? as *mut B;
            Some((&mut *a_ptr, &mut *b_ptr))
        }
    }
    
    /// 複数のリソースを読み取りモードでバッチ処理
    pub fn batch<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&ResourceBatch<dyn Resource>) -> R,
    {
        // 安全なリソースバッチを作成
        let empty_batch = ResourceBatch { resource: &() as &dyn Resource };
        f(&empty_batch)
    }
    
    /// 複数のリソースを書き込みモードでバッチ処理
    pub fn batch_mut<F, R>(&mut self, f: F) -> R
    where
        F: FnOnce(&mut ResourceBatchMut<dyn Resource>) -> R,
    {
        // 安全なリソースバッチを作成
        let mut empty_batch = ResourceBatchMut { resource: &mut () as &mut dyn Resource };
        f(&mut empty_batch)
    }
    
    /// 読み取り専用リソースバッチの取得
    pub fn fetch<R: Resource>(&self) -> ResourceBatch<R> {
        ResourceBatch {
            resource: self.get::<R>().expect("指定されたリソースが見つかりません"),
        }
    }
    
    /// 書き込み可能リソースバッチの取得
    pub fn fetch_mut<R: Resource>(&mut self) -> ResourceBatchMut<R> {
        ResourceBatchMut {
            resource: self.get_mut::<R>().expect("指定されたリソースが見つかりません"),
        }
    }
}

/// 読み取り専用リソースへのアクセスを提供する構造体
pub struct ResourceBatch<'a, R: Resource + ?Sized> {
    resource: &'a R,
}

impl<'a, R: Resource + ?Sized> ResourceBatch<'a, R> {
    pub fn resource(&self) -> &R {
        self.resource
    }
}

/// 可変リソースへのアクセスを提供する構造体
pub struct ResourceBatchMut<'a, R: Resource + ?Sized> {
    resource: &'a mut R,
}

impl<'a, R: Resource + ?Sized> ResourceBatchMut<'a, R> {
    pub fn resource(&self) -> &R {
        self.resource
    }

    pub fn resource_mut(&mut self) -> &mut R {
        self.resource
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, PartialEq)]
    struct TestResource {
        value: i32,
    }

    #[derive(Debug, PartialEq)]
    struct OtherResource {
        name: String,
    }

    #[test]
    fn test_insert_and_get() {
        let mut manager = ResourceManager::new();
        
        manager.insert(TestResource { value: 42 });
        let resource = manager.get::<TestResource>();
        
        assert!(resource.is_some());
        assert_eq!(resource.unwrap().value, 42);
    }

    #[test]
    fn test_get_mut() {
        let mut manager = ResourceManager::new();
        
        manager.insert(TestResource { value: 42 });
        
        // 参照を取得して変更
        if let Some(resource) = manager.get_mut::<TestResource>() {
            resource.value = 100;
        }
        
        // 変更が反映されていることを確認
        let resource = manager.get::<TestResource>().unwrap();
        assert_eq!(resource.value, 100);
    }

    #[test]
    fn test_remove() {
        let mut manager = ResourceManager::new();
        
        manager.insert(TestResource { value: 42 });
        assert!(manager.contains::<TestResource>());
        
        let removed = manager.remove::<TestResource>();
        assert!(removed.is_some());
        assert_eq!(removed.unwrap().value, 42);
        
        // 削除後は存在しない
        assert!(!manager.contains::<TestResource>());
    }

    #[test]
    fn test_multi_resource() {
        let mut manager = ResourceManager::new();
        
        manager.insert(TestResource { value: 42 });
        manager.insert(OtherResource { name: "Test".to_string() });
        
        // 複数リソースの参照を取得
        let (test, other) = manager.get_multi::<TestResource, OtherResource>().unwrap();
        assert_eq!(test.value, 42);
        assert_eq!(other.name, "Test");
        
        // 複数リソースの可変参照を取得
        let (test_mut, other_mut) = manager.get_multi_mut::<TestResource, OtherResource>().unwrap();
        test_mut.value = 100;
        other_mut.name = "Updated".to_string();
        
        // 変更が反映されていることを確認
        let (test, other) = manager.get_multi::<TestResource, OtherResource>().unwrap();
        assert_eq!(test.value, 100);
        assert_eq!(other.name, "Updated");
    }

    #[test]
    fn test_batch() {
        let mut manager = ResourceManager::new();
        
        manager.insert(TestResource { value: 42 });
        manager.insert(OtherResource { name: "Test".to_string() });
        
        // 読み取りバッチ
        let result = manager.batch(|batch| {
            let test = batch.read::<TestResource>().unwrap();
            let other = batch.read::<OtherResource>().unwrap();
            test.value + other.name.len() as i32
        });
        
        assert_eq!(result, 42 + 4);
        
        // 書き込みバッチ
        manager.batch_mut(|mut batch| {
            if let Some(test) = batch.write::<TestResource>() {
                test.value = 100;
            }
            if let Some(other) = batch.write::<OtherResource>() {
                other.name = "Updated".to_string();
            }
        });
        
        // 変更が反映されていることを確認
        assert_eq!(manager.get::<TestResource>().unwrap().value, 100);
        assert_eq!(manager.get::<OtherResource>().unwrap().name, "Updated");
    }
} 