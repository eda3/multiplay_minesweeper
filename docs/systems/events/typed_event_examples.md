# TypedEvent 実装サンプル集

最終更新日: 2025年4月6日

このドキュメントでは、様々なユースケースにおける`TypedEvent`の実装例を紹介します。

## 基本的な実装パターン

### 1. 単純なイベント

最も基本的な実装パターンです。イベントデータのみを持ち、特別な変換を必要としないケース。

```rust
use serde::{Serialize, Deserialize};
use crate::events::event_trait::Event;
use crate::events::typed_event::TypedEvent;
use crate::impl_event;
use crate::impl_typed_event;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerConnectedEvent {
    pub player_id: String,
    pub username: String,
    pub connection_time: u64,
}

// Event基本トレイトの実装
impl_event!(PlayerConnectedEvent, "PlayerConnected");

// TypedEventトレイトの実装（デフォルト変換）
impl_typed_event!(PlayerConnectedEvent);
```

### 2. EventDataへの変換をサポートするイベント

`EventData`列挙型へ変換できるようにする実装パターンです。

```rust
use serde::{Serialize, Deserialize};
use crate::events::event_trait::Event;
use crate::events::typed_event::TypedEvent;
use crate::events::EventData;
use crate::impl_event;
use crate::impl_typed_event_with_conversion;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CellUpdatedEvent {
    pub x: u32,
    pub y: u32,
    pub new_value: i32,
}

// Event基本トレイトの実装
impl_event!(CellUpdatedEvent, "CellUpdated");

// 変換機能付きのTypedEvent実装
impl_typed_event_with_conversion!(CellUpdatedEvent, 
    |event: &CellUpdatedEvent| Some(EventData::CellUpdated(event.clone()))
);
```

## 高度な実装パターン

### 1. ジェネリックイベント

型パラメータを持つイベントの実装例です。

```rust
use std::marker::PhantomData;
use serde::{Serialize, Deserialize};
use crate::events::event_trait::Event;
use crate::events::typed_event::TypedEvent;
use crate::impl_event;

#[derive(Debug, Clone)]
pub struct ValueChangedEvent<T: Clone + 'static> {
    pub old_value: T,
    pub new_value: T,
    pub source: String,
    _phantom: PhantomData<T>,
}

// Event基本トレイトの手動実装
impl<T: Clone + std::fmt::Debug + 'static> Event for ValueChangedEvent<T> {
    fn name(&self) -> &'static str {
        "ValueChanged"
    }
    
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

// TypedEventトレイトの手動実装
impl<T: Clone + std::fmt::Debug + 'static> TypedEvent for ValueChangedEvent<T> {
    // デフォルト実装を利用
}

// 使用例
let score_change = ValueChangedEvent::<i32> {
    old_value: 100,
    new_value: 150,
    source: "player_action".to_string(),
    _phantom: PhantomData,
};
```

### 2. 継承関係を持つイベント

基本イベントと派生イベントの関係を表現する実装例です。

```rust
use serde::{Serialize, Deserialize};
use crate::events::event_trait::Event;
use crate::events::typed_event::TypedEvent;
use crate::impl_event;
use crate::impl_typed_event;

// 基本イベント
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaseNetworkEvent {
    pub timestamp: u64,
    pub connection_id: String,
}

// 派生イベント
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkDataReceivedEvent {
    pub base: BaseNetworkEvent,
    pub data_type: String,
    pub size: usize,
}

// Event基本トレイトの実装
impl_event!(BaseNetworkEvent, "BaseNetwork");
impl_event!(NetworkDataReceivedEvent, "NetworkDataReceived");

// TypedEventトレイトの実装
impl_typed_event!(BaseNetworkEvent);
impl_typed_event!(NetworkDataReceivedEvent);

// 使用例：派生イベントから基本情報を取得
fn process_network_event(event: &NetworkDataReceivedEvent) {
    let connection = &event.base.connection_id;
    println!("接続ID: {}, データ型: {}", connection, event.data_type);
}
```

### 3. カスタムマクロを使った実装

繰り返しのコードを減らすためのカスタムマクロ例です。

```rust
// カスタムマクロの定義
#[macro_export]
macro_rules! define_game_event {
    ($event_name:ident, $event_type:expr, $($field_name:ident: $field_type:ty),*) => {
        #[derive(Debug, Clone, Serialize, Deserialize)]
        pub struct $event_name {
            pub timestamp: u64,
            $(pub $field_name: $field_type,)*
        }
        
        impl_event!($event_name, $event_type);
        impl_typed_event!($event_name);
        
        impl $event_name {
            pub fn new($($field_name: $field_type,)*) -> Self {
                Self {
                    timestamp: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs(),
                    $($field_name,)*
                }
            }
        }
    }
}

// マクロを使ったイベント定義
define_game_event!(ScoreChangedEvent, "ScoreChanged", 
    player_id: String, 
    old_score: i32, 
    new_score: i32, 
    reason: String
);

define_game_event!(AchievementUnlockedEvent, "AchievementUnlocked", 
    player_id: String, 
    achievement_id: String, 
    points: u32
);
```

## イベント間の変換パターン

既存のイベントから新しいイベントを生成するパターンです。

