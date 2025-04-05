/**
 * コンポーネントトレイト
 * 
 * ECSパターンのコンポーネントを定義する基本トレイト
 */
use std::any::{Any, TypeId};
use std::fmt::Debug;
use crate::entities::EntityId;
use serde::{Serialize, Deserialize};

/// コンポーネントの基本インターフェース
/// 
/// 全てのコンポーネントはこのトレイトを実装する必要があります。
/// 型消去とダウンキャストを可能にするため、Any トレイトも実装して
/// Box化されたコンポーネントを保存・取得できるようにしています。
pub trait Component: Debug + 'static {
    /// Any型のイミュータブル参照にキャスト
    fn as_any(&self) -> &dyn Any;
    
    /// Any型のミュータブル参照にキャスト
    fn as_any_mut(&mut self) -> &mut dyn Any;
    
    /// このコンポーネントのクローンを作成し、Box化して返す
    fn clone_box(&self) -> Box<dyn Component>;
    
    /// エンティティに追加された時に呼ばれる
    fn on_init(&mut self, _entity_id: EntityId) {}
    
    /// エンティティから取り除かれる時に呼ばれる
    fn on_remove(&mut self, _entity_id: EntityId) {}
    
    /// エンティティが削除される時に呼ばれる
    fn on_entity_destroy(&mut self, _entity_id: EntityId) {}
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