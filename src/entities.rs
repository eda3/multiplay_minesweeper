/**
 * エンティティマネージャー
 * 
 * エンティティとコンポーネントを管理するモジュール
 */
use std::any::{Any, TypeId};
use std::collections::{HashMap, HashSet};
use std::marker::PhantomData;

/// エンティティマネージャー
/// ECSのエンティティ管理とコンポーネントストレージを担当
#[derive(Debug)]
pub struct EntityManager {
    /// 次に割り当てるエンティティID
    next_entity_id: usize,
    /// 削除されたエンティティIDの再利用プール
    deleted_entity_ids: Vec<usize>,
    /// アクティブエンティティのセット
    active_entities: HashSet<usize>,
    /// コンポーネントストレージ (TypeId => (EntityId => Component))
    components: HashMap<TypeId, Box<dyn ComponentStorage>>,
}

impl EntityManager {
    /// 新しいエンティティマネージャーを作成
    pub fn new() -> Self {
        Self {
            next_entity_id: 0,
            deleted_entity_ids: Vec::new(),
            active_entities: HashSet::new(),
            components: HashMap::new(),
        }
    }
    
    /// 新しいエンティティを作成
    pub fn create_entity(&mut self) -> usize {
        // 再利用可能なIDがあればそれを使う
        let entity_id = if let Some(id) = self.deleted_entity_ids.pop() {
            id
        } else {
            let id = self.next_entity_id;
            self.next_entity_id += 1;
            id
        };
        
        // アクティブエンティティに追加
        self.active_entities.insert(entity_id);
        
        entity_id
    }
    
    /// エンティティをコンポーネントから作成
    pub fn create_entity_with_components<C: 'static + Clone>(&mut self, components: Vec<C>) -> usize {
        let entity_id = self.create_entity();
        
        for component in components {
            self.add_component(entity_id, component);
        }
        
        entity_id
    }
    
    /// エンティティを削除
    pub fn remove_entity(&mut self, entity_id: usize) -> bool {
        if self.active_entities.remove(&entity_id) {
            // 削除されたIDを再利用プールに追加
            self.deleted_entity_ids.push(entity_id);
            
            // すべてのコンポーネントストレージからエンティティを削除
            for storage in self.components.values_mut() {
                storage.remove(entity_id);
            }
            
            true
        } else {
            false
        }
    }
    
    /// エンティティが存在するか確認
    pub fn is_entity_active(&self, entity_id: usize) -> bool {
        self.active_entities.contains(&entity_id)
    }
    
    /// コンポーネントを追加
    pub fn add_component<T: 'static + Clone>(&mut self, entity_id: usize, component: T) {
        if !self.active_entities.contains(&entity_id) {
            return;
        }
        
        let type_id = TypeId::of::<T>();
        
        // そのタイプのコンポーネントストレージがなければ作成
        if !self.components.contains_key(&type_id) {
            self.components.insert(type_id, Box::new(ComponentVec::<T>::new()));
        }
        
        // コンポーネントをストレージに追加
        if let Some(storage) = self.components.get_mut(&type_id) {
            if let Some(typed_storage) = storage.as_any_mut().downcast_mut::<ComponentVec<T>>() {
                typed_storage.insert(entity_id, component);
            }
        }
    }
    
    /// コンポーネントを取得
    pub fn get_component<T: 'static + Clone>(&self, entity_id: usize) -> Option<T> {
        if !self.active_entities.contains(&entity_id) {
            return None;
        }
        
        let type_id = TypeId::of::<T>();
        
        if let Some(storage) = self.components.get(&type_id) {
            if let Some(typed_storage) = storage.as_any().downcast_ref::<ComponentVec<T>>() {
                return typed_storage.get(entity_id).cloned();
            }
        }
        
        None
    }
    
    /// コンポーネントを取得（可変参照）
    pub fn get_component_mut<T: 'static + Clone>(&mut self, entity_id: usize) -> Option<&mut T> {
        if !self.active_entities.contains(&entity_id) {
            return None;
        }
        
        let type_id = TypeId::of::<T>();
        
        if let Some(storage) = self.components.get_mut(&type_id) {
            if let Some(typed_storage) = storage.as_any_mut().downcast_mut::<ComponentVec<T>>() {
                return typed_storage.get_mut(entity_id);
            }
        }
        
        None
    }
    
    /// コンポーネントを削除
    pub fn remove_component<T: 'static>(&mut self, entity_id: usize) -> bool {
        if !self.active_entities.contains(&entity_id) {
            return false;
        }
        
        let type_id = TypeId::of::<T>();
        
        if let Some(storage) = self.components.get_mut(&type_id) {
            storage.remove(entity_id);
            return true;
        }
        
        false
    }
    
    /// 特定のコンポーネントを持つエンティティを検索
    pub fn find_entities_with_component<T: 'static>(&self) -> Vec<usize> {
        let type_id = TypeId::of::<T>();
        
        if let Some(storage) = self.components.get(&type_id) {
            if let Some(typed_storage) = storage.as_any().downcast_ref::<ComponentVec<T>>() {
                return typed_storage.get_entities();
            }
        }
        
        Vec::new()
    }
    
    /// 複数のコンポーネントを持つエンティティを検索
    pub fn find_entities_with_components<T: ComponentQuery>(&self) -> Vec<usize> {
        T::find_entities(self)
    }
    
    /// すべてのアクティブエンティティを取得
    pub fn get_all_entities(&self) -> Vec<usize> {
        self.active_entities.iter().cloned().collect()
    }
}

