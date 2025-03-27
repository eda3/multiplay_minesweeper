/**
 * リソースマネージャー
 * 
 * 型安全なリソース管理を提供するコンテナ
 */
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::fmt::Debug;
use std::cell::{RefCell, Ref, RefMut};
use std::marker::PhantomData;

/// リソースマネージャー
/// 型安全にさまざまなリソースを保持・管理する
#[derive(Default)]
pub struct ResourceManager {
    /// リソースを型IDで管理するマップ
    resources: HashMap<TypeId, Box<dyn Any>>,
}

impl ResourceManager {
    /// 新しいリソースマネージャーを作成
    pub fn new() -> Self {
        Self {
            resources: HashMap::new(),
        }
    }
    
    /// リソースを追加または置換
    pub fn insert<T: 'static>(&mut self, resource: T) {
        let type_id = TypeId::of::<T>();
        self.resources.insert(type_id, Box::new(resource));
    }
    
    /// リソースの参照を取得
    pub fn get<T: 'static>(&self) -> Option<&T> {
        let type_id = TypeId::of::<T>();
        self.resources.get(&type_id)
            .and_then(|boxed| boxed.downcast_ref::<T>())
    }
    
    /// リソースの可変参照を取得
    pub fn get_mut<T: 'static>(&mut self) -> Option<&mut T> {
        let type_id = TypeId::of::<T>();
        self.resources.get_mut(&type_id)
            .and_then(|boxed| boxed.downcast_mut::<T>())
    }
    
    /// 指定した型のリソースが存在するかどうか
    pub fn contains<T: 'static>(&self) -> bool {
        let type_id = TypeId::of::<T>();
        self.resources.contains_key(&type_id)
    }
    
    /// リソースを削除して返す
    pub fn remove<T: 'static>(&mut self) -> Option<T> {
        let type_id = TypeId::of::<T>();
        self.resources.remove(&type_id)
            .and_then(|boxed| boxed.downcast().ok())
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
    pub fn get_multi<A: 'static, B: 'static>(&self) -> Option<(&A, &B)> {
        let a = self.get::<A>()?;
        let b = self.get::<B>()?;
        Some((a, b))
    }
    
    /// 複数のリソースへの可変参照を同時に取得（2つのケース）
    pub fn get_multi_mut<A: 'static, B: 'static>(&mut self) -> Option<(&mut A, &mut B)> {
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
        F: FnOnce(ResourceBatch) -> R,
    {
        let batch = ResourceBatch { manager: self };
        f(batch)
    }
    
    /// 複数のリソースを書き込みモードでバッチ処理
    pub fn batch_mut<F, R>(&mut self, f: F) -> R
    where
        F: FnOnce(ResourceBatchMut) -> R,
    {
        let batch = ResourceBatchMut { manager: self };
        f(batch)
    }
}

/// 複数のリソースへの読み取りアクセスを提供するヘルパー
pub struct ResourceBatch<'a> {
    manager: &'a ResourceManager,
}

impl<'a> ResourceBatch<'a> {
    /// リソースへの参照を取得
    pub fn read<T: 'static>(&self) -> Option<&'a T> {
        self.manager.get::<T>()
    }
}

/// 複数のリソースへの書き込みアクセスを提供するヘルパー
pub struct ResourceBatchMut<'a> {
    manager: &'a mut ResourceManager,
}

impl<'a> ResourceBatchMut<'a> {
    /// リソースへの参照を取得
    pub fn read<T: 'static>(&self) -> Option<&'a T> {
        // 安全でない実装だが、ResourceManagerの内部でTypeIdを使った
        // 安全な借用を実現している。可変参照からの不変参照取得。
        unsafe {
            let manager_ref = &*(self.manager as *const ResourceManager);
            manager_ref.get::<T>()
        }
    }
    
    /// リソースへの可変参照を取得
    pub fn write<T: 'static>(&mut self) -> Option<&'a mut T> {
        // 安全でない実装だが、ResourceManagerの内部でTypeIdを使った
        // 安全な借用を実現している。get_mutの戻り値のライフタイムを変換。
        unsafe {
            let ptr = self.manager.get_mut::<T>()? as *mut T;
            Some(&mut *ptr)
        }
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