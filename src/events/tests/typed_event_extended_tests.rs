/**
 * 型付きイベントシステムの拡張テスト
 * 
 * 型安全性、エッジケース、エラー処理に焦点を当てた詳細なテスト
 */
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::any::TypeId;
use std::collections::HashSet;
use std::time::Duration;

use crate::events::typed_event::{TypedEvent, TypedEventId, HandlerId, generate_id};
use crate::events::typed_handler::TypedEventHandler;
use crate::events::typed_event_bus::TypedEventBus;
use crate::events::EventData;
use crate::events::board_events::{
    CellRevealedEvent, MineExplodedEvent, BoardInitializedEvent,
    MultipleCellsRevealedEvent, GameProgressEvent
};
use crate::models::cell::CellValue;
use crate::models::coordinate::Coordinate;

// 型IDテスト用のイベント
#[derive(Debug, Clone)]
struct TypeIdTestEvent {
    id: u64,
}

impl crate::events::event_trait::Event for TypeIdTestEvent {
    fn name(&self) -> &'static str {
        "TypeIdTestEvent"
    }
    
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl TypedEvent for TypeIdTestEvent {}

#[test]
fn test_typed_event_id_type_checking() {
    // TypedEventIdの型チェック機能のテスト
    
    // 異なるイベント型のID
    let event_id_1 = TypedEventId::new::<TypeIdTestEvent>(1);
    let event_id_2 = TypedEventId::new::<CellRevealedEvent>(2);
    
    // 同じ型のIDが一致することを確認
    let event_id_3 = TypedEventId::new::<TypeIdTestEvent>(3);
    
    // 型IDが正しく機能することを確認
    assert!(event_id_1.is_type::<TypeIdTestEvent>());
    assert!(!event_id_1.is_type::<CellRevealedEvent>());
    
    assert!(event_id_2.is_type::<CellRevealedEvent>());
    assert!(!event_id_2.is_type::<TypeIdTestEvent>());
    
    assert!(event_id_3.is_type::<TypeIdTestEvent>());
    
    // TypeIDが一致することを確認
    assert_eq!(event_id_1.type_id, TypeId::of::<TypeIdTestEvent>());
    assert_eq!(event_id_3.type_id, TypeId::of::<TypeIdTestEvent>());
    assert_eq!(event_id_2.type_id, TypeId::of::<CellRevealedEvent>());
    
    // IDが異なることを確認
    assert_ne!(event_id_1.id, event_id_3.id);
}

#[test]
fn test_handler_id_type_matching() {
    // HandlerIdの型マッチング機能のテスト
    
    // 異なるイベント型のハンドラID
    let handler_id_1 = HandlerId::new::<TypeIdTestEvent>(1);
    let handler_id_2 = HandlerId::new::<CellRevealedEvent>(2);
    
    // 型チェック機能をテスト
    assert!(handler_id_1.handles_type::<TypeIdTestEvent>());
    assert!(!handler_id_1.handles_type::<CellRevealedEvent>());
    
    assert!(handler_id_2.handles_type::<CellRevealedEvent>());
    assert!(!handler_id_2.handles_type::<TypeIdTestEvent>());
}

#[test]
fn test_typed_event_static_methods() {
    // 静的メソッドのテスト
    
    // 型IDが正しく取得できることを確認
    assert_eq!(TypeIdTestEvent::type_id(), TypeId::of::<TypeIdTestEvent>());
    assert_eq!(CellRevealedEvent::type_id(), TypeId::of::<CellRevealedEvent>());
    
    // 型名が正しく取得できることを確認
    assert!(TypeIdTestEvent::type_name().contains("TypeIdTestEvent"));
    assert!(CellRevealedEvent::type_name().contains("CellRevealedEvent"));
}

#[test]
fn test_typed_event_concrete_type_name() {
    // インスタンスメソッドのテスト
    
    let event1 = TypeIdTestEvent { id: 42 };
    let event2 = CellRevealedEvent {
        coord: Coordinate::new(1, 1),
        value: CellValue::Empty(0),
        is_chain: false,
    };
    
    // 具体的な型名が取得できることを確認
    assert!(event1.concrete_type_name().contains("TypeIdTestEvent"));
    assert!(event2.concrete_type_name().contains("CellRevealedEvent"));
}

