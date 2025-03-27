#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::EntityManager;
    
    // テスト用のダミーシステム
    #[derive(Debug)]
    struct TestSystem {
        name: &'static str,
        priority: i32,
        dependencies: Vec<&'static str>,
        is_active: bool,
        init_called: bool,
        update_called: bool,
        shutdown_called: bool,
    }
    
    impl TestSystem {
        fn new(name: &'static str, priority: i32) -> Self {
            Self {
                name,
                priority,
                dependencies: Vec::new(),
                is_active: true,
                init_called: false,
                update_called: false,
                shutdown_called: false,
            }
        }
        
        fn with_dependencies(mut self, deps: Vec<&'static str>) -> Self {
            self.dependencies = deps;
            self
        }
    }
    
    impl System for TestSystem {
        fn name(&self) -> &str {
            self.name
        }
        
        fn init(&mut self, _entity_manager: &mut EntityManager) {
            self.init_called = true;
        }
        
        fn update(&mut self, _entity_manager: &mut EntityManager, _delta_time: f32) {
            self.update_called = true;
        }
        
        fn shutdown(&mut self, _entity_manager: &mut EntityManager) {
            self.shutdown_called = true;
        }
        
        fn dependencies(&self) -> Vec<&str> {
            self.dependencies.clone()
        }
        
        fn priority(&self) -> i32 {
            self.priority
        }
        
        fn is_active(&self) -> bool {
            self.is_active
        }
        
        fn set_active(&mut self, active: bool) {
            self.is_active = active;
        }
    }
    
    #[test]
    fn test_system_registry_basic() {
        let mut registry = SystemRegistry::new();
        let system1 = TestSystem::new("System1", 0);
        let system2 = TestSystem::new("System2", 1);
        
        registry.register(system1);
        registry.register(system2);
        
        let mut entity_manager = EntityManager::new();
        let result = registry.update_all(&mut entity_manager, 0.016);
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_system_group() {
        let mut group = SystemGroup::new("TestGroup", 0);
        let system1 = TestSystem::new("System1", 0);
        let system2 = TestSystem::new("System2", 1);
        
        group.add(system1);
        group.add(system2);
        
        assert_eq!(group.system_count(), 2);
        assert!(group.contains_system("System1"));
        assert!(group.contains_system("System2"));
        assert!(!group.contains_system("System3"));
        
        let mut entity_manager = EntityManager::new();
        group.update_all(&mut entity_manager, 0.016);
        
        // 優先度によってSystem1が先に呼ばれるはず
        assert!(group.get_system("System1").unwrap().is_active());
        assert!(group.get_system("System2").unwrap().is_active());
    }
    
    #[test]
    fn test_system_dependencies() {
        let mut registry = SystemRegistry::new();
        
        // A depends on B, B depends on C
        let system_a = TestSystem::new("A", 0).with_dependencies(vec!["B"]);
        let system_b = TestSystem::new("B", 0).with_dependencies(vec!["C"]);
        let system_c = TestSystem::new("C", 0);
        
        registry.register(system_a);
        registry.register(system_b);
        registry.register(system_c);
        
        let mut entity_manager = EntityManager::new();
        let result = registry.update_all(&mut entity_manager, 0.016);
        assert!(result.is_ok());
        
        // 実行順序は C -> B -> A になるはず
    }
    
    #[test]
    fn test_rate_controlled_system() {
        let system = TestSystem::new("RateTest", 0);
        let mut rate_system = RateControlledSystem::new(system, 10.0); // 10 Hz
        
        let mut entity_manager = EntityManager::new();
        
        // 0.05秒は更新間隔（0.1秒）より短いので更新されないはず
        rate_system.update(&mut entity_manager, 0.05);
        assert!(!rate_system.system.update_called);
        
        // さらに0.06秒で合計0.11秒になるので更新されるはず
        rate_system.update(&mut entity_manager, 0.06);
        assert!(rate_system.system.update_called);
    }
    
    #[test]
    fn test_parallel_executor() {
        let mut registry = SystemRegistry::new();
        
        // システム間の依存関係を設定
        let system_a = TestSystem::new("A", 0).with_dependencies(vec!["B", "C"]);
        let system_b = TestSystem::new("B", 0).with_dependencies(vec!["D"]);
        let system_c = TestSystem::new("C", 0).with_dependencies(vec!["D"]);
        let system_d = TestSystem::new("D", 0);
        
        registry.register(system_a);
        registry.register(system_b);
        registry.register(system_c);
        registry.register(system_d);
        
        let mut entity_manager = EntityManager::new();
        let result = registry.update_all(&mut entity_manager, 0.016);
        assert!(result.is_ok());
        
        // 実行順序は D -> (B, C) -> A になるはず（B, Cは並列可能）
    }
} 