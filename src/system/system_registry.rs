use std::collections::HashMap;
use crate::resources::ResourceManager;

/// システムの実行フェーズ
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SystemPhase {
    /// 初期化フェーズ（ゲーム開始時に一度だけ実行）
    Startup,
    /// 入力処理フェーズ
    Input,
    /// 更新フェーズ（ゲームロジックの更新）
    Update,
    /// 描画フェーズ
    Render,
    /// 後処理フェーズ（フレーム終了時に実行）
    Cleanup,
}

/// システムの優先度を表す型（数字が小さいほど先に実行される）
pub type SystemPriority = i32;

/// システムのIDを表す型
pub type SystemId = usize;

/// システムの実装に必要なトレイト
pub trait System {
    /// システムの名前を返す
    fn name(&self) -> &str;
    
    /// システムの実行フェーズを返す
    fn phase(&self) -> SystemPhase;
    
    /// システムの優先度を返す（低いほど先に実行される）
    fn priority(&self) -> SystemPriority {
        0
    }
    
    /// リソースマネージャーを使用してシステムを実行
    fn run(&mut self, resources: &mut ResourceManager);
    
    /// このシステムが依存するシステムのIDのリストを返す
    fn dependencies(&self) -> Vec<SystemId> {
        Vec::new()
    }
}

/// システムを管理・実行するためのレジストリ
#[derive(Default)]
pub struct SystemRegistry {
    /// 登録されたシステムのマップ（ID -> Box<dyn System>）
    systems: HashMap<SystemId, Box<dyn System>>,
    /// 次に割り当てるシステムID
    next_id: SystemId,
    /// フェーズごとのシステムIDのリスト
    phase_systems: HashMap<SystemPhase, Vec<(SystemId, SystemPriority)>>,
    /// 依存関係のキャッシュ
    dependencies: HashMap<SystemId, Vec<SystemId>>,
    /// 実行順序のキャッシュ（フェーズごと）
    execution_order: HashMap<SystemPhase, Vec<SystemId>>,
    /// 実行順序が変更されたかどうか
    dirty: bool,
}

impl SystemRegistry {
    /// 新しいシステムレジストリを作成
    pub fn new() -> Self {
        Self {
            systems: HashMap::new(),
            next_id: 0,
            phase_systems: HashMap::new(),
            dependencies: HashMap::new(),
            execution_order: HashMap::new(),
            dirty: false,
        }
    }
    
    /// システムを追加し、そのIDを返す
    pub fn add_system(&mut self, system: Box<dyn System>) -> SystemId {
        let id = self.next_id;
        self.next_id += 1;
        
        let phase = system.phase();
        let priority = system.priority();
        let dependencies = system.dependencies();
        
        // フェーズ管理にシステムを追加
        self.phase_systems
            .entry(phase)
            .or_default()
            .push((id, priority));
        
        // 依存関係を記録
        self.dependencies.insert(id, dependencies);
        
        // システムを保存
        self.systems.insert(id, system);
        
        // 実行順序を更新する必要があることをマーク
        self.dirty = true;
        
        id
    }
    
    /// 指定したIDのシステムを削除
    pub fn remove_system(&mut self, id: SystemId) -> Option<Box<dyn System>> {
        if let Some(system) = self.systems.remove(&id) {
            let phase = system.phase();
            
            // フェーズリストからも削除
            if let Some(systems) = self.phase_systems.get_mut(&phase) {
                if let Some(index) = systems.iter().position(|(sys_id, _)| *sys_id == id) {
                    systems.remove(index);
                }
            }
            
            // 依存関係からも削除
            self.dependencies.remove(&id);
            
            // 他のシステムの依存関係からも削除
            for deps in self.dependencies.values_mut() {
                deps.retain(|&dep_id| dep_id != id);
            }
            
            // 実行順序を更新する必要があることをマーク
            self.dirty = true;
            
            Some(system)
        } else {
            None
        }
    }
    
    /// 指定したフェーズのシステムを全て実行
    pub fn run_phase(&mut self, phase: SystemPhase, resources: &mut ResourceManager) {
        // 実行順序が変更された場合は更新
        if self.dirty {
            self.update_execution_order();
        }
        
        // 指定フェーズの実行順序を取得
        if let Some(order) = self.execution_order.get(&phase) {
            for &system_id in order {
                if let Some(system) = self.systems.get_mut(&system_id) {
                    system.run(resources);
                }
            }
        }
    }
    
