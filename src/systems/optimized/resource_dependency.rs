/**
 * リソース依存関係システム
 * 
 * システムからリソースへのアクセスと依存関係を管理するためのトレイト
 */
use std::marker::PhantomData;
use std::fmt::{Debug, Formatter, Result as FmtResult};
use std::any::TypeId;

/// リソース依存関係を保持するマーカートレイト
pub trait ResourceDependency: Send + Sync + 'static + Debug {
    /// 依存するリソースのタイプIDを取得
    fn resource_type_ids(&self) -> Vec<TypeId>;
    
    /// 依存リソースの名前（デバッグ用）
    fn resource_type_names(&self) -> Vec<&'static str>;
}

/// 読み取り専用リソース依存
#[derive(Clone)]
pub struct ReadResource<T: 'static> {
    _marker: PhantomData<fn() -> T>,
}

impl<T: 'static> ReadResource<T> {
    /// 新しい読み取り専用リソース依存を作成
    pub fn new() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

impl<T: 'static> Default for ReadResource<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: 'static + std::fmt::Debug> Debug for ReadResource<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "ReadResource<{}>", std::any::type_name::<T>())
    }
}

impl<T: 'static + std::fmt::Debug> ResourceDependency for ReadResource<T> {
    fn resource_type_ids(&self) -> Vec<TypeId> {
        vec![TypeId::of::<T>()]
    }
    
    fn resource_type_names(&self) -> Vec<&'static str> {
        vec![std::any::type_name::<T>()]
    }
}

/// 書き込み可能リソース依存
#[derive(Clone)]
pub struct WriteResource<T: 'static> {
    _marker: PhantomData<fn() -> T>,
}

impl<T: 'static> WriteResource<T> {
    /// 新しい書き込み可能リソース依存を作成
    pub fn new() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

impl<T: 'static> Default for WriteResource<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: 'static + std::fmt::Debug> Debug for WriteResource<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "WriteResource<{}>", std::any::type_name::<T>())
    }
}

impl<T: 'static + std::fmt::Debug> ResourceDependency for WriteResource<T> {
    fn resource_type_ids(&self) -> Vec<TypeId> {
        vec![TypeId::of::<T>()]
    }
    
    fn resource_type_names(&self) -> Vec<&'static str> {
        vec![std::any::type_name::<T>()]
    }
}

/// 複数リソース依存（タプル実装）
#[derive(Clone, Debug)]
pub struct ResourceSet<A, B>(pub A, pub B);

impl<A: ResourceDependency, B: ResourceDependency> ResourceDependency for ResourceSet<A, B> {
    fn resource_type_ids(&self) -> Vec<TypeId> {
        let mut ids = self.0.resource_type_ids();
        ids.extend(self.1.resource_type_ids());
        ids
    }
    
    fn resource_type_names(&self) -> Vec<&'static str> {
        let mut names = self.0.resource_type_names();
        names.extend(self.1.resource_type_names());
        names
    }
}

/// 依存リソースなし
#[derive(Clone, Debug, Default)]
pub struct NoResources;

impl ResourceDependency for NoResources {
    fn resource_type_ids(&self) -> Vec<TypeId> {
        Vec::new()
    }
    
    fn resource_type_names(&self) -> Vec<&'static str> {
        Vec::new()
    }
} 