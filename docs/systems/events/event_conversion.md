# イベント変換メカニズム

最終更新日: 2025年4月6日

## 概要

イベント変換メカニズムは、型安全なイベントシステムにおいて、型付きイベント（`TypedEvent`）と従来の型消去されたイベントデータ（`EventData`）間の変換を行う仕組みです。このドキュメントでは、`to_event_data`メソッドを中心に、イベント変換の実装方法と活用パターンを解説します。

## 変換の基本原理

### 1. `to_event_data`メソッド

`TypedEvent`トレイトは、イベントを`EventData`型に変換するためのメソッドを提供します：

```rust
pub trait TypedEvent: Event + 'static {
    // ... 他のメソッド ...
    
    /// イベントをEventDataに変換
    fn to_event_data(&self) -> Option<EventData> {
        None // デフォルト実装
    }
}
```

このメソッドにより、型情報を保持したまま処理できる`TypedEvent`オブジェクトを、レガシーシステムや外部インターフェースで使用される`EventData`列挙型に変換できます。

### 2. 変換の方向

イベント変換は主に2つの方向で行われます：

1. **TypedEvent → EventData** (`to_event_data`メソッド)
   - 型安全なイベントを型消去版に変換
   - レガシーシステムとの互換性を確保
   - ネットワーク送信やシリアライズ前の準備

2. **EventData → TypedEvent** (明示的なダウンキャストまたは専用メソッド)
   - 型消去されたイベントから特定の型へ復元
   - 外部からのイベントデータを型安全に処理

## 実装パターン

### マクロを使った実装

`impl_typed_event_with_conversion`マクロは、変換ロジックを簡潔に実装します：

```rust
// EventData変換をサポートするTypedEvent実装
impl_typed_event_with_conversion!(CellRevealedEvent, 
    |event: &CellRevealedEvent| Some(EventData::CellRevealed(event.clone()))
);
```

このマクロは以下の実装を展開します：

```rust
impl TypedEvent for CellRevealedEvent {
    fn to_event_data(&self) -> Option<EventData> {
        Some(EventData::CellRevealed(self.clone()))
    }
}
```

### 手動実装

複雑な変換ロジックが必要な場合は、手動で実装することもできます：

```rust
impl TypedEvent for ComplexEvent {
    fn to_event_data(&self) -> Option<EventData> {
        if self.should_convert() {
            // 条件に基づいて変換
            Some(EventData::Complex(ComplexEventData {
                id: self.id,
                data: self.processed_data(),
                timestamp: std::time::SystemTime::now(),
            }))
        } else {
            None // 変換しない
        }
    }
}
```

## 変換のユースケース

### 1. イベントバスの相互運用

新旧のイベントバス間でのイベント転送：

```rust
// 型安全なイベントバスで発行されたイベントをレガシーイベントバスにも転送
impl TypedEventBus {
    pub fn forward_to_legacy(&self, legacy_bus: &EventBus) {
        self.add_global_processor(move |event: &dyn TypedEvent| {
            if let Some(event_data) = event.to_event_data() {
                legacy_bus.publish(event_data);
            }
        });
    }
}
```

### 2. ネットワーク送信

イベントをネットワーク経由で送信する際の変換：

```rust
pub fn send_event_to_network<E: TypedEvent>(event: &E, network: &NetworkManager) {
    if let Some(event_data) = event.to_event_data() {
        // イベントデータをシリアライズ
        let serialized = serde_json::to_string(&event_data).unwrap();
        // ネットワーク送信
        network.send_message("event", &serialized);
    }
}
```

### 3. イベント履歴とデバッグ

イベント履歴を記録し、デバッグに活用：

```rust
pub struct EventLogger {
    history: Vec<EventData>,
}

impl EventLogger {
    pub fn log<E: TypedEvent>(&mut self, event: &E) {
        if let Some(event_data) = event.to_event_data() {
            println!("イベント記録: {:?}", event_data);
            self.history.push(event_data);
        }
    }
    
    pub fn get_history(&self) -> &[EventData] {
        &self.history
    }
}
```

