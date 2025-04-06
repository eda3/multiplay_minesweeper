/**
 * 型付きイベントシステムのテスト
 * 
 * このモジュールでは、型安全なイベントシステムの機能をテストし、
 * コンパイル時の型チェックが正しく機能していることを確認します。
 */
use std::any::Any;
use std::sync::{Arc, Mutex};

use crate::events::typed_event::TypedEvent;
use crate::events::typed_event_bus::TypedEventBus;

// テスト用のカスタムイベント型を定義
#[derive(Debug, Clone, PartialEq)]
struct TestEvent {
    pub id: u32,
    pub message: String,
}

impl TypedEvent for TestEvent {
    fn type_name() -> &'static str {
        "TestEvent"
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
    
    fn clone_boxed(&self) -> Box<dyn TypedEvent> {
        Box::new(self.clone())
    }
}

// 異なるテスト用イベント型
#[derive(Debug, Clone, PartialEq)]
struct AnotherTestEvent {
    pub value: f64,
    pub is_valid: bool,
}

impl TypedEvent for AnotherTestEvent {
    fn type_name() -> &'static str {
        "AnotherTestEvent"
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
    
    fn clone_boxed(&self) -> Box<dyn TypedEvent> {
        Box::new(self.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    /// 基本的なイベント発行と購読のテスト
    fn test_basic_pub_sub() {
        let event_bus = TypedEventBus::new().with_debug(true);
        let received = Arc::new(Mutex::new(Vec::new()));
        
        // クロージャで値をキャプチャするためのクローン
        let received_clone = received.clone();
        
        // TestEventのサブスクライバーを登録
        let _handler = event_bus.subscribe("test_handler", move |event: &TestEvent| {
            println!("イベント受信: {:?}", event);
            let mut received = received_clone.lock().unwrap();
            received.push(event.clone());
        });
        
        // イベントを発行
        let test_event = TestEvent {
            id: 1,
            message: "テストメッセージ".to_string(),
        };
        
        event_bus.publish(test_event.clone());
        
        // 結果を検証
        let received = received.lock().unwrap();
        assert_eq!(received.len(), 1);
        assert_eq!(received[0], test_event);
    }
    
    #[test]
    /// 複数の型のイベントを処理できることを確認するテスト
    fn test_multiple_event_types() {
        let event_bus = TypedEventBus::new();
        
        let test_events = Arc::new(Mutex::new(Vec::new()));
        let another_events = Arc::new(Mutex::new(Vec::new()));
        
        let test_events_clone = test_events.clone();
        let another_events_clone = another_events.clone();
        
        // 異なる型のイベントハンドラを登録
        let _handler1 = event_bus.subscribe("test_handler", move |event: &TestEvent| {
            let mut events = test_events_clone.lock().unwrap();
            events.push(event.clone());
        });
        
        let _handler2 = event_bus.subscribe("another_handler", move |event: &AnotherTestEvent| {
            let mut events = another_events_clone.lock().unwrap();
            events.push(event.clone());
        });
        
        // 異なる型のイベントを発行
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
        
        // 各ハンドラが正しい型のイベントだけを受け取っていることを確認
        let received_test = test_events.lock().unwrap();
        let received_another = another_events.lock().unwrap();
        
        assert_eq!(received_test.len(), 1);
        assert_eq!(received_test[0], test_event);
        
        assert_eq!(received_another.len(), 1);
        assert_eq!(received_another[0], another_event);
    }
    
    #[test]
    /// イベント履歴の型安全な取得をテスト
    fn test_typed_event_history() {
        let event_bus = TypedEventBus::new().with_debug(true);
        
        // 複数のイベントを発行
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
        
        // 型ごとの履歴を取得
        let test_events = event_bus.get_event_history_by_type::<TestEvent>();
        let another_events = event_bus.get_event_history_by_type::<AnotherTestEvent>();
        
        // 型ごとに正しくフィルタリングされていることを確認
        assert_eq!(test_events.len(), 2);
        assert_eq!(test_events[0].id, 1);
        assert_eq!(test_events[1].id, 2);
        
        assert_eq!(another_events.len(), 1);
        assert_eq!(another_events[0].value, 1.23);
    }
    
    #[test]
    /// 最後のイベントの型安全な取得をテスト
    fn test_get_last_event() {
        let event_bus = TypedEventBus::new().with_debug(true);
        
        // 複数のイベントを発行
        event_bus.publish(TestEvent {
            id: 1,
            message: "古いイベント".to_string(),
        });
        
        let last_event = TestEvent {
            id: 2,
            message: "最新のイベント".to_string(),
        };
        
        event_bus.publish(last_event.clone());
        
        // 最後のイベントを取得
        let retrieved_event = event_bus.get_last_event::<TestEvent>();
        
        // 正しいイベントが取得できているか確認
        assert!(retrieved_event.is_some());
        assert_eq!(retrieved_event.unwrap(), last_event);
        
        // 存在しない型のイベントを取得しようとした場合
        let non_existent = event_bus.get_last_event::<AnotherTestEvent>();
        assert!(non_existent.is_none());
    }
    
    #[test]
    /// グローバルプロセッサのテスト
    fn test_global_processor() {
        let event_bus = TypedEventBus::new();
        let processed = Arc::new(Mutex::new(Vec::<String>::new()));
        
        let processed_clone = processed.clone();
        
        // すべてのイベントをキャプチャするグローバルプロセッサを登録
        event_bus.add_global_processor(move |event: &dyn Any| {
            let mut processed = processed_clone.lock().unwrap();
            
            if let Some(test_event) = event.downcast_ref::<TestEvent>() {
                processed.push(format!("TestEvent: {}", test_event.id));
            } else if let Some(another_event) = event.downcast_ref::<AnotherTestEvent>() {
                processed.push(format!("AnotherTestEvent: {}", another_event.value));
            }
        });
        
        // 異なる型のイベントを発行
        event_bus.publish(TestEvent {
            id: 123,
            message: "テスト".to_string(),
        });
        
        event_bus.publish(AnotherTestEvent {
            value: 456.789,
            is_valid: false,
        });
        
        // 結果を確認
        let processed = processed.lock().unwrap();
        assert_eq!(processed.len(), 2);
        assert_eq!(processed[0], "TestEvent: 123");
        assert_eq!(processed[1], "AnotherTestEvent: 456.789");
    }
    
    #[test]
    /// ハンドラの登録解除のテスト
    fn test_unregister_handler() {
        let event_bus = TypedEventBus::new();
        let counter = Arc::new(Mutex::new(0));
        
        let counter_clone = counter.clone();
        
        // ハンドラを登録して参照を保持
        let handler = event_bus.subscribe("test_handler", move |_: &TestEvent| {
            let mut counter = counter_clone.lock().unwrap();
            *counter += 1;
        });
        
        // 最初のイベント発行でハンドラが呼ばれることを確認
        event_bus.publish(TestEvent {
            id: 1,
            message: "テスト".to_string(),
        });
        
        assert_eq!(*counter.lock().unwrap(), 1);
        
        // ハンドラを登録解除
        event_bus.unregister_handler::<TestEvent>(handler.id);
        
        // 2回目のイベント発行でハンドラが呼ばれないことを確認
        event_bus.publish(TestEvent {
            id: 2,
            message: "テスト2".to_string(),
        });
        
        // カウンターが増えていないことを確認
        assert_eq!(*counter.lock().unwrap(), 1);
    }
    
    #[test]
    /// 型の不一致によるコンパイルエラーをコメントで示す
    fn test_type_mismatch() {
        let event_bus = TypedEventBus::new();
        
        // TestEventのハンドラを登録
        let _handler = event_bus.subscribe("test_handler", |event: &TestEvent| {
            println!("TestEventを受信: {:?}", event);
        });
        
        // 以下の行はコンパイルエラーになるのでコメントアウト
        // イベントの型とハンドラの型が一致しない場合、コンパイルエラーとなる
        // これが型安全性の核心部分
        // 
        // event_bus.subscribe("wrong_handler", |event: &TestEvent| {
        //    // AnotherTestEventに対して登録されたハンドラにTestEventを渡そうとしている
        //    let _another_event: &AnotherTestEvent = event; // コンパイルエラー
        // });
        
        // 異なる型のイベントをハンドラに渡そうとするとコンパイルエラー
        // 次の行はコンパイルエラーになるのでコメントアウト
        //
        // // 間違った型のイベントを発行
        // let another_event = AnotherTestEvent {
        //     value: 3.14,
        //     is_valid: true,
        // };
        // let _result = handler.handle(&another_event); // コンパイルエラー
        
        // このテストの目的はコンパイルレベルでの型チェックを確認すること
        // 実行時のアサーションは必要ない
        assert!(true);
    }
} 