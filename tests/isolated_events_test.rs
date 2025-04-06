/**
 * 完全に分離された型付きイベントシステムのテスト
 * 
 * このモジュールはメインプロジェクトから完全に独立したテスト環境を提供します。
 */
use std::any::Any;
use std::fmt::Debug;
use std::sync::{Arc, Mutex};

/// シンプルな型消去イベントトレイト
pub trait SimpleTypedEvent: Clone + Debug + Send + Sync + 'static {
    fn type_name() -> &'static str;
    fn as_any(&self) -> &dyn Any;
}

/// ハンドラID型
pub type HandlerId = u64;

/// シンプルなイベントハンドラ
#[derive(Clone)]
pub struct SimpleEventHandler<E: SimpleTypedEvent> {
    pub id: HandlerId,
    pub name: String,
    handler: Arc<dyn Fn(&E) + Send + Sync + 'static>,
}

impl<E: SimpleTypedEvent> SimpleEventHandler<E> {
    pub fn new(name: &str, handler: impl Fn(&E) + Send + Sync + 'static) -> Self {
        Self {
            id: rand::random::<u64>(),
            name: name.to_string(),
            handler: Arc::new(handler),
        }
    }
    
    pub fn handle(&self, event: &E) {
        (self.handler)(event);
    }
}

/// シンプルなイベントバス
#[derive(Clone, Default)]
pub struct SimpleEventBus {
    handlers: Arc<Mutex<Vec<(HandlerId, String, Box<dyn Fn(&dyn Any) + Send + Sync>)>>>,
    history: Arc<Mutex<Vec<Box<dyn Any + Send + Sync>>>>,
}

impl SimpleEventBus {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn subscribe<E: SimpleTypedEvent, F>(&self, name: &str, handler: F) -> HandlerId
    where
        F: Fn(&E) + Send + Sync + 'static,
    {
        let id = rand::random::<u64>();
        let name = name.to_string();
        
        let boxed_handler = Box::new(move |event: &dyn Any| {
            if let Some(typed_event) = event.downcast_ref::<E>() {
                handler(typed_event);
            }
        });
        
        let mut handlers = self.handlers.lock().unwrap();
        handlers.push((id, name, boxed_handler));
        
        id
    }
    
    pub fn publish<E: SimpleTypedEvent>(&self, event: E) {
        let event_box = Box::new(event.clone());
        
        // イベント履歴に追加
        {
            let mut history = self.history.lock().unwrap();
            history.push(event_box);
        }
        
        // ハンドラを実行
        let handlers = self.handlers.lock().unwrap();
        for (_, _, handler) in &*handlers {
            handler(&event as &dyn Any);
        }
    }
    
    pub fn get_history<E: SimpleTypedEvent>(&self) -> Vec<E> {
        let history = self.history.lock().unwrap();
        
        history.iter()
            .filter_map(|event| {
                event.downcast_ref::<E>().cloned()
            })
            .collect()
    }
    
    pub fn get_last_event<E: SimpleTypedEvent>(&self) -> Option<E> {
        let history = self.history.lock().unwrap();
        
        history.iter()
            .filter_map(|event| {
                event.downcast_ref::<E>().cloned()
            })
            .last()
    }
    
    pub fn clear_history(&self) {
        let mut history = self.history.lock().unwrap();
        history.clear();
    }
    
    pub fn unregister(&self, handler_id: HandlerId) {
        let mut handlers = self.handlers.lock().unwrap();
        handlers.retain(|(id, _, _)| *id != handler_id);
    }
}

// テスト用のイベント型
#[derive(Debug, Clone, PartialEq)]
struct TestEvent {
    pub id: u32,
    pub message: String,
}

impl SimpleTypedEvent for TestEvent {
    fn type_name() -> &'static str {
        "TestEvent"
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
}

// 別のテスト用イベント型
#[derive(Debug, Clone, PartialEq)]
struct AnotherTestEvent {
    pub value: f64,
    pub is_valid: bool,
}

impl SimpleTypedEvent for AnotherTestEvent {
    fn type_name() -> &'static str {
        "AnotherTestEvent"
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::Rng;
    
