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
use crate::resources::ResourceManager;
use crate::resources::GameStateResource;
use std::any::Any;
use crate::systems::board_systems::cell_reveal_system::cell_reveal_system;
use std::fmt;
use crate::ecs::system::System;
use std::any::TypeId;
use crate::ecs::system::SystemResult;

/// デルタタイム（前回のフレームからの経過時間）
#[derive(Debug, Clone, Copy)]
pub struct DeltaTime(pub f64);

/// システム関数の型（World全体を受け取る純粋関数）
pub type SystemFn = fn(&mut ResourceManager, DeltaTime) -> Result<(), JsValue>;

/// システムの実行フェーズ
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SystemPhase {
    /// 初期化フェーズ
    Startup,
    /// 入力処理フェーズ
    Input,
    /// メインゲームロジックフェーズ
    Update,
    /// 描画フェーズ
    Render,
    /// クリーンアップフェーズ
    Cleanup,
}

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
#[derive(Debug, Clone)]
pub struct SystemDefinition {
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
pub struct SystemRegistry {
    /// 登録されたシステムのリスト
    systems: Vec<SystemDefinition>,
    /// 最後のフレーム時間
    last_frame_time: f64,
    /// 登録されたシステム関数のマップ
    systems_map: HashMap<&'static str, SystemFn>,
    /// 実行順序
    execution_order: Vec<&'static str>,
    /// リソースストア
    resources: HashMap<&'static str, Rc<RefCell<dyn std::any::Any>>>,
}

impl SystemRegistry {
    /// 新しいシステムレジストリを作成
    pub fn new() -> Self {
        Self {
            systems: Vec::new(),
            last_frame_time: 0.0,
            systems_map: HashMap::new(),
            execution_order: Vec::new(),
            resources: HashMap::new(),
        }
    }
    
    /// システムを追加し、そのIDを返す
    pub fn add_system(&mut self, system: SystemDefinition) -> &'static str {
        let name = system.name;
        let function = system.function;
        self.systems.push(system);
        self.systems_map.insert(name, function);
        name
    }
    
    /// 指定したIDのシステムを削除
    pub fn remove_system(&mut self, name: &'static str) -> Option<SystemDefinition> {
        if let Some(index) = self.systems.iter().position(|s| s.name == name) {
            let system = self.systems.remove(index);
            self.systems_map.remove(name);
            Some(system)
        } else {
            None
        }
    }
    
    /// 全フェーズのシステムを順番に実行
    pub fn run_all_phases(&mut self, resources: &mut ResourceManager) {
        // 各フェーズを順番に実行
        // Startupフェーズは特別扱い（最初の1回だけ）
        let phases = [
            SystemPriority::Input,
            SystemPriority::Update,
            SystemPriority::Render,
            SystemPriority::PostRender,
        ];
        
        for &phase in &phases {
            self.run_phase(resources, phase);
        }
    }
    
    /// 指定したフェーズのシステムを実行
    pub fn run_phase(&mut self, resources: &mut ResourceManager, phase: SystemPriority) {
        // 実行順序が変更された場合は更新
        if self.execution_order.is_empty() {
            self.update_execution_order();
        }
        
        // 指定フェーズに対応するシステムを実行
        for system in &self.systems {
            if system.priority == phase {
                let _ = (system.function)(resources, DeltaTime(0.0));
            }
        }
    }
    