#[test]
fn test_id_generation() {
    // ID生成関数のテスト
    
    // 生成されたIDがユニークであることを確認
    let mut ids = HashSet::new();
    for _ in 0..100 {
        let id = generate_id();
        assert!(!ids.contains(&id), "重複するIDが生成されました: {}", id);
        ids.insert(id);
    }
}

// イベント変換テスト用のカスタムイベント
#[derive(Debug, Clone)]
struct ConditionalConversionEvent {
    id: u64,
    should_convert: bool,
}

impl crate::events::event_trait::Event for ConditionalConversionEvent {
    fn name(&self) -> &'static str {
        "ConditionalConversionEvent"
    }
    
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

// 条件付き変換をサポートするTypedEvent実装
impl TypedEvent for ConditionalConversionEvent {
    fn to_event_data(&self) -> Option<EventData> {
        if self.should_convert {
            // このテスト用に既存のイベントタイプを流用
            Some(EventData::GameProgress(GameProgressEvent {
                revealed_count: self.id as u32,
                remaining_non_mine_cells: 100,
                completion_rate: self.id as f32 / 100.0,
            }))
        } else {
            None // 変換しない
        }
    }
}

#[test]
fn test_conditional_event_conversion() {
    // 条件付きイベント変換のテスト
    
    // 変換するイベント
    let event1 = ConditionalConversionEvent {
        id: 42,
        should_convert: true,
    };
    
    // 変換しないイベント
    let event2 = ConditionalConversionEvent {
        id: 43,
        should_convert: false,
    };
    
    // 変換が成功することを確認
    let converted1 = event1.to_event_data();
    assert!(converted1.is_some(), "変換可能なイベントが変換されませんでした");
    
    if let Some(EventData::GameProgress(progress)) = converted1 {
        assert_eq!(progress.revealed_count, 42);
        assert_eq!(progress.completion_rate, 42.0 / 100.0);
    } else {
        panic!("イベントが正しく変換されませんでした");
    }
    
    // 変換が失敗することを確認
    let converted2 = event2.to_event_data();
    assert!(converted2.is_none(), "変換不可能なイベントが変換されました");
}

#[test]
fn test_large_event_batch() {
    // 大量イベント処理のテスト
    
    // イベントバスの作成
    let event_bus = TypedEventBus::new();
    
    // カウンタ
    let counter = Arc::new(AtomicUsize::new(0));
    let counter_clone = counter.clone();
    
    // イベントハンドラを登録
    event_bus.subscribe::<TypeIdTestEvent, _>(
        "batch_handler",
        move |_| {
            counter_clone.fetch_add(1, Ordering::SeqCst);
            // 処理時間をシミュレート
            std::thread::sleep(Duration::from_micros(10));
        }
    );
    
    // 大量のイベントを発行
    const EVENT_COUNT: usize = 100;
    for i in 0..EVENT_COUNT {
        event_bus.publish(TypeIdTestEvent { id: i as u64 });
    }
    
    // 少し待って全てのイベントが処理されるのを待つ
    std::thread::sleep(Duration::from_millis(200));
    
    // 全てのイベントが処理されたことを確認
    assert_eq!(counter.load(Ordering::SeqCst), EVENT_COUNT);
}

// エラー検出用の特殊イベント
#[derive(Debug, Clone)]
struct ErrorDetectionEvent {
    should_panic: bool,
}

impl crate::events::event_trait::Event for ErrorDetectionEvent {
    fn name(&self) -> &'static str {
        "ErrorDetectionEvent"
    }
    
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl TypedEvent for ErrorDetectionEvent {}

