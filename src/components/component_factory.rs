/**
 * コンポーネントファクトリー
 * 
 * コンポーネントの動的生成を担当するファクトリーシステム
 */
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::fmt;
use crate::components::component_trait::Component;

/// コンポーネントを動的に生成するファクトリー
#[derive(Default)]
pub struct ComponentFactory {
    creators: HashMap<TypeId, Box<dyn Fn() -> Box<dyn Any + Send + Sync>>>,
    type_names: HashMap<String, TypeId>,
}

// Debugトレイトの手動実装
impl fmt::Debug for ComponentFactory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ComponentFactory")
            .field("registered_types", &self.type_names.keys().collect::<Vec<_>>())
            .finish()
    }
}

impl ComponentFactory {
    /// 新しいコンポーネントファクトリーを作成
    pub fn new() -> Self {
        Self {
            creators: HashMap::new(),
            type_names: HashMap::new(),
        }
    }
    
    /// コンポーネント型を登録
    pub fn register<T: Component + Default>(&mut self) {
        let type_id = TypeId::of::<T>();
        let type_name = std::any::type_name::<T>().to_string();
        
        let creator = Box::new(|| Box::new(T::default()) as Box<dyn Any + Send + Sync>);
        self.creators.insert(type_id, creator);
        self.type_names.insert(type_name, type_id);
    }
    
    /// 型IDからデフォルトコンポーネントを作成
    pub fn create_default(&self, type_id: TypeId) -> Result<Box<dyn Any + Send + Sync>, String> {
        self.creators.get(&type_id)
            .ok_or_else(|| format!("登録されていないコンポーネントタイプ: {:?}", type_id))
            .map(|creator| creator())
    }
    
    /// コンポーネント型名からデフォルトコンポーネントを作成
    pub fn create_by_name(&self, name: &str) -> Result<Box<dyn Any + Send + Sync>, String> {
        let type_id = self.type_names.get(name)
            .ok_or_else(|| format!("登録されていないコンポーネント名: {}", name))?;
            
        self.create_default(*type_id)
    }
    
    /// 登録されているすべてのコンポーネント型名を取得
    pub fn get_registered_type_names(&self) -> Vec<String> {
        self.type_names.keys().cloned().collect()
    }
    
    /// 型IDが登録されているか確認
    pub fn is_registered(&self, type_id: TypeId) -> bool {
        self.creators.contains_key(&type_id)
    }
    
    /// 型名から型IDを取得
    pub fn get_type_id(&self, name: &str) -> Option<TypeId> {
        self.type_names.get(name).copied()
    }
} 