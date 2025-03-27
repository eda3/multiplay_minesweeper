/**
 * エンティティマネージャー
 * 
 * エンティティの作成、管理、削除を行うマネージャー
 */
use std::collections::{HashMap, HashSet};
use std::any::TypeId;
use crate::entities::entity::{Entity, EntityId};
use crate::entities::entity_id_generator::EntityIdGenerator;
use crate::components::{Component, ComponentDependencyHandler, ComponentFactory};

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

/// 親子関係を表すコンポーネント
#[derive(Debug, Clone)]
pub struct Hierarchy {
    /// 親エンティティID（存在する場合）
    pub parent: Option<EntityId>,
    /// 子エンティティIDのリスト
    pub children: Vec<EntityId>,
}

impl Hierarchy {
    /// 新しい階層コンポーネントを作成
    pub fn new() -> Self {
        Self {
            parent: None,
            children: Vec::new(),
        }
    }
    
    /// 子エンティティを追加
    pub fn add_child(&mut self, child_id: EntityId) {
        if !self.children.contains(&child_id) {
            self.children.push(child_id);
        }
    }
    
    /// 子エンティティを削除
    pub fn remove_child(&mut self, child_id: EntityId) {
        self.children.retain(|id| *id != child_id);
    }
}

/// エンティティマネージャー
/// ゲーム内の全エンティティを管理する
#[derive(Debug)]
pub struct EntityManager {
    /// エンティティの格納庫
    entities: HashMap<EntityId, Entity>,
    /// エンティティIDジェネレーター
    id_generator: EntityIdGenerator,
    /// 削除待ちのエンティティID
    pending_removal: HashSet<EntityId>,
    /// タグごとのエンティティID
    tags_to_entities: HashMap<String, HashSet<EntityId>>,
    /// コンポーネントタイプごとのエンティティID
    component_indices: HashMap<TypeId, HashSet<EntityId>>,
    /// コンポーネントファクトリー
    component_factory: Option<ComponentFactory>,
}

impl Default for EntityManager {
    fn default() -> Self {
        Self {
            entities: HashMap::new(),
            id_generator: EntityIdGenerator::default(),
            pending_removal: HashSet::new(),
            tags_to_entities: HashMap::new(),
            component_indices: HashMap::new(),
            component_factory: Some(ComponentFactory::new()),
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
        let id = self.id_generator.generate();
        
        let entity = Entity::new(id);
        self.entities.insert(id, entity);
        
        id
    }
    
    /// バッチでエンティティを作成
    pub fn create_entities(&mut self, count: usize) -> Vec<EntityId> {
        let mut ids = Vec::with_capacity(count);
        for _ in 0..count {
            ids.push(self.create_entity());
        }
        ids
    }
    
    /// エンティティビルダーを取得
    pub fn create_builder(&mut self) -> EntityBuilder {
        let id = self.id_generator.generate();
        
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
        
        // コンポーネントタイプのインデックスを更新（将来の高速クエリ用）
        self.update_component_indices(&entity);
        
        self.entities.insert(id, entity);
        id
    }
    
    /// エンティティを削除
    pub fn remove_entity(&mut self, id: EntityId) {
        self.pending_removal.insert(id);
    }
    
    /// バッチでエンティティを削除
    pub fn remove_entities<I: IntoIterator<Item = EntityId>>(&mut self, ids: I) {
        for id in ids {
            self.pending_removal.insert(id);
        }
    }
    
    /// 即時エンティティを削除（待機なし）
    pub fn remove_entity_immediate(&mut self, id: EntityId) -> Option<Entity> {
        let entity = self.entities.remove(&id)?;
        
        // タグマップから削除
        for tag in entity.get_tags() {
            if let Some(tag_set) = self.tags_to_entities.get_mut(tag) {
                tag_set.remove(&id);
                
                if tag_set.is_empty() {
                    self.tags_to_entities.remove(tag);
                }
            }
        }
        
        // コンポーネントインデックスから削除
        for indices in self.component_indices.values_mut() {
            indices.remove(&id);
        }
        
        // IDをリサイクル
        self.id_generator.recycle(id);
        
        Some(entity)
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
        let type_id = TypeId::of::<T>();
        
        // インデックスが構築済みの場合はそれを使用
        if let Some(indices) = self.component_indices.get(&type_id) {
            return indices.iter().copied().collect();
        }
        
        // 未構築の場合はフルスキャン
        self.entities.iter()
            .filter(|(_, entity)| entity.has_component::<T>())
            .map(|(id, _)| *id)
            .collect()
    }
    
    /// コンポーネントタイプのインデックスを構築
    pub fn build_component_index<T: 'static>(&mut self) -> HashSet<EntityId> {
        let type_id = TypeId::of::<T>();
        
        let ids: HashSet<EntityId> = self.entities.iter()
            .filter_map(|(id, entity)| {
                if entity.has_component::<T>() {
                    Some(*id)
                } else {
                    None
                }
            })
            .collect();
            
        self.component_indices.insert(type_id, ids.clone());
        ids
    }
    