#[test]
fn test_error_handling_in_handlers() {
    // ハンドラ内のエラー処理テスト
    
    // イベントバスの作成
    let event_bus = TypedEventBus::new();
    
    // カウンタ
    let error_counter = Arc::new(AtomicUsize::new(0));
    let success_counter = Arc::new(AtomicUsize::new(0));
    
    let error_counter_clone = error_counter.clone();
    let success_counter_clone = success_counter.clone();
    
    // エラーハンドラを登録
    event_bus.subscribe::<ErrorDetectionEvent, _>(
        "error_handler",
        move |event| {
            if event.should_panic {
                error_counter_clone.fetch_add(1, Ordering::SeqCst);
                // エラーをシミュレート（パニックはしないが、カウントする）
            } else {
                success_counter_clone.fetch_add(1, Ordering::SeqCst);
            }
        }
    );
    
    // テストイベントを発行
    event_bus.publish(ErrorDetectionEvent { should_panic: true });
    event_bus.publish(ErrorDetectionEvent { should_panic: false });
    event_bus.publish(ErrorDetectionEvent { should_panic: true });
    event_bus.publish(ErrorDetectionEvent { should_panic: false });
    
    // 各カウンタの値を確認
    assert_eq!(error_counter.load(Ordering::SeqCst), 2);
    assert_eq!(success_counter.load(Ordering::SeqCst), 2);
}

// イベント履歴検証用の複数型イベント
#[derive(Debug, Clone)]
struct HistoryTestEvent {
    value: String,
}

impl crate::events::event_trait::Event for HistoryTestEvent {
    fn name(&self) -> &'static str {
        "HistoryTestEvent"
    }
    
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl TypedEvent for HistoryTestEvent {
    fn to_event_data(&self) -> Option<EventData> {
        // このテスト用に既存のイベントタイプを流用
        Some(EventData::GameProgress(GameProgressEvent {
            revealed_count: 1,
            remaining_non_mine_cells: 99,
            completion_rate: 0.01,
        }))
    }
}

#[test]
fn test_typed_event_history() {
    // イベント履歴の型安全な取得テスト
    
    // イベントバスの作成（デバッグモードでヒストリを有効化）
    let event_bus = TypedEventBus::new().with_debug(true);
    
    // 異なる型のイベントを発行
    event_bus.publish(TypeIdTestEvent { id: 1 });
    event_bus.publish(HistoryTestEvent { value: "test".to_string() });
    event_bus.publish(TypeIdTestEvent { id: 2 });
    event_bus.publish(TypeIdTestEvent { id: 3 });
    
    // 型別の履歴を取得
    let type_id_events = event_bus.get_event_history_by_type::<TypeIdTestEvent>();
    let history_events = event_bus.get_event_history_by_type::<HistoryTestEvent>();
    
    // 各型の履歴数を確認
    assert_eq!(type_id_events.len(), 3);
    assert_eq!(history_events.len(), 1);
    
    // 最後のイベントを取得
    if let Some(last_event) = event_bus.get_last_event::<TypeIdTestEvent>() {
        assert_eq!(last_event.id, 3);
    } else {
        panic!("最後のTypeIdTestEventが取得できませんでした");
    }
    
    if let Some(last_history) = event_bus.get_last_event::<HistoryTestEvent>() {
        assert_eq!(last_history.value, "test");
    } else {
        panic!("最後のHistoryTestEventが取得できませんでした");
    }
}

// 未登録ハンドラ用のイベント
#[derive(Debug, Clone)]
struct UnregisteredEvent {
    data: u32,
}

impl crate::events::event_trait::Event for UnregisteredEvent {
    fn name(&self) -> &'static str {
        "UnregisteredEvent"
    }
    
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl TypedEvent for UnregisteredEvent {}

#[test]
fn test_unregistered_event_handling() {
    // 未登録イベント型のハンドリングテスト
    
    let event_bus = TypedEventBus::new();
    
    // 何も登録せずにイベントを発行
    // これは内部でエラーにならずに無視されるべき
    event_bus.publish(UnregisteredEvent { data: 42 });
    
    // グローバルハンドラカウンタ
    let global_counter = Arc::new(AtomicUsize::new(0));
    let global_counter_clone = global_counter.clone();
    
    // 全てのイベントを処理するグローバルハンドラ
    event_bus.add_global_processor(move |_| {
        global_counter_clone.fetch_add(1, Ordering::SeqCst);
    });
    
    // さらにイベントを発行
    event_bus.publish(UnregisteredEvent { data: 100 });
    
    // グローバルハンドラが呼び出されたことを確認
    assert_eq!(global_counter.load(Ordering::SeqCst), 1);
} 