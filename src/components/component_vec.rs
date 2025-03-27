/**
 * 最適化されたコンポーネントコレクション
 * 
 * パフォーマンス向上のための同種コンポーネント格納コンテナ
 */
use std::collections::HashMap;
use crate::entities::EntityId;
use crate::components::component_trait::Component;

/// 同じタイプのコンポーネントをまとめて格納するコレクション
pub struct ComponentVec<T: Component> {
    components: Vec<Option<T>>,
    entity_indices: HashMap<EntityId, usize>,
}

impl<T: Component> ComponentVec<T> {
    /// 新しいコンポーネントコレクションを作成
    pub fn new() -> Self {
        Self {
            components: Vec::new(),
            entity_indices: HashMap::new(),
        }
    }
    
    /// 指定容量で初期化
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            components: Vec::with_capacity(capacity),
            entity_indices: HashMap::with_capacity(capacity),
        }
    }
    
    /// コンポーネントを挿入
    pub fn insert(&mut self, entity_id: EntityId, component: T) {
        if let Some(&index) = self.entity_indices.get(&entity_id) {
            // 既存の位置を更新
            self.components[index] = Some(component);
        } else {
            // 新しい位置に追加
            let index = self.components.len();
            self.components.push(Some(component));
            self.entity_indices.insert(entity_id, index);
        }
    }
    
    /// コンポーネントを削除
    pub fn remove(&mut self, entity_id: EntityId) -> Option<T> {
        if let Some(&index) = self.entity_indices.get(&entity_id) {
            let component = self.components[index].take();
            self.entity_indices.remove(&entity_id);
            component
        } else {
            None
        }
    }
    
    /// コンポーネントを取得（参照）
    pub fn get(&self, entity_id: EntityId) -> Option<&T> {
        self.entity_indices.get(&entity_id)
            .and_then(|&index| self.components.get(index).and_then(|c| c.as_ref()))
    }
    
    /// コンポーネントを取得（可変参照）
    pub fn get_mut(&mut self, entity_id: EntityId) -> Option<&mut T> {
        if let Some(&index) = self.entity_indices.get(&entity_id) {
            self.components.get_mut(index).and_then(|c: &mut Option<T>| c.as_mut())
        } else {
            None
        }
    }
    
    /// すべてのコンポーネントをイテレート
    pub fn iter(&self) -> impl Iterator<Item = (EntityId, &T)> {
        self.entity_indices.iter()
            .filter_map(|(entity_id, &index)| {
                self.components.get(index)
                    .and_then(|opt_comp| opt_comp.as_ref().map(|comp| (*entity_id, comp)))
            })
    }
    
    /// すべてのコンポーネントを可変イテレート
    pub fn iter_mut(&mut self) -> Vec<(EntityId, &mut T)> {
        // Rustのライフタイムの制約により、通常のイテレータを返すのは難しいため
        // ここでは中間的なVecを使用します
        let mut result = Vec::with_capacity(self.entity_indices.len());
        let mut indices: Vec<(EntityId, usize)> = self.entity_indices.iter()
            .map(|(&e, &i)| (e, i))
            .collect();
            
        for (entity_id, index) in indices {
            if let Some(Some(comp)) = self.components.get_mut(index) {
                // 安全でない部分: 複数の可変参照を作成します
                // これはRustの借用チェッカーを回避するための非推奨パターンで、
                // 実際の実装では別のアプローチを検討すべきです
                let comp_ref = unsafe { &mut *(comp as *mut T) };
                result.push((entity_id, comp_ref));
            }
        }
        
        result
    }
    
    /// エンティティがコンポーネントを持っているか確認
    pub fn contains(&self, entity_id: EntityId) -> bool {
        self.entity_indices.contains_key(&entity_id)
    }
    
    /// コンポーネントの数を取得
    pub fn len(&self) -> usize {
        self.entity_indices.len()
    }
    
    /// コレクションが空かどうか確認
    pub fn is_empty(&self) -> bool {
        self.entity_indices.is_empty()
    }
    
    /// 内部状態を最適化（断片化を解消）
    pub fn optimize(&mut self) {
        // 使用中の要素だけを格納した新しいベクターを作成
        let mut new_components = Vec::with_capacity(self.entity_indices.len());
        let mut new_indices = HashMap::with_capacity(self.entity_indices.len());
        
        // 有効なコンポーネントだけを新しいベクターに移動
        for (entity_id, &old_index) in &self.entity_indices {
            if let Some(component) = self.components[old_index].take() {
                let new_index = new_components.len();
                new_components.push(Some(component));
                new_indices.insert(*entity_id, new_index);
            }
        }
        
        // 古いデータを新しいデータで置き換え
        self.components = new_components;
        self.entity_indices = new_indices;
    }
} 