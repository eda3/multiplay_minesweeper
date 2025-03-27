use wasm_multiplayer::resources::ResourceManager;
use std::rc::Rc;
use std::cell::RefCell;

#[derive(Debug, Clone)]
struct TestResource {
    value: i32
}

#[test]
fn test_new_resource_manager_is_empty() {
    let resource_manager = ResourceManager::new();
    assert_eq!(resource_manager.resource_count(), 0);
}

#[test]
fn test_add_resource() {
    let mut resource_manager = ResourceManager::new();
    let test_resource = TestResource { value: 42 };
    
    resource_manager.add_resource("test", test_resource);
    assert_eq!(resource_manager.resource_count(), 1);
}

#[test]
fn test_get_resource() {
    let mut resource_manager = ResourceManager::new();
    let test_resource = TestResource { value: 42 };
    
    resource_manager.add_resource("test", test_resource);
    
    if let Some(resource) = resource_manager.get_resource::<TestResource>("test") {
        let resource = resource.borrow();
        assert_eq!(resource.value, 42);
    } else {
        panic!("Resource not found when it should exist");
    }
}

#[test]
fn test_get_nonexistent_resource() {
    let resource_manager = ResourceManager::new();
    let resource = resource_manager.get_resource::<TestResource>("nonexistent");
    assert!(resource.is_none());
}

#[test]
fn test_remove_resource() {
    let mut resource_manager = ResourceManager::new();
    let test_resource = TestResource { value: 42 };
    
    resource_manager.add_resource("test", test_resource);
    assert_eq!(resource_manager.resource_count(), 1);
    
    resource_manager.remove_resource("test");
    assert_eq!(resource_manager.resource_count(), 0);
}

#[test]
fn test_batch_access() {
    let mut resource_manager = ResourceManager::new();
    resource_manager.add_resource("res1", TestResource { value: 1 });
    resource_manager.add_resource("res2", TestResource { value: 2 });
    
    let result = resource_manager.batch(|batch| {
        let res1 = batch.get::<TestResource>("res1").unwrap();
        let res2 = batch.get::<TestResource>("res2").unwrap();
        
        res1.borrow().value + res2.borrow().value
    });
    
    assert_eq!(result, 3);
}

#[test]
fn test_batch_mut_access() {
    let mut resource_manager = ResourceManager::new();
    resource_manager.add_resource("res", TestResource { value: 10 });
    
    resource_manager.batch_mut(|batch| {
        if let Some(res) = batch.get_mut::<TestResource>("res") {
            res.borrow_mut().value += 5;
        }
    });
    
    if let Some(res) = resource_manager.get_resource::<TestResource>("res") {
        assert_eq!(res.borrow().value, 15);
    } else {
        panic!("Resource not found");
    }
} 