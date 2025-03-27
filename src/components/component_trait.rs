/**
 * コンポーネントトレイト定義
 * 
 * コンポーネントの標準インターフェースを定義します
 */
use std::any::{Any, TypeId};
use crate::entities::{EntityId, EntityManager};
use serde::{Serialize, Deserialize};

/// すべてのコンポーネントが実装すべき基本トレイト
pub trait Component: 'static + Send + Sync + Any + Clone + std::fmt::Debug {
    /// コンポーネントが初期化された時に呼ばれる
    fn on_init(&mut self, _entity_id: EntityId) {}
    
    /// コンポーネントが削除される前に呼ばれる
    fn on_remove(&mut self, _entity_id: EntityId) {}
    
    /// エンティティに追加された後に呼ばれる
    fn on_added(&mut self, _entity_id: EntityId) {}
    
    /// エンティティが持つ他のコンポーネントと相互作用が必要な場合に呼ばれる
    fn on_entity_ready(&mut self, _entity_id: EntityId, _entity_manager: &EntityManager) {}
    
    /// コンポーネントの一意の識別子を返す
    fn type_name(&self) -> &'static str {
        std::any::type_name::<Self>()
    }
    
    /// シリアライズ可能かどうかを返す
    fn is_serializable(&self) -> bool {
        false
    }
    
    /// このコンポーネントが依存する他のコンポーネントの型IDのリストを返す
    fn dependencies(&self) -> Vec<TypeId> {
        Vec::new()
    }
}

/// シリアライズ可能なコンポーネント用の拡張トレイト
pub trait SerializableComponent: Component + Serialize + for<'de> Deserialize<'de> {
    fn as_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }
    
    fn from_json(json: &str) -> Result<Self, serde_json::Error> 
    where
        Self: Sized,
    {
        serde_json::from_str(json)
    }
}

/// コンポーネントの依存関係を処理するトレイト（Object Safe版）
pub trait ComponentDependencyHandler: 'static + Send + Sync {
    /// 依存するコンポーネントがエンティティに追加されたときに呼ばれる
    fn on_dependency_added(&mut self, entity_id: EntityId, dependency_type_id: TypeId);
    
    /// 依存するコンポーネントがエンティティから削除されるときに呼ばれる
    fn on_dependency_removed(&mut self, entity_id: EntityId, dependency_type_id: TypeId);
}

/// 自動的にSerializableComponentを実装するマクロ
#[macro_export]
macro_rules! impl_serializable_component {
    ($type:ty) => {
        impl SerializableComponent for $type {}
        
        impl Component for $type {
            fn is_serializable(&self) -> bool {
                true
            }
        }
    };
} 