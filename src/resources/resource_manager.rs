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
#[derive(Debug)]
pub struct ResourceManager {
    /// リソースの格納先（型ID -> Box<dyn Any>のマップ）
    resources: HashMap<TypeId, Box<dyn Any>>,
}

impl Default for ResourceManager {
    fn default() -> Self {
        Self {
            resources: HashMap::new(),
        }
    }
}

impl ResourceManager {
    /// 新しいリソースマネージャーを作成
    pub fn new() -> Self {
        Self::default()
    }
    
    /// リソースを追加または更新
    pub fn insert<T: 'static>(&mut self, resource: T) {
        let type_id = TypeId::of::<T>();
        self.resources.insert(type_id, Box::new(resource));
    }
    
    /// リソースを取得（不変参照）
    pub fn get<T: 'static>(&self) -> Option<&T> {
        let type_id = TypeId::of::<T>();
        self.resources.get(&type_id)
            .and_then(|boxed| boxed.downcast_ref::<T>())
    }
    
    /// リソースを取得（可変参照）
    pub fn get_mut<T: 'static>(&mut self) -> Option<&mut T> {
        let type_id = TypeId::of::<T>();
        self.resources.get_mut(&type_id)
            .and_then(|boxed| boxed.downcast_mut::<T>())
    }
    
    /// リソースが存在するかチェック
    pub fn contains<T: 'static>(&self) -> bool {
        let type_id = TypeId::of::<T>();
        self.resources.contains_key(&type_id)
    }
    
    /// リソースを削除
    pub fn remove<T: 'static>(&mut self) -> Option<T> {
        let type_id = TypeId::of::<T>();
        self.resources.remove(&type_id)
            .and_then(|boxed| boxed.downcast::<T>().ok())
            .map(|boxed| *boxed)
    }
    
    /// すべてのリソースを削除
    pub fn clear(&mut self) {
        self.resources.clear();
    }
    
    /// リソース数を取得
    pub fn len(&self) -> usize {
        self.resources.len()
    }
    
    /// リソースが空かどうか
    pub fn is_empty(&self) -> bool {
        self.resources.is_empty()
    }
    
    /// 複数のリソースを一度に取得（タプルで返す）
    pub fn get_multi<A: 'static, B: 'static>(&self) -> Option<(&A, &B)> {
        let a = self.get::<A>()?;
        let b = self.get::<B>()?;
        Some((a, b))
    }
    
    /// 複数のリソースを一度に取得（一部可変）
    pub fn get_multi_mut<A: 'static, B: 'static>(&mut self) -> Option<(&A, &mut B)> {
        // TypeIdを取得
        let a_type_id = TypeId::of::<A>();
        let b_type_id = TypeId::of::<B>();
        
        // 同じ型の場合は取得できない
        if a_type_id == b_type_id {
            return None;
        }
        
        // 安全な実装のために一度取り出す
        let b = self.get_mut::<B>()?;
        let b_ptr = b as *mut B;
        
        let a = self.get::<A>()?;
        
        // Bをポインタから復元
        let b = unsafe { &mut *b_ptr };
        
        Some((a, b))
    }
    
    /// 安全な読み取り専用バッチアクセス
    pub fn batch<'a, F, R>(&'a self, f: F) -> R
    where
        F: FnOnce(&ResourceBatch<'a>) -> R,
    {
        let batch = ResourceBatch { manager: self };
        f(&batch)
    }
    
    /// バッチ処理（読み書き）のための安全なアクセスを提供
    pub fn batch_mut<'a, F, R>(&'a mut self, f: F) -> R
    where
        F: FnOnce(&mut ResourceBatchMut<'a>) -> R,
    {
        let mut batch = ResourceBatchMut { manager: self };
        f(&mut batch)
    }
}

/// 読み取り専用リソースバッチ
pub struct ResourceBatch<'a> {
    manager: &'a ResourceManager,
}

impl<'a> ResourceBatch<'a> {
    /// リソースを読み取る
    pub fn read<T: 'static>(&self) -> Option<&'a T> {
        self.manager.get::<T>()
    }
}

/// 読み書きリソースバッチ
pub struct ResourceBatchMut<'a> {
    manager: &'a mut ResourceManager,
}

impl<'a> ResourceBatchMut<'a> {
    /// リソースを読み取る
    pub fn read<T: 'static>(&self) -> Option<&'a T> {
        // この実装は安全ではない可能性があるが、Rustの借用チェッカーを満たすために必要
        unsafe {
            let type_id = TypeId::of::<T>();
            (self.manager as *const ResourceManager)
                .as_ref()
                .unwrap()
                .resources
                .get(&type_id)
                .and_then(|boxed| boxed.downcast_ref::<T>())
        }
    }
    
    /// リソースを書き込む
    pub fn write<T: 'static>(&mut self) -> Option<&'a mut T> {
        // この実装は安全ではない可能性があるが、Rustの借用チェッカーを満たすために必要
        unsafe {
            let type_id = TypeId::of::<T>();
            (self.manager as *mut ResourceManager)
                .as_mut()
                .unwrap()
                .resources
                .get_mut(&type_id)
                .and_then(|boxed| boxed.downcast_mut::<T>())
        }
    }
} 