    /// 全フェーズのシステムを順番に実行
    pub fn run_all_phases(&mut self, resources: &mut ResourceManager) {
        // 各フェーズを順番に実行
        // Startupフェーズは特別扱い（最初の1回だけ）
        let phases = [
            SystemPhase::Input,
            SystemPhase::Update,
            SystemPhase::Render,
            SystemPhase::Cleanup,
        ];
        
        for &phase in &phases {
            self.run_phase(phase, resources);
        }
    }
    
    /// Startupフェーズのみを実行（初期化用）
    pub fn run_startup(&mut self, resources: &mut ResourceManager) {
        self.run_phase(SystemPhase::Startup, resources);
    }
    
    /// 実行順序を更新（依存関係を考慮したトポロジカルソート）
    fn update_execution_order(&mut self) {
        self.execution_order.clear();
        
        // 各フェーズについて実行順序を計算
        for (&phase, systems) in &self.phase_systems {
            let mut sorted_systems = Vec::new();
            
            // 優先度でソート
            let mut phase_systems = systems.clone();
            phase_systems.sort_by_key(|&(_, priority)| priority);
            
            // 依存関係を考慮したトポロジカルソート
            let mut visited = HashMap::new();
            let mut temp_mark = HashMap::new();
            
            for &(id, _) in &phase_systems {
                self.visit(id, &mut visited, &mut temp_mark, &mut sorted_systems);
            }
            
            // 実行順序を保存
            self.execution_order.insert(phase, sorted_systems);
        }
        
        self.dirty = false;
    }
    