    /// 特定のタグを持つエンティティを全て取得
    pub fn get_entities_with_tag(&self, tag: &str) -> Vec<EntityId> {
        self.tags_to_entities
            .get(tag)
            .map(|set| set.iter().copied().collect())
            .unwrap_or_default()
    }
    
    /// 複数条件によるクエリ：指定したコンポーネントとタグを持つエンティティを取得
    pub fn query_with_component_and_tag<T: 'static>(&self, tag: &str) -> Vec<EntityId> {
        // タグによるフィルタ
        let tag_entities = match self.tags_to_entities.get(tag) {
            Some(entities) => entities,
            None => return Vec::new(),
        };
        
        // コンポーネントによるフィルタ
        self.entities.iter()
            .filter_map(|(id, entity)| {
                if tag_entities.contains(id) && entity.has_component::<T>() {
                    Some(*id)
                } else {
                    None
                }
            })
            .collect()
    }
    
    /// 親子関係を設定
    pub fn set_parent(&mut self, child: EntityId, parent: EntityId) -> Result<(), &'static str> {
        if !self.entities.contains_key(&child) || !self.entities.contains_key(&parent) {
            return Err("エンティティが存在しません");
        }
        
        // 子エンティティの更新
        if let Some(child_entity) = self.get_entity_mut(child) {
            // 子エンティティの階層コンポーネントを取得または作成
            if !child_entity.has_component::<Hierarchy>() {
                child_entity.add_component(Hierarchy::new());
            }
            
            // 過去の親情報を一時保存
            let old_parent = {
                if let Some(hierarchy) = child_entity.get_component::<Hierarchy>() {
                    hierarchy.parent
                } else {
                    None
                }
            };
            
            // 新しい親情報を設定
            if let Some(hierarchy) = child_entity.get_component_mut::<Hierarchy>() {
                hierarchy.parent = Some(parent);
            }
            
            // 古い親から削除
            if let Some(old_parent_id) = old_parent {
                if let Some(old_parent_entity) = self.get_entity_mut(old_parent_id) {
                    if let Some(parent_hierarchy) = old_parent_entity.get_component_mut::<Hierarchy>() {
                        parent_hierarchy.remove_child(child);
                    }
                }
            }
        }
        
        // 親エンティティの更新
        if let Some(parent_entity) = self.get_entity_mut(parent) {
            // 親エンティティの階層コンポーネントを取得または作成
            if !parent_entity.has_component::<Hierarchy>() {
                parent_entity.add_component(Hierarchy::new());
            }
            
            // 子を追加
            if let Some(parent_hierarchy) = parent_entity.get_component_mut::<Hierarchy>() {
                parent_hierarchy.add_child(child);
            }
        }
        
        Ok(())
    }
    
    /// 再帰的にエンティティを削除（親が削除されたら子も削除）
    pub fn remove_entity_recursive(&mut self, id: EntityId) {
        let children = {
            if let Some(entity) = self.get_entity(id) {
                if let Some(hierarchy) = entity.get_component::<Hierarchy>() {
                    hierarchy.children.clone()
                } else {
                    Vec::new()
                }
            } else {
                Vec::new()
            }
        };
        
        // 子エンティティを再帰的に削除
        for child in children {
            self.remove_entity_recursive(child);
        }
        
        // 自身を削除
        self.remove_entity(id);
    }
    
    /// 削除待ちのエンティティを本当に削除する
    pub fn flush_removals(&mut self) {
        // 削除待ちリストのコピーを作成（削除中に変更を避けるため）
        let to_remove: Vec<EntityId> = self.pending_removal.iter().copied().collect();
        
        for id in to_remove {
            self.remove_entity_immediate(id);
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
        self.component_indices.clear();
        // IDジェネレーターはリセットしない（一意性を保つため）
    }
    
    /// コンポーネントインデックスを更新
    fn update_component_indices(&mut self, entity: &Entity) {
        // 全てのコンポーネントタイプに対してインデックスを更新
        for type_id in entity.get_component_types() {
            let entities = self.component_indices
                .entry(type_id)
                .or_insert_with(HashSet::new);
                
            entities.insert(entity.id);
        }
    }
    
    /// コンポーネントファクトリーを設定
    pub fn set_component_factory(&mut self, factory: ComponentFactory) {
        self.component_factory = Some(factory);
    }
    
    /// コンポーネントファクトリーを取得
    pub fn get_component_factory(&self) -> Option<&ComponentFactory> {
        self.component_factory.as_ref()
    }
    
    /// コンポーネントファクトリーを取得（可変）
    pub fn get_component_factory_mut(&mut self) -> Option<&mut ComponentFactory> {
        self.component_factory.as_mut()
    }
    
    /// エンティティにコンポーネントを追加
    pub fn add_component<T: Component>(&mut self, entity_id: EntityId, mut component: T) -> Result<(), &'static str> {
        // 事前にエンティティの存在確認
        if !self.entities.contains_key(&entity_id) {
            return Err("エンティティが存在しません");
        }
        
        // コンポーネントの初期化
        component.on_init(entity_id);
        
        // 依存関係を取得
        let dependencies = component.dependencies();
        
        // 依存関係の確認とファクトリー情報の取得（事前に行い、後で可変参照を取る前に済ませる）
        let missing_dependencies = dependencies.iter()
            .filter(|&&type_id| !self.entity_has_component_by_type_id(entity_id, type_id))
            .collect::<Vec<_>>();
            
        // ファクトリー情報をコピー（所有権の問題を回避）
        let has_factory = self.component_factory.is_some();
        
        // 依存関係の処理
        if !missing_dependencies.is_empty() {
            if !has_factory {
                return Err("コンポーネントファクトリーが設定されていません");
            }
            
            for &type_id in &missing_dependencies {
                // ここでファクトリーを使用して依存コンポーネントを作成
                let factory = self.component_factory.as_ref().unwrap();
                if !factory.is_registered(*type_id) {
                    return Err("依存コンポーネントがファクトリーに登録されていません");
                }
                
                // コンポーネントを作成
                let result = factory.create_default(*type_id);
                match result {
                    Ok(component) => {
                        // エンティティを取得して依存コンポーネントを追加
                        if let Some(entity) = self.entities.get_mut(&entity_id) {
                            entity.add_component_boxed(*type_id, component);
                        } else {
                            return Err("エンティティが見つかりません");
                        }
                    },
                    Err(_) => return Err("デフォルトコンポーネントの作成に失敗しました"),
                }
            }
        }
        
        // メインのコンポーネントを追加
        if let Some(entity) = self.entities.get_mut(&entity_id) {
            entity.add_component(component);
        } else {
            return Err("エンティティが見つかりません");
        }
        
        // インデックスを更新
        self.update_component_index::<T>(entity_id);
        
        // コンポーネント追加イベントを呼び出し
        if let Some(entity) = self.entities.get_mut(&entity_id) {
            if let Some(comp) = entity.get_component_mut::<T>() {
                comp.on_added(entity_id);
            }
        }
        
        Ok(())
    }
    
    /// 型IDによるコンポーネント所持確認
    fn entity_has_component_by_type_id(&self, entity_id: EntityId, type_id: TypeId) -> bool {
        if let Some(entity) = self.get_entity(entity_id) {
            for comp_type_id in entity.get_component_types() {
                if comp_type_id == type_id {
                    return true;
                }
            }
        }
        false
    }
    
    /// 特定タイプのコンポーネントに対するインデックスを更新
    fn update_component_index<T: 'static>(&mut self, entity_id: EntityId) {
        let type_id = TypeId::of::<T>();
        
        // このタイプのコンポーネントを持つエンティティのセットを取得または作成
        let entities = self.component_indices
            .entry(type_id)
            .or_insert_with(HashSet::new);
            
        // エンティティIDを追加
        entities.insert(entity_id);
    }
    
    /// エンティティから特定タイプのコンポーネントを取得
    pub fn get_component<T: 'static>(&self, entity_id: EntityId) -> Option<&T> {
        if let Some(entity) = self.entities.get(&entity_id) {
            entity.get_component::<T>()
        } else {
            None
        }
    }
    
    /// エンティティから特定タイプのコンポーネントを可変参照で取得
    pub fn get_component_mut<T: 'static>(&mut self, entity_id: EntityId) -> Option<&mut T> {
        if let Some(entity) = self.entities.get_mut(&entity_id) {
            entity.get_component_mut::<T>()
        } else {
            None
        }
    }
    
    /// エンティティが特定タイプのコンポーネントを持っているか確認
    pub fn has_component<T: 'static>(&self, entity_id: EntityId) -> bool {
        if let Some(entity) = self.entities.get(&entity_id) {
            entity.has_component::<T>()
        } else {
            false
        }
    }
    
    /// エンティティから特定タイプのコンポーネントを削除
    pub fn remove_component<T: 'static>(&mut self, entity_id: EntityId) -> Option<T> {
        let result = if let Some(entity) = self.entities.get_mut(&entity_id) {
            entity.remove_component::<T>()
        } else {
            None
        };
        
        // コンポーネントインデックスを更新
        if result.is_some() {
            let type_id = TypeId::of::<T>();
            if let Some(entities) = self.component_indices.get_mut(&type_id) {
                entities.remove(&entity_id);
            }
        }
        
        result
    }
    
    /// 特定のコンポーネントを持つエンティティをすべて検索
    pub fn find_entities_with_component<T: 'static>(&self) -> Vec<EntityId> {
        // get_entities_with_component メソッドを呼び出して結果を返す
        self.get_entities_with_component::<T>()
    }
}