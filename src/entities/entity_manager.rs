/**
 * エンティティマネージャー
 * 
 * エンティティの作成、管理、削除を行うマネージャー
 */
use std::collections::{HashMap, HashSet};
use crate::entities::entity::{Entity, EntityId};

/// エンティティビルダー
/// エンティティを簡単に構築するためのビルダーパターン実装
pub struct EntityBuilder {
    /// 構築中のエンティティ
    entity: Entity,
}

impl EntityBuilder {
    /// 新しいビルダーを作成
    fn new(id: EntityId) -> Self {
        Self {
            entity: Entity::new(id),
        }
    }
    
    /// コンポーネントを追加
    pub fn with_component<T: 'static>(mut self, component: T) -> Self {
        self.entity.add_component(component);
        self
    }
    
    /// タグを追加
    pub fn with_tag(mut self, tag: &str) -> Self {
        self.entity.add_tag(tag);
        self
    }
    
    /// エンティティを構築
    pub fn build(self) -> Entity {
        self.entity
    }
}

/// エンティティマネージャー
/// ゲーム内の全エンティティを管理する
#[derive(Debug)]
pub struct EntityManager {
    /// エンティティの格納庫
    entities: HashMap<EntityId, Entity>,
    /// 次に割り当てるエンティティID
    next_id: u64,
    /// 削除待ちのエンティティID
    pending_removal: HashSet<EntityId>,
    /// タグごとのエンティティID
    tags_to_entities: HashMap<String, HashSet<EntityId>>,
}

impl Default for EntityManager {
    fn default() -> Self {
        Self {
            entities: HashMap::new(),
            next_id: 1, // 0は無効なIDとして使用
            pending_removal: HashSet::new(),
            tags_to_entities: HashMap::new(),
        }
    }
}

impl EntityManager {
    /// 新しいエンティティマネージャーを作成
    pub fn new() -> Self {
        Self::default()
    }
    
    /// 新しいエンティティを作成
    pub fn create_entity(&mut self) -> EntityId {
        let id = EntityId(self.next_id);
        self.next_id += 1;
        
        let entity = Entity::new(id);
        self.entities.insert(id, entity);
        
        id
    }
    
    /// エンティティビルダーを取得
    pub fn create_builder(&mut self) -> EntityBuilder {
        let id = EntityId(self.next_id);
        self.next_id += 1;
        
        EntityBuilder::new(id)
    }
    
    /// ビルダーで作成したエンティティを登録
    pub fn register_entity(&mut self, entity: Entity) -> EntityId {
        let id = entity.id;
        
        // タグ情報を更新
        for tag in entity.get_tags() {
            self.tags_to_entities
                .entry(tag.clone())
                .or_insert_with(HashSet::new)
                .insert(id);
        }
        
        self.entities.insert(id, entity);
        id
    }
    
    /// エンティティを削除
    pub fn remove_entity(&mut self, id: EntityId) {
        self.pending_removal.insert(id);
    }
    
    /// エンティティを取得
    pub fn get_entity(&self, id: EntityId) -> Option<&Entity> {
        self.entities.get(&id)
    }
    
    /// エンティティを取得（可変）
    pub fn get_entity_mut(&mut self, id: EntityId) -> Option<&mut Entity> {
        self.entities.get_mut(&id)
    }
    
    /// 全エンティティを取得
    pub fn get_all_entities(&self) -> impl Iterator<Item = &Entity> {
        self.entities.values()
    }
    
    /// 全エンティティIDを取得
    pub fn get_all_entity_ids(&self) -> impl Iterator<Item = EntityId> + '_ {
        self.entities.keys().copied()
    }
    
    /// 特定のコンポーネントを持つエンティティを全て取得
    pub fn get_entities_with_component<T: 'static>(&self) -> Vec<EntityId> {
        self.entities.iter()
            .filter(|(_, entity)| entity.has_component::<T>())
            .map(|(id, _)| *id)
            .collect()
    }
    
    /// 特定のタグを持つエンティティを全て取得
    pub fn get_entities_with_tag(&self, tag: &str) -> Vec<EntityId> {
        self.tags_to_entities
            .get(tag)
            .map(|set| set.iter().copied().collect())
            .unwrap_or_default()
    }
    
    /// 削除待ちのエンティティを本当に削除する
    pub fn flush_removals(&mut self) {
        for id in &self.pending_removal {
            if let Some(entity) = self.entities.remove(id) {
                // タグマップからも削除
                for tag in entity.get_tags() {
                    if let Some(tag_set) = self.tags_to_entities.get_mut(tag) {
                        tag_set.remove(id);
                        
                        // 空になったらHashSetごと削除
                        if tag_set.is_empty() {
                            self.tags_to_entities.remove(tag);
                        }
                    }
                }
            }
        }
        
        self.pending_removal.clear();
    }
    
    /// エンティティの数を取得
    pub fn entity_count(&self) -> usize {
        self.entities.len()
    }
    
    /// 全エンティティをクリア
    pub fn clear(&mut self) {
        self.entities.clear();
        self.pending_removal.clear();
        self.tags_to_entities.clear();
        // IDはリセットしない（一意性を保つため）
    }
} 