## EventDataからTypedEventへの変換

逆方向の変換（型消去から型付きへ）は、通常以下の方法で行います：

### 1. プロセッサによる型チェックとダウンキャスト

```rust
event_bus.add_processor::<EventData, _>(
    "data_processor",
    move |event: &EventData| {
        match event {
            EventData::CellRevealed(cell_event) => {
                // 型情報が復元された特定型のイベントを処理
                process_cell_revealed(cell_event);
            },
            EventData::MineExploded(mine_event) => {
                process_mine_exploded(mine_event);
            },
            // 他のバリアント...
            _ => {}
        }
    }
);
```

### 2. イベント拡張メソッドによる変換

```rust
impl EventData {
    pub fn as_cell_revealed(&self) -> Option<&CellRevealedEvent> {
        match self {
            EventData::CellRevealed(event) => Some(event),
            _ => None,
        }
    }
    
    pub fn as_mine_exploded(&self) -> Option<&MineExplodedEvent> {
        match self {
            EventData::MineExploded(event) => Some(event),
            _ => None,
        }
    }
    
    // 他のイベント型に対しても同様のメソッドを実装...
}

// 使用例
fn process_event(event_data: &EventData) {
    if let Some(cell_event) = event_data.as_cell_revealed() {
        // 型情報が復元されたCellRevealedEventとして処理
    }
}
```

## 型安全性の確保

イベント変換における型安全性を確保するためのベストプラクティス：

1. **網羅的なパターンマッチング**: `EventData`列挙型の全バリアントを処理
2. **エラー処理**: 変換に失敗した場合の適切なエラーハンドリング
3. **単体テスト**: 変換ロジックの正確性を確認する単体テスト

```rust
#[test]
fn test_event_conversion() {
    // イベント作成
    let original = CellRevealedEvent {
        coord: Coordinate::new(5, 10),
        value: CellValue::Empty,
        is_chain: false,
    };
    
    // TypedEvent → EventData変換
    let event_data = original.to_event_data().expect("変換に失敗しました");
    
    // EventData → 特定の型への変換
    match event_data {
        EventData::CellRevealed(converted) => {
            // 変換の正確性を検証
            assert_eq!(converted.coord.x, 5);
            assert_eq!(converted.coord.y, 10);
            assert_eq!(converted.value, CellValue::Empty);
            assert_eq!(converted.is_chain, false);
        },
        _ => panic!("間違ったイベント型に変換されました"),
    }
}
```

## パフォーマンスと最適化

### 1. クローンの最小化

```rust
// クローンを避けるためにリファレンスを使用
impl_typed_event_with_conversion!(LargeEvent, 
    |event: &LargeEvent| {
        // 新しいインスタンスを作成し、必要なデータのみをコピー
        Some(EventData::LargeEvent(LargeEventData {
            id: event.id,             // 単純な型はコピー
            summary: event.summary(), // 導出データを使用
            // 大きなデータは参照または遅延ロードを検討
        }))
    }
);
```

### 2. 遅延変換

イベントの変換を必要になるまで遅延させる：

```rust
pub struct LazyEventData<E: TypedEvent> {
    source: E,
    converted: Option<EventData>,
}

impl<E: TypedEvent> LazyEventData<E> {
    pub fn new(event: E) -> Self {
        Self {
            source: event,
            converted: None,
        }
    }
    
    pub fn get_event_data(&mut self) -> Option<&EventData> {
        if self.converted.is_none() {
            self.converted = self.source.to_event_data();
        }
        self.converted.as_ref()
    }
    
    pub fn source(&self) -> &E {
        &self.source
    }
}
```

## まとめ

イベント変換メカニズムは、型安全なイベントシステムとレガシーコードや外部システムとの間の橋渡しとして機能します。`to_event_data`メソッドと`TypedEvent`トレイトを活用することで、型安全性と互換性の両方を確保できます。

イベント変換を実装する際は、型安全性、パフォーマンス、エラー処理に注意し、適切なテストを行うことが重要です。 