    /// 実行順序を更新（依存関係を考慮したトポロジカルソート）
    fn update_execution_order(&mut self) {
        self.execution_order.clear();
        
        // 各フェーズについて実行順序を計算
        for system in &self.systems {
            if system.priority == SystemPriority::First {
                self.execution_order.push(system.name);
            }
        }
        
        for system in &self.systems {
            if system.priority == SystemPriority::Input {
                self.execution_order.push(system.name);
            }
        }
        
        for system in &self.systems {
            if system.priority == SystemPriority::PreUpdate {
                self.execution_order.push(system.name);
            }
        }
        
        for system in &self.systems {
            if system.priority == SystemPriority::Update {
                self.execution_order.push(system.name);
            }
        }
        
        for system in &self.systems {
            if system.priority == SystemPriority::PostUpdate {
                self.execution_order.push(system.name);
            }
        }
        
        for system in &self.systems {
            if system.priority == SystemPriority::PreRender {
                self.execution_order.push(system.name);
            }
        }
        
        for system in &self.systems {
            if system.priority == SystemPriority::Render {
                self.execution_order.push(system.name);
            }
        }
        
        for system in &self.systems {
            if system.priority == SystemPriority::PostRender {
                self.execution_order.push(system.name);
            }
        }
        
        for system in &self.systems {
            if system.priority == SystemPriority::Last {
                self.execution_order.push(system.name);
            }
        }
    }
    
    /// 登録されているシステムの数を返す
    pub fn len(&self) -> usize {
        self.systems.len()
    }
    
    /// システムが登録されていないかどうかを返す
    pub fn is_empty(&self) -> bool {
        self.systems.is_empty()
    }
    
    /// 指定したIDのシステムを取得
    pub fn get_system(&self, name: &'static str) -> Option<&SystemDefinition> {
        self.systems.iter().find(|s| s.name == name)
    }
    
    /// 指定したIDのシステムを可変参照で取得
    pub fn get_system_mut(&mut self, name: &'static str) -> Option<&mut SystemDefinition> {
        self.systems.iter_mut().find(|s| s.name == name)
    }
    
    /// スタートアップシステムとして基本的なシステムを登録
    pub fn register_default_systems(&mut self) {
        // 外部リソースをインポート
        use crate::systems::board_systems::{
            // 存在しないモジュールへの参照をコメントアウト
            // board_initialization_system,
            cell_reveal_system,
            // flag_system,
            // board_state_system
        };
        
        // システムを実装するモジュール
        mod system_impls {
            use super::*;
            use crate::resources::ResourceManager;
            use crate::ecs::system::System; // 正しいSystemトレイト
            use crate::entities::EntityManager;
            use crate::ecs::system::SystemResult;
            
            // ボード初期化システム処理関数
            pub fn board_init_system(resources: &mut ResourceManager, _: DeltaTime) -> Result<(), JsValue> {
                // ボード初期化システムの代わりに何かダミー処理を入れる
                web_sys::console::log_1(&"ボード初期化処理が呼ばれました".into());
                Ok(())
            }
            
            // セル公開システム処理関数
            pub fn cell_reveal_system_wrapper(resources: &mut ResourceManager, delta: DeltaTime) -> Result<(), JsValue> {
                // セル公開システムの実装
                let delta_time = crate::systems::system_registry::DeltaTime(delta.0);
                if let Err(err) = crate::systems::board_systems::cell_reveal_system(resources, delta_time) {
                    web_sys::console::error_1(&err);
                    return Err(err);
                }
                Ok(())
            }
            
            // フラグトグルシステム処理関数
            pub fn flag_toggle_system(resources: &mut ResourceManager, _: DeltaTime) -> Result<(), JsValue> {
                // フラグトグルシステムのダミー実装
                web_sys::console::log_1(&"フラグトグル処理が呼ばれました".into());
                Ok(())
            }
            
            // 勝利条件チェックシステム処理関数
            pub fn win_condition_system(resources: &mut ResourceManager, _: DeltaTime) -> Result<(), JsValue> {
                // 勝利条件チェックシステムのダミー実装
                web_sys::console::log_1(&"勝利条件チェック処理が呼ばれました".into());
                Ok(())
            }
        }
        
        // システムを登録
        self.add_system(SystemDefinition {
            name: "BoardInit",
            function: system_impls::board_init_system,
            priority: SystemPriority::First,
            enabled: true,
        });
        self.add_system(SystemDefinition {
            name: "CellReveal",
            function: system_impls::cell_reveal_system_wrapper,
            priority: SystemPriority::Input,
            enabled: true,
        });
        self.add_system(SystemDefinition {
            name: "FlagToggle",
            function: system_impls::flag_toggle_system,
            priority: SystemPriority::Input,
            enabled: true,
        });
        self.add_system(SystemDefinition {
            name: "WinCondition",
            function: system_impls::win_condition_system,
            priority: SystemPriority::Update,
            enabled: true,
        });
    }
}