```rust
use serde::{Serialize, Deserialize};
use crate::events::event_trait::Event;
use crate::events::typed_event::TypedEvent;
use crate::impl_event;
use crate::impl_typed_event;

// 基本イベント
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CellClickEvent {
    pub x: u32,
    pub y: u32,
    pub is_right_click: bool,
}

// 派生イベント
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CellRevealEvent {
    pub x: u32,
    pub y: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CellFlagEvent {
    pub x: u32,
    pub y: u32,
    pub is_flagged: bool,
}

// トレイト実装
impl_event!(CellClickEvent, "CellClick");
impl_event!(CellRevealEvent, "CellReveal");
impl_event!(CellFlagEvent, "CellFlag");

impl_typed_event!(CellClickEvent);
impl_typed_event!(CellRevealEvent);
impl_typed_event!(CellFlagEvent);

// イベント変換ロジック
impl CellClickEvent {
    pub fn to_reveal_event(&self) -> Option<CellRevealEvent> {
        if !self.is_right_click {
            Some(CellRevealEvent {
                x: self.x,
                y: self.y,
            })
        } else {
            None
        }
    }
    
    pub fn to_flag_event(&self) -> Option<CellFlagEvent> {
        if self.is_right_click {
            Some(CellFlagEvent {
                x: self.x,
                y: self.y,
                is_flagged: true, // トグル動作は呼び出し側で処理
            })
        } else {
            None
        }
    }
}

// イベント処理システムでの使用例
fn process_cell_click(event: &CellClickEvent, resources: &ResourceManager) {
    // クリックイベントを適切な派生イベントに変換して発行
    if let Some(reveal_event) = event.to_reveal_event() {
        resources.get::<TypedEventBus>().publish(reveal_event);
    }
    
    if let Some(flag_event) = event.to_flag_event() {
        resources.get::<TypedEventBus>().publish(flag_event);
    }
}
```

## テスト用モックイベント

テストで使用する特別なイベント実装例です。

```rust
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use crate::events::event_trait::Event;
use crate::events::typed_event::TypedEvent;

// テスト用イベント
#[derive(Debug, Clone)]
pub struct TestEvent {
    pub id: usize,
    pub processed: Arc<AtomicBool>,
}

// 手動実装
impl Event for TestEvent {
    fn name(&self) -> &'static str {
        "TestEvent"
    }
    
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl TypedEvent for TestEvent {}

// テストでの使用例
#[test]
fn test_event_processing() {
    let event_bus = TypedEventBus::new();
    
    // 処理フラグ
    let processed = Arc::new(AtomicBool::new(false));
    let processed_clone = processed.clone();
    
    // テストイベント
    let test_event = TestEvent {
        id: 1,
        processed: processed.clone(),
    };
    
    // ハンドラ登録
    event_bus.subscribe::<TestEvent, _>(
        "test_handler",
        move |event| {
            event.processed.store(true, Ordering::SeqCst);
        }
    );
    
    // イベント発行
    event_bus.publish(test_event);
    
    // イベントが処理されたか確認
    assert!(processed_clone.load(Ordering::SeqCst));
}
```

## パフォーマンス最適化テクニック

### イベントのプーリング

頻繁に生成されるイベントの割り当てを最適化する例です。

```rust
use std::sync::Mutex;
use std::collections::VecDeque;
use crate::events::event_trait::Event;
use crate::events::typed_event::TypedEvent;
use crate::impl_event;
use crate::impl_typed_event;

// プールするイベント
#[derive(Debug, Clone)]
pub struct PositionUpdateEvent {
    pub entity_id: u64,
    pub x: f32,
    pub y: f32,
}

impl_event!(PositionUpdateEvent, "PositionUpdate");
impl_typed_event!(PositionUpdateEvent);

// イベントプール
pub struct PositionEventPool {
    pool: Mutex<VecDeque<PositionUpdateEvent>>,
}

impl PositionEventPool {
    pub fn new(capacity: usize) -> Self {
        let mut pool = VecDeque::with_capacity(capacity);
        
        // プールを事前に埋める
        for _ in 0..capacity {
            pool.push_back(PositionUpdateEvent {
                entity_id: 0,
                x: 0.0,
                y: 0.0,
            });
        }
        
        Self {
            pool: Mutex::new(pool),
        }
    }
    
    pub fn get(&self, entity_id: u64, x: f32, y: f32) -> PositionUpdateEvent {
        let mut pool = self.pool.lock().unwrap();
        
        // プールからイベントを取得または新規作成
        match pool.pop_front() {
            Some(mut event) => {
                event.entity_id = entity_id;
                event.x = x;
                event.y = y;
                event
            },
            None => PositionUpdateEvent {
                entity_id,
                x,
                y,
            }
        }
    }
    
    pub fn recycle(&self, event: PositionUpdateEvent) {
        let mut pool = self.pool.lock().unwrap();
        pool.push_back(event);
    }
}

// 使用例
let event_pool = PositionEventPool::new(100); // 100個のイベントをプール

// イベント取得と発行
let event = event_pool.get(entity_id, position.x, position.y);
event_bus.publish(event.clone());

// 使用後にプールに戻す
event_pool.recycle(event);
``` 