    #[test]
    fn test_basic_pub_sub() {
        let event_bus = SimpleEventBus::new();
        let received = Arc::new(Mutex::new(Vec::new()));
        
        let received_clone = received.clone();
        
        let handler_id = event_bus.subscribe("test_handler", move |event: &TestEvent| {
            println!("イベント受信: {:?}", event);
            let mut received = received_clone.lock().unwrap();
            received.push(event.clone());
        });
        
        let test_event = TestEvent {
            id: 1,
            message: "テストメッセージ".to_string(),
        };
        
        event_bus.publish(test_event.clone());
        
        let received = received.lock().unwrap();
        assert_eq!(received.len(), 1);
        assert_eq!(received[0], test_event);
        
        // ハンドラ登録解除のテスト
        event_bus.unregister(handler_id);
        
        let test_event2 = TestEvent {
            id: 2,
            message: "2つ目のメッセージ".to_string(),
        };
        
        event_bus.publish(test_event2);
        
        // ハンドラが解除されたので、receivedには追加されない
        let received = received.lock().unwrap();
        assert_eq!(received.len(), 1);
    }
    
    #[test]
    fn test_multiple_event_types() {
        let event_bus = SimpleEventBus::new();
        
        let test_events = Arc::new(Mutex::new(Vec::new()));
        let another_events = Arc::new(Mutex::new(Vec::new()));
        
        let test_events_clone = test_events.clone();
        let another_events_clone = another_events.clone();
        
        event_bus.subscribe("test_handler", move |event: &TestEvent| {
            let mut events = test_events_clone.lock().unwrap();
            events.push(event.clone());
        });
        
        event_bus.subscribe("another_handler", move |event: &AnotherTestEvent| {
            let mut events = another_events_clone.lock().unwrap();
            events.push(event.clone());
        });
        
        let test_event = TestEvent {
            id: 42,
            message: "型安全性テスト".to_string(),
        };
        
        let another_event = AnotherTestEvent {
            value: 3.14,
            is_valid: true,
        };
        
        event_bus.publish(test_event.clone());
        event_bus.publish(another_event.clone());
        
        let received_test = test_events.lock().unwrap();
        let received_another = another_events.lock().unwrap();
        
        assert_eq!(received_test.len(), 1);
        assert_eq!(received_test[0], test_event);
        
        assert_eq!(received_another.len(), 1);
        assert_eq!(received_another[0], another_event);
    }
    
    #[test]
    fn test_event_history() {
        let event_bus = SimpleEventBus::new();
        
        event_bus.publish(TestEvent {
            id: 1,
            message: "最初のイベント".to_string(),
        });
        
        event_bus.publish(TestEvent {
            id: 2,
            message: "2番目のイベント".to_string(),
        });
        
        event_bus.publish(AnotherTestEvent {
            value: 1.23,
            is_valid: true,
        });
        
        let test_history = event_bus.get_history::<TestEvent>();
        assert_eq!(test_history.len(), 2);
        assert_eq!(test_history[0].id, 1);
        assert_eq!(test_history[1].id, 2);
        
        let last_test_event = event_bus.get_last_event::<TestEvent>();
        assert!(last_test_event.is_some());
        assert_eq!(last_test_event.unwrap().id, 2);
        
        let another_history = event_bus.get_history::<AnotherTestEvent>();
        assert_eq!(another_history.len(), 1);
        assert_eq!(another_history[0].value, 1.23);
    }
    
    #[test]
    fn test_performance() {
        let event_bus = SimpleEventBus::new();
        let mut rng = rand::thread_rng();
        
        // たくさんのハンドラを登録
        for i in 0..100 {
            event_bus.subscribe(&format!("handler_{}", i), move |event: &TestEvent| {
                // 無効な処理
                let _ = event.id + 1;
            });
        }
        
        // 多数のイベントを発行
        for i in 0..100 {
            event_bus.publish(TestEvent {
                id: i,
                message: format!("パフォーマンステスト {}", i),
            });
        }
        
        // ランダムな別のイベント
        for _ in 0..50 {
            event_bus.publish(AnotherTestEvent {
                value: rng.gen_range(0.0..100.0),
                is_valid: rng.gen_bool(0.5),
            });
        }
        
        // 履歴が正しく記録されていることを確認
        let test_history = event_bus.get_history::<TestEvent>();
        assert_eq!(test_history.len(), 100);
        
        let another_history = event_bus.get_history::<AnotherTestEvent>();
        assert_eq!(another_history.len(), 50);
    }
} 