/**
 * システムレジストリ
 * 
 * システムの登録と実行を管理します
 */
use std::collections::HashMap;
use wasm_bindgen::JsValue;
use std::rc::Rc;
use std::cell::RefCell;
use crate::entities::EntityManager;
use crate::resources::GameState;

/// デルタタイム（前回のフレームからの経過時間）
#[derive(Debug, Clone, Copy)]
pub struct DeltaTime(pub f64);

/// システム関数の型（World全体を受け取る純粋関数）
pub type SystemFn = fn(&mut EntityManager, &mut HashMap<&'static str, Rc<RefCell<dyn std::any::Any>>>, DeltaTime) -> Result<(), JsValue>;

/// システムの実行優先度
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SystemPriority {
    First = 0,
    Input = 100,
    Network = 200,
    PreUpdate = 300,
    Update = 400,
    PostUpdate = 500,
    PreRender = 600,
    Render = 700,
    PostRender = 800,
    Last = 900,
}

/// システム定義
#[derive(Debug)]
pub struct System {
    /// システム名
    pub name: &'static str,
    /// システム関数
    pub function: SystemFn,
    /// 実行優先度
    pub priority: SystemPriority,
    /// 有効かどうか
    pub enabled: bool,
}

/// システムレジストリ
/// 全システムの登録と実行を管理
#[derive(Debug)]
pub struct SystemRegistry {
    /// 登録されたシステムのリスト
    systems: Vec<System>,
    /// 最後のフレーム時間
    last_frame_time: f64,
    /// 登録されたシステム関数のマップ
    systems_map: HashMap<&'static str, SystemFn>,
    /// 実行順序
    execution_order: Vec<&'static str>,
    /// リソースストア
    resources: HashMap<&'static str, Rc<RefCell<dyn std::any::Any>>>,
}

impl Default for SystemRegistry {
    fn default() -> Self {
        Self {
            systems: Vec::new(),
            last_frame_time: js_sys::Date::now(),
            systems_map: HashMap::new(),
            execution_order: Vec::new(),
            resources: HashMap::new(),
        }
    }
}

impl SystemRegistry {
    /// 新しいシステムレジストリを作成
    pub fn new() -> Self {
        Self::default()
    }
    
    /// システムを登録
    pub fn register_system(&mut self, name: &'static str, function: SystemFn, priority: SystemPriority) {
        let system = System {
            name,
            function,
            priority,
            enabled: true,
        };
        
        self.systems.push(system);
        // 優先度でソート
        self.systems.sort_by_key(|s| s.priority);
        
        self.systems_map.insert(name, function);
        
        // 実行順序に追加（もし既に存在しなければ）
        if !self.execution_order.contains(&name) {
            self.execution_order.push(name);
        }
    }
    
    /// 名前でシステムを有効/無効化
    pub fn set_system_enabled(&mut self, name: &str, enabled: bool) -> bool {
        for system in &mut self.systems {
            if system.name == name {
                system.enabled = enabled;
                return true;
            }
        }
        false
    }
    
    /// 優先度グループでシステムを有効/無効化
    pub fn set_priority_enabled(&mut self, priority: SystemPriority, enabled: bool) {
        for system in &mut self.systems {
            if system.priority == priority {
                system.enabled = enabled;
            }
        }
    }
    
    /// リソースを追加または更新
    pub fn add_resource<T: 'static>(&mut self, name: &'static str, resource: T) {
        self.resources.insert(name, Rc::new(RefCell::new(resource)));
    }
    
    /// リソースを取得
    pub fn get_resource<T: 'static>(&self, name: &'static str) -> Option<Rc<RefCell<T>>> {
        self.resources.get(name).map(|rc_any| {
            let any_ref = rc_any.clone();
            let any_refcell = any_ref.as_ref();
            
            // ダウンキャスト
            Rc::new(RefCell::new(any_refcell.borrow().downcast_ref::<T>().unwrap().clone()))
        })
    }
    
    /// 全システムを実行
    pub fn run_systems(&mut self, entity_manager: &mut EntityManager, delta_time: DeltaTime) -> Result<(), JsValue> {
        for system_name in &self.execution_order {
            if let Some(system) = self.systems_map.get(system_name) {
                system(entity_manager, &mut self.resources, delta_time)?;
            }
        }
        Ok(())
    }
    
    /// 特定の優先度範囲のシステムを実行
    pub fn run_priority_range(
        &mut self,
        entity_manager: &mut EntityManager,
        resources: &mut HashMap<&'static str, Rc<RefCell<dyn std::any::Any>>>,
        min_priority: SystemPriority,
        max_priority: SystemPriority,
    ) -> Result<(), JsValue> {
        // 現在の時間を取得
        let now = js_sys::Date::now();
        let delta_time = (now - self.last_frame_time) / 1000.0; // 秒単位
        
        // 指定された範囲内の有効なシステムを実行
        for system in &self.systems {
            if system.enabled && 
               system.priority >= min_priority && 
               system.priority <= max_priority {
                (system.function)(entity_manager, resources, DeltaTime(delta_time))?;
            }
        }
        
        Ok(())
    }
} 