/// コンポーネントストレージのトレイト
trait ComponentStorage: Any {
    /// Anyトレイトへのダウンキャスト
    fn as_any(&self) -> &dyn Any;
    /// Anyトレイトへのダウンキャスト（可変）
    fn as_any_mut(&mut self) -> &mut dyn Any;
    /// コンポーネントを削除
    fn remove(&mut self, entity_id: usize);
}

/// コンポーネントをストアするベクター実装
#[derive(Debug)]
struct ComponentVec<T: 'static + Clone> {
    /// コンポーネントのマップ
    components: HashMap<usize, T>,
    /// 型情報（コンパイル時に消去される）
    _marker: PhantomData<T>,
}

impl<T: 'static + Clone> ComponentVec<T> {
    /// 新しいコンポーネントベクターを作成
    fn new() -> Self {
        Self {
            components: HashMap::new(),
            _marker: PhantomData,
        }
    }
    
    /// コンポーネントを挿入
    fn insert(&mut self, entity_id: usize, component: T) {
        self.components.insert(entity_id, component);
    }
    
    /// コンポーネントを取得
    fn get(&self, entity_id: usize) -> Option<&T> {
        self.components.get(&entity_id)
    }
    
    /// コンポーネントを取得（可変）
    fn get_mut(&mut self, entity_id: usize) -> Option<&mut T> {
        self.components.get_mut(&entity_id)
    }
    
    /// このコンポーネントを持つエンティティIDを取得
    fn get_entities(&self) -> Vec<usize> {
        self.components.keys().cloned().collect()
    }
}

impl<T: 'static + Clone> ComponentStorage for ComponentVec<T> {
    fn as_any(&self) -> &dyn Any {
        self
    }
    
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
    
    fn remove(&mut self, entity_id: usize) {
        self.components.remove(&entity_id);
    }
}

/// コンポーネントクエリトレイト
/// find_entities_with_components のジェネリック実装に使用
pub trait ComponentQuery {
    /// 指定されたコンポーネントを持つエンティティを検索
    fn find_entities(manager: &EntityManager) -> Vec<usize>;
}

/// 単一コンポーネントのクエリ実装
impl<T: 'static> ComponentQuery for T {
    fn find_entities(manager: &EntityManager) -> Vec<usize> {
        manager.find_entities_with_component::<T>()
    }
}

/// 2つのコンポーネントのクエリ実装
impl<T1: 'static, T2: 'static> ComponentQuery for (T1, T2) {
    fn find_entities(manager: &EntityManager) -> Vec<usize> {
        let entities1 = manager.find_entities_with_component::<T1>();
        let entities2 = manager.find_entities_with_component::<T2>();
        
        // 両方のコンポーネントを持つエンティティを見つける
        entities1.into_iter()
            .filter(|e| entities2.contains(e))
            .collect()
    }
}

/// 3つのコンポーネントのクエリ実装
impl<T1: 'static, T2: 'static, T3: 'static> ComponentQuery for (T1, T2, T3) {
    fn find_entities(manager: &EntityManager) -> Vec<usize> {
        let entities1 = manager.find_entities_with_component::<T1>();
        let entities2 = manager.find_entities_with_component::<T2>();
        let entities3 = manager.find_entities_with_component::<T3>();
        
        // 3つ全てのコンポーネントを持つエンティティを見つける
        entities1.into_iter()
            .filter(|e| entities2.contains(e) && entities3.contains(e))
            .collect()
    }
} 