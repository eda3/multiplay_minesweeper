/**
 * 基本的なエンティティの定義
 * 
 * ECSパターンのエンティティを表す型を定義します
 */
use std::collections::HashMap;
use std::any::{Any, TypeId};
use std::fmt;

/// エンティティID（ユニーク識別子）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EntityId(pub u64);

impl fmt::Display for EntityId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Entity({})", self.0)
    }
}

/// 基本的なエンティティ
/// 一意のIDとコンポーネントの集合です
#[derive(Debug)]
pub struct Entity {
    /// エンティティID
    pub id: EntityId,
    /// コンポーネントのマップ（TypeId -> Box<dyn Any>）
    components: HashMap<TypeId, Box<dyn Any>>,
    /// タグ（任意のラベル）
    tags: Vec<String>,
}

impl Entity {
    /// 新しいエンティティを作成
    pub fn new(id: EntityId) -> Self {
        Self {
            id,
            components: HashMap::new(),
            tags: Vec::new(),
        }
    }
    
    /// コンポーネントを追加
    pub fn add_component<T: 'static>(&mut self, component: T) -> &mut Self {
        let type_id = TypeId::of::<T>();
        self.components.insert(type_id, Box::new(component));
        self
    }
    
    /// コンポーネントを取得
    pub fn get_component<T: 'static>(&self) -> Option<&T> {
        let type_id = TypeId::of::<T>();
        self.components.get(&type_id)
            .and_then(|boxed| boxed.downcast_ref::<T>())
    }
    
    /// コンポーネントを取得（可変）
    pub fn get_component_mut<T: 'static>(&mut self) -> Option<&mut T> {
        let type_id = TypeId::of::<T>();
        self.components.get_mut(&type_id)
            .and_then(|boxed| boxed.downcast_mut::<T>())
    }
    
    /// コンポーネントを削除
    pub fn remove_component<T: 'static>(&mut self) -> Option<T> {
        let type_id = TypeId::of::<T>();
        self.components.remove(&type_id)
            .and_then(|boxed| boxed.downcast::<T>().ok())
            .map(|boxed| *boxed)
    }
    
    /// 指定したコンポーネントを持っているか確認
    pub fn has_component<T: 'static>(&self) -> bool {
        let type_id = TypeId::of::<T>();
        self.components.contains_key(&type_id)
    }
    
    /// タグを追加
    pub fn add_tag(&mut self, tag: &str) -> &mut Self {
        if !self.tags.contains(&tag.to_string()) {
            self.tags.push(tag.to_string());
        }
        self
    }
    
    /// タグを削除
    pub fn remove_tag(&mut self, tag: &str) {
        self.tags.retain(|t| t != tag);
    }
    
    /// タグを持っているか確認
    pub fn has_tag(&self, tag: &str) -> bool {
        self.tags.contains(&tag.to_string())
    }
    
    /// 全てのタグを取得
    pub fn get_tags(&self) -> &[String] {
        &self.tags
    }
} 