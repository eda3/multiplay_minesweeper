/**
 * 型安全なイベントシステムのベンチマーク
 * 
 * イベント発行、購読、履歴管理などの性能評価
 */
#![feature(test)]

extern crate test;
extern crate wasm_multiplayer;

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use test::Bencher;

use wasm_multiplayer::events::typed_event::TypedEvent;
use wasm_multiplayer::events::typed_handler::TypedEventHandler;
use wasm_multiplayer::events::typed_event_bus::TypedEventBus;
use wasm_multiplayer::events::EventData;
use wasm_multiplayer::events::event_trait::Event;
use wasm_multiplayer::events::board_events::CellRevealedEvent;
use wasm_multiplayer::models::cell::CellValue;
use wasm_multiplayer::models::coordinate::Coordinate;

// ベンチマーク用の単純なイベント
#[derive(Debug, Clone)]
struct SimpleEvent {
    id: u64,
}

impl Event for SimpleEvent {
    fn name(&self) -> &'static str {
        "SimpleEvent"
    }
    
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl TypedEvent for SimpleEvent {}

// 大きなデータを持つイベント
#[derive(Debug, Clone)]
struct LargeEvent {
    id: u64,
    data: Vec<u8>, // 大量のデータ
}

impl Event for LargeEvent {
    fn name(&self) -> &'static str {
        "LargeEvent"
    }
    
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl TypedEvent for LargeEvent {}

// 複数のハンドラを登録してシンプルなイベント発行をベンチマーク
#[bench]
fn bench_simple_event_publishing(b: &mut Bencher) {
    // イベントバスの準備
    let event_bus = TypedEventBus::new();
    
    // カウンタ
    let counter = Arc::new(AtomicUsize::new(0));
    
    // 複数のハンドラを登録
    for i in 0..10 {
        let counter_clone = counter.clone();
        let handler = TypedEventHandler::new(
            &format!("handler_{}", i),
            move |_: &SimpleEvent| {
                counter_clone.fetch_add(1, Ordering::Relaxed);
            }
        );
        event_bus.register_handler(handler);
    }
    
    // 計測対象: イベント発行
    b.iter(|| {
        event_bus.publish(SimpleEvent { id: 1 });
    });
    
    // 全てのイベントが処理されたか確認
    assert!(counter.load(Ordering::Relaxed) >= 10);
}

// 大きなデータを持つイベントの発行をベンチマーク
#[bench]
fn bench_large_event_publishing(b: &mut Bencher) {
    // イベントバスの準備
    let event_bus = TypedEventBus::new();
    
    // 大きなデータを準備（約10KB）
    let large_data = vec![0u8; 10 * 1024];
    
    // カウンタ
    let counter = Arc::new(AtomicUsize::new(0));
    let counter_clone = counter.clone();
    
    // ハンドラを登録
    event_bus.subscribe::<LargeEvent, _>(
        "large_handler",
        move |_| {
            counter_clone.fetch_add(1, Ordering::Relaxed);
        }
    );
    
    // 計測対象: 大きなイベント発行
    b.iter(|| {
        event_bus.publish(LargeEvent { 
            id: 1, 
            data: large_data.clone(),
        });
    });
    
    // 全てのイベントが処理されたか確認
    assert!(counter.load(Ordering::Relaxed) > 0);
}

// イベント変換のベンチマーク
#[bench]
fn bench_event_conversion(b: &mut Bencher) {
    // CellRevealedEventを準備
    let event = CellRevealedEvent {
        coord: Coordinate::new(5, 5),
        value: CellValue::Empty(0),
        is_chain: false,
    };
    
    // 計測対象: イベント変換処理
    b.iter(|| {
        let _ = event.to_event_data();
    });
}

// 多数のイベントタイプに対応するイベントバスのベンチマーク
#[bench]
fn bench_multi_type_event_bus(b: &mut Bencher) {
    // イベントバスの準備
    let event_bus = TypedEventBus::new();
    
    // 複数のイベントタイプに対するハンドラを登録
    event_bus.subscribe::<SimpleEvent, _>(
        "simple_handler",
        |_| {}
    );
    
    event_bus.subscribe::<LargeEvent, _>(
        "large_handler",
        |_| {}
    );
    
    event_bus.subscribe::<CellRevealedEvent, _>(
        "cell_handler",
        |_| {}
    );
    
    // 計測対象: 異なるタイプのイベント発行
    b.iter(|| {
        event_bus.publish(SimpleEvent { id: 1 });
        event_bus.publish(LargeEvent { id: 2, data: vec![0; 100] });
        event_bus.publish(CellRevealedEvent {
            coord: Coordinate::new(3, 3),
            value: CellValue::Empty(0),
            is_chain: false,
        });
    });
}

// イベント履歴取得のベンチマーク
#[bench]
fn bench_event_history(b: &mut Bencher) {
    // デバッグモードで履歴を有効化したイベントバス
    let event_bus = TypedEventBus::new().with_debug(true);
    
    // 履歴にイベントを追加
    for i in 0..100 {
        event_bus.publish(SimpleEvent { id: i });
    }
    
    // 計測対象: 特定タイプの履歴取得
    b.iter(|| {
        let history = event_bus.get_event_history_by_type::<SimpleEvent>();
        assert_eq!(history.len(), 100);
    });
}

// ハンドラ登録と解除のベンチマーク
#[bench]
fn bench_handler_registration(b: &mut Bencher) {
    // イベントバスの準備
    let event_bus = Arc::new(TypedEventBus::new());
    
    // 計測対象: ハンドラの登録と解除
    b.iter(|| {
        let event_bus_clone = event_bus.clone();
        
        // ハンドラを登録
        let handler = event_bus_clone.subscribe::<SimpleEvent, _>(
            "temp_handler",
            move |_| {}
        );
        
        // ハンドラを解除
        event_bus_clone.unregister_handler::<SimpleEvent>(handler.id);
    });
}

// ハンドラの優先度処理のベンチマーク
#[bench]
fn bench_handler_priority(b: &mut Bencher) {
    // イベントバスの準備
    let event_bus = TypedEventBus::new();
    
    // 異なる優先度のハンドラを登録（意図的に逆順に）
    for i in (0..10).rev() {
        let handler = TypedEventHandler::new(
            &format!("priority_handler_{}", i),
            move |_: &SimpleEvent| {}
        ).with_priority(i);
        
        event_bus.register_handler(handler);
    }
    
    // 計測対象: 優先度の適用されたイベント処理
    b.iter(|| {
        event_bus.publish(SimpleEvent { id: 1 });
    });
}

// 条件付きイベント発行のベンチマーク
#[bench]
fn bench_conditional_event_dispatch(b: &mut Bencher) {
    // イベントバスの準備
    let event_bus = TypedEventBus::new();
    
    // ハンドラを登録
    event_bus.subscribe::<SimpleEvent, _>(
        "simple_handler",
        |_| {}
    );
    
    // 様々な条件の配列
    let conditions = [true, false, true, false, true];
    let mut index = 0;
    
    // 計測対象: 条件付きイベント発行
    b.iter(|| {
        let condition = conditions[index % conditions.len()];
        index += 1;
        
        if condition {
            event_bus.publish(SimpleEvent { id: 1 });
        }
    });
} 