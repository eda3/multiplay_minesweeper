/**
 * 型付きイベントシステムのテスト
 */
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use crate::events::typed_event::TypedEvent;
use crate::events::typed_handler::TypedEventHandler;
use crate::events::typed_event_bus::TypedEventBus;

// テスト用のイベント
#[derive(Debug, Clone)]
struct TestEvent {
    id: usize,
    message: String,
}

impl TypedEvent for TestEvent {}

// 別のテスト用イベント
#[derive(Debug, Clone)]
struct OtherEvent {
    code: i32,
}

impl TypedEvent for OtherEvent {}

#[test]
fn test_typed_event_bus_basic() {
    // イベントバスの作成
    let event_bus = TypedEventBus::new();
    
    // ハンドラカウンタ
    let counter = Arc::new(AtomicUsize::new(0));
    let counter_clone = counter.clone();
    
    // イベントハンドラを登録
    let handler = event_bus.subscribe::<TestEvent, _>(
        "test_handler",
        move |event| {
            assert_eq!(event.id, 42);
            assert_eq!(event.message, "Hello, typed events!");
            counter_clone.fetch_add(1, Ordering::SeqCst);
        }
    );
    
    // イベントを発行
    event_bus.publish(TestEvent {
        id: 42,
        message: "Hello, typed events!".to_string(),
    });
    
    // ハンドラが呼び出されたことを確認
    assert_eq!(counter.load(Ordering::SeqCst), 1);
    
    // ハンドラを削除
    event_bus.unregister_handler::<TestEvent>(handler.id);
    
    // イベントをもう一度発行
    event_bus.publish(TestEvent {
        id: 42,
        message: "This should not be processed".to_string(),
    });
    
    // カウンタが変わっていないことを確認
    assert_eq!(counter.load(Ordering::SeqCst), 1);
}

#[test]
fn test_typed_event_handler_priority() {
    // イベントバスの作成
    let event_bus = TypedEventBus::new();
    
    // 実行順序を記録する配列
    let order = Arc::new(std::sync::Mutex::new(Vec::new()));
    
    // 低優先度ハンドラを登録
    let order_clone = order.clone();
    let low_priority = TypedEventHandler::new(
        "low_priority",
        move |_: &TestEvent| {
            order_clone.lock().unwrap().push("low");
        }
    ).with_priority(0);
    
    // 中優先度ハンドラを登録
    let order_clone = order.clone();
    let medium_priority = TypedEventHandler::new(
        "medium_priority",
        move |_: &TestEvent| {
            order_clone.lock().unwrap().push("medium");
        }
    ).with_priority(5);
    
    // 高優先度ハンドラを登録
    let order_clone = order.clone();
    let high_priority = TypedEventHandler::new(
        "high_priority",
        move |_: &TestEvent| {
            order_clone.lock().unwrap().push("high");
        }
    ).with_priority(10);
    
    // ハンドラを登録 - 意図的に優先度と逆順に登録
    event_bus.register_handler(low_priority);
    event_bus.register_handler(high_priority);
    event_bus.register_handler(medium_priority);
    
    // イベントを発行
    event_bus.publish(TestEvent {
        id: 1,
        message: "Test priority".to_string(),
    });
    
    // 実行順序を確認
    let execution_order = order.lock().unwrap();
    assert_eq!(*execution_order, vec!["high", "medium", "low"]);
}

#[test]
fn test_typed_event_once() {
    // イベントバスの作成
    let event_bus = TypedEventBus::new();
    
    // カウンタ
    let counter = Arc::new(AtomicUsize::new(0));
    let counter_clone = counter.clone();
    
    // 1回だけ実行するハンドラを登録
    let handler = TypedEventHandler::new(
        "once_handler",
        move |_: &TestEvent| {
            counter_clone.fetch_add(1, Ordering::SeqCst);
        }
    ).once();
    
    event_bus.register_handler(handler);
    
    // イベントを2回発行
    event_bus.publish(TestEvent {
        id: 1,
        message: "First event".to_string(),
    });
    
    event_bus.publish(TestEvent {
        id: 2,
        message: "Second event".to_string(),
    });
    
    // ハンドラは1回だけ実行されたことを確認
    assert_eq!(counter.load(Ordering::SeqCst), 1);
}

#[test]
fn test_typed_event_type_safety() {
    // イベントバスの作成
    let event_bus = TypedEventBus::new();
    
    // TestEventのカウンタ
    let test_counter = Arc::new(AtomicUsize::new(0));
    let test_counter_clone = test_counter.clone();
    
    // OtherEventのカウンタ
    let other_counter = Arc::new(AtomicUsize::new(0));
    let other_counter_clone = other_counter.clone();
    
    // TestEventのハンドラを登録
    event_bus.subscribe::<TestEvent, _>(
        "test_handler",
        move |_| {
            test_counter_clone.fetch_add(1, Ordering::SeqCst);
        }
    );
    
    // OtherEventのハンドラを登録
    event_bus.subscribe::<OtherEvent, _>(
        "other_handler",
        move |_| {
            other_counter_clone.fetch_add(1, Ordering::SeqCst);
        }
    );
    
    // TestEventを発行
    event_bus.publish(TestEvent {
        id: 1,
        message: "Test event".to_string(),
    });
    
    // OtherEventを発行
    event_bus.publish(OtherEvent {
        code: 200,
    });
    
    // 各ハンドラが正しく呼び出されたことを確認
    assert_eq!(test_counter.load(Ordering::SeqCst), 1);
    assert_eq!(other_counter.load(Ordering::SeqCst), 1);
    
    // TestEventを発行しても、OtherEventのハンドラは呼び出されない
    event_bus.publish(TestEvent {
        id: 2,
        message: "Another test event".to_string(),
    });
    
    assert_eq!(test_counter.load(Ordering::SeqCst), 2);
    assert_eq!(other_counter.load(Ordering::SeqCst), 1);
}

#[test]
fn test_typed_event_handler_enabled() {
    // イベントバスの作成
    let event_bus = TypedEventBus::new();
    
    // カウンタ
    let counter = Arc::new(AtomicUsize::new(0));
    let counter_clone = counter.clone();
    
    // ハンドラを登録
    let handler = event_bus.subscribe::<TestEvent, _>(
        "test_handler",
        move |_| {
            counter_clone.fetch_add(1, Ordering::SeqCst);
        }
    );
    
    // イベントを発行
    event_bus.publish(TestEvent {
        id: 1,
        message: "First event".to_string(),
    });
    
    // ハンドラを無効化
    handler.set_enabled(false);
    
    // イベントをもう一度発行
    event_bus.publish(TestEvent {
        id: 2,
        message: "Second event".to_string(),
    });
    
    // ハンドラを再度有効化
    handler.set_enabled(true);
    
    // イベントをもう一度発行
    event_bus.publish(TestEvent {
        id: 3,
        message: "Third event".to_string(),
    });
    
    // ハンドラが正しく実行されたことを確認
    assert_eq!(counter.load(Ordering::SeqCst), 2); // 最初と3回目のみ
} 