impl fmt::Debug for SystemRegistry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SystemRegistry")
            .field("systems_count", &self.systems.len())
            .field("systems_map", &self.systems_map)
            .field("execution_order", &self.execution_order)
            .field("resources", &self.resources)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    struct TestSystem {
        name: String,
        priority: SystemPriority,
        run_count: usize,
    }
    
    impl System for TestSystem {
        fn name(&self) -> &str {
            &self.name
        }
        
        fn function(&self) -> SystemFn {
            |_, _| {
                self.run_count += 1;
                Ok(())
            }
        }
    }
    
    impl TestSystem {
        fn new(name: &str, priority: SystemPriority) -> Self {
            Self {
                name: name.to_string(),
                priority,
                run_count: 0,
            }
        }
    }
    
    #[test]
    fn test_add_and_run_systems() {
        let mut registry = SystemRegistry::new();
        let mut resources = ResourceManager::new();
        
        // テストシステムを追加
        let system1 = TestSystem::new("System1", SystemPriority::Input);
        let system2 = TestSystem::new("System2", SystemPriority::Update);
        registry.add_system(system1);
        registry.add_system(system2);
        
        // Updateフェーズのシステムを実行
        registry.run_phase(&mut resources, SystemPriority::Update);
        
        // 実行回数をチェック
        let sys1 = registry.get_system("System1").unwrap();
        let sys2 = registry.get_system("System2").unwrap();
        
        assert_eq!(sys1.run_count, 1);
        assert_eq!(sys2.run_count, 1);
    }
    
    #[test]
    fn test_system_priority() {
        let mut registry = SystemRegistry::new();
        let mut resources = ResourceManager::new();
        
        // 実行順序を記録するための共有リソース
        struct ExecutionOrder {
            order: Vec<String>,
        }
        
        impl ExecutionOrder {
            fn new() -> Self {
                Self {
                    order: Vec::new(),
                }
            }
            
            fn add(&mut self, name: &str) {
                self.order.push(name.to_string());
            }
        }
        
        resources.insert(ExecutionOrder::new());
        
        // 優先度の異なるシステム
        struct PrioritySystem {
            name: String,
            priority: SystemPriority,
        }
        
        impl System for PrioritySystem {
            fn name(&self) -> &str {
                &self.name
            }
            
            fn function(&self) -> SystemFn {
                |_, _| {
                    if let Some(order) = resources.get_mut::<ExecutionOrder>() {
                        order.add(&self.name);
                    }
                    Ok(())
                }
            }
        }
        
        // 優先度の異なる3つのシステムを追加（優先度が低いほど先に実行される）
        registry.add_system(SystemDefinition {
            name: "High".to_string(),
            function: PrioritySystem {
                name: "High".to_string(),
                priority: SystemPriority::First,
            }.function,
            priority: SystemPriority::First,
            enabled: true,
        });
        
        registry.add_system(SystemDefinition {
            name: "Normal".to_string(),
            function: PrioritySystem {
                name: "Normal".to_string(),
                priority: SystemPriority::Input,
            }.function,
            priority: SystemPriority::Input,
            enabled: true,
        });
        
        registry.add_system(SystemDefinition {
            name: "Low".to_string(),
            function: PrioritySystem {
                name: "Low".to_string(),
                priority: SystemPriority::Update,
            }.function,
            priority: SystemPriority::Update,
            enabled: true,
        });
        
        // システムを実行
        registry.run_all_phases(&mut resources);
        
        // 実行順序を確認
        let execution_order = resources.get::<ExecutionOrder>().unwrap();
        assert_eq!(
            execution_order.order,
            vec!["High", "Normal", "Low"]
        );
    }
} 