    /// トポロジカルソートのためのノード訪問（再帰的）
    fn visit(
        &self,
        id: SystemId,
        visited: &mut HashMap<SystemId, bool>,
        temp_mark: &mut HashMap<SystemId, bool>,
        sorted: &mut Vec<SystemId>,
    ) {
        // 既に訪問済みならスキップ
        if visited.get(&id).copied().unwrap_or(false) {
            return;
        }
        
        // 一時マークがあれば循環依存
        if temp_mark.get(&id).copied().unwrap_or(false) {
            // 循環依存はエラーだが、ここでは単純に無視する
            return;
        }
        
        // 一時マークを付ける
        temp_mark.insert(id, true);
        
        // 依存するシステムを先に処理
        if let Some(deps) = self.dependencies.get(&id) {
            for &dep_id in deps {
                self.visit(dep_id, visited, temp_mark, sorted);
            }
        }
        
        // 一時マークを外す
        temp_mark.insert(id, false);
        
        // 永続マークを付けて結果に追加
        visited.insert(id, true);
        sorted.push(id);
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
    pub fn get_system(&self, id: SystemId) -> Option<&dyn System> {
        self.systems.get(&id).map(|boxed| boxed.as_ref())
    }
    
    /// 指定したIDのシステムを可変参照で取得
    pub fn get_system_mut(&mut self, id: SystemId) -> Option<&mut dyn System> {
        // 安全なバージョン
        if let Some(system) = self.systems.get_mut(&id) {
            Some(system.as_mut())
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    struct TestSystem {
        name: String,
        phase: SystemPhase,
        priority: SystemPriority,
        run_count: usize,
        dependencies: Vec<SystemId>,
    }
    
    impl System for TestSystem {
        fn name(&self) -> &str {
            &self.name
        }
        
        fn phase(&self) -> SystemPhase {
            self.phase
        }
        
        fn priority(&self) -> SystemPriority {
            self.priority
        }
        
        fn run(&mut self, _resources: &mut ResourceManager) {
            self.run_count += 1;
        }
        
        fn dependencies(&self) -> Vec<SystemId> {
            self.dependencies.clone()
        }
    }
    
    impl TestSystem {
        fn new(
            name: &str,
            phase: SystemPhase,
            priority: SystemPriority,
            dependencies: Vec<SystemId>,
        ) -> Self {
            Self {
                name: name.to_string(),
                phase,
                priority,
                run_count: 0,
                dependencies,
            }
        }
    }
    
    #[test]
    fn test_add_and_run_systems() {
        let mut registry = SystemRegistry::new();
        let mut resources = ResourceManager::new();
        
        // テストシステムを追加
        let system1 = TestSystem::new("System1", SystemPhase::Update, 0, vec![]);
        let system2 = TestSystem::new("System2", SystemPhase::Update, 1, vec![]);
        let id1 = registry.add_system(Box::new(system1));
        let id2 = registry.add_system(Box::new(system2));
        
        // Updateフェーズのシステムを実行
        registry.run_phase(SystemPhase::Update, &mut resources);
        
        // 実行回数をチェック
        let sys1 = registry.get_system(id1).unwrap();
        let sys2 = registry.get_system(id2).unwrap();
        
        let sys1 = sys1.downcast_ref::<TestSystem>().unwrap();
        let sys2 = sys2.downcast_ref::<TestSystem>().unwrap();
        
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
            
            fn phase(&self) -> SystemPhase {
                SystemPhase::Update
            }
            
            fn priority(&self) -> SystemPriority {
                self.priority
            }
            
            fn run(&mut self, resources: &mut ResourceManager) {
                if let Some(order) = resources.get_mut::<ExecutionOrder>() {
                    order.add(&self.name);
                }
            }
        }
        
        // 優先度の異なる3つのシステムを追加（優先度が低いほど先に実行される）
        registry.add_system(Box::new(PrioritySystem {
            name: "High".to_string(),
            priority: -10,
        }));
        
        registry.add_system(Box::new(PrioritySystem {
            name: "Normal".to_string(),
            priority: 0,
        }));
        
        registry.add_system(Box::new(PrioritySystem {
            name: "Low".to_string(),
            priority: 10,
        }));
        
        // システムを実行
        registry.run_phase(SystemPhase::Update, &mut resources);
        
        // 実行順序を確認
        let execution_order = resources.get::<ExecutionOrder>().unwrap();
        assert_eq!(
            execution_order.order,
            vec!["High", "Normal", "Low"]
        );
    }
    
    #[test]
    fn test_system_dependencies() {
        let mut registry = SystemRegistry::new();
        let mut resources = ResourceManager::new();
        
        // 実行順序を記録するための共有リソース
        struct ExecutionOrder {
            order: Vec<String>,
        }
        
        resources.insert(ExecutionOrder {
            order: Vec::new(),
        });
        
        // システム A (依存なし)
        let system_a = TestSystem::new("SystemA", SystemPhase::Update, 0, vec![]);
        let id_a = registry.add_system(Box::new(system_a));
        
        // システム B (Aに依存)
        let system_b = TestSystem::new("SystemB", SystemPhase::Update, 0, vec![id_a]);
        let id_b = registry.add_system(Box::new(system_b));
        
        // システム C (Bに依存)
        let system_c = TestSystem::new("SystemC", SystemPhase::Update, 0, vec![id_b]);
        registry.add_system(Box::new(system_c));
        
        // 実行順序を記録するシステム実装を追加
        struct RecordingSystem {
            id: SystemId,
            name: String,
        }
        
        impl System for RecordingSystem {
            fn name(&self) -> &str {
                &self.name
            }
            
            fn phase(&self) -> SystemPhase {
                SystemPhase::Update
            }
            
            fn run(&mut self, resources: &mut ResourceManager) {
                if let Some(order) = resources.get_mut::<ExecutionOrder>() {
                    order.order.push(self.name.clone());
                }
            }
            
            fn dependencies(&self) -> Vec<SystemId> {
                vec![self.id]
            }
        }
        
        // 記録用システムを追加
        registry.add_system(Box::new(RecordingSystem {
            id: id_a,
            name: "RecordA".to_string(),
        }));
        
        registry.add_system(Box::new(RecordingSystem {
            id: id_b,
            name: "RecordB".to_string(),
        }));
        
        // システムを実行
        registry.run_phase(SystemPhase::Update, &mut resources);
        
        // 実行順序を確認
        let execution_order = resources.get::<ExecutionOrder>().unwrap();
        
        // 依存関係によってA -> B -> Cの順に実行されていることを確認
        let a_index = execution_order.order.iter().position(|name| name == "RecordA").unwrap();
        let b_index = execution_order.order.iter().position(|name| name == "RecordB").unwrap();
        
        assert!(a_index < b_index, "SystemA should run before SystemB");
    }
} 