# イベントシステム

最終更新日: 2025年4月6日

## 概要

イベントシステムはマインスイーパーゲーム内のコンポーネント間の疎結合なコミュニケーションを実現する仕組みです。型安全なイベント処理によって、コンパイル時の型チェックとランタイムでの高効率な処理を両立しています。

## アーキテクチャ

![イベントシステムアーキテクチャ](../../../assets/img/event_system_architecture.png)

イベントシステムは以下のコンポーネントで構成されています：

1. **Event** - 基本イベントトレイト
2. **TypedEvent** - 型情報を保持する拡張イベントトレイト
3. **EventBus** - イベントの発行と購読を管理
4. **TypedEventBus** - 型安全なイベントバス
5. **EventHandler** - イベントハンドラの基本型
6. **TypedEventHandler** - 型情報を活用したイベントハンドラ

## 型安全なイベントシステム（TypedEvent）の使い方

### 1. イベントの定義

新しいイベントを定義するには、以下の手順に従います：

```rust
// 1. 構造体を定義
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MyCustomEvent {
    pub data: String,
    pub value: i32,
}

// 2. Event基本トレイトを実装
impl_event!(MyCustomEvent, "MyCustom");

// 3. TypedEventトレイトを実装
impl_typed_event!(MyCustomEvent);
```

EventDataへの変換が必要な場合：

```rust
// EventData列挙型に新しいバリアントを追加
pub enum EventData {
    // ... 既存のバリアント ...
    MyCustom(MyCustomEvent),
}

// 変換をサポートするTypedEvent実装
impl_typed_event_with_conversion!(MyCustomEvent, 
    |event: &MyCustomEvent| Some(EventData::MyCustom(event.clone()))
);
```

### 2. イベントの発行

```rust
// イベントバスの取得
let event_bus = resources.get::<TypedEventBus>();

// イベントの作成
let my_event = MyCustomEvent {
    data: "Hello".to_string(),
    value: 42,
};

// 型安全なイベント発行
event_bus.publish(my_event);
```

システムからの発行：

```rust
// TypedEventSystemTraitをインポート
use crate::systems::typed_event_system_trait::TypedEventSystemTrait;

// システム実装内
impl GameSystem {
    fn process(&self, resources: &ResourceManager) {
        // イベント作成
        let event = MyCustomEvent { 
            data: "System generated".to_string(),
            value: 100 
        };
        
        // publish_typed_eventメソッドを使用
        self.publish_typed_event(event, resources);
    }
}
```

### 3. イベントの購読

```rust
// イベントバスの取得
let event_bus = resources.get::<TypedEventBus>();

// 型安全なハンドラを登録
let handler_id = event_bus.subscribe::<MyCustomEvent, _>(
    "my_custom_handler",
    move |event| {
        println!("受信したデータ: {}, 値: {}", event.data, event.value);
        // イベント処理ロジック
    }
);

// ハンドラの登録解除（必要な場合）
event_bus.unregister_handler::<MyCustomEvent>(handler_id);
```

優先度付きハンドラ：

```rust
// 高優先度のハンドラを作成
let high_priority_handler = TypedEventHandler::new(
    "high_priority_handler",
    move |event: &MyCustomEvent| {
        // 処理ロジック
    }
).with_priority(10); // 高い数値ほど優先度が高い

// ハンドラを登録
event_bus.register_handler(high_priority_handler);
```

一度だけ実行するハンドラ：

```rust
// 一度だけ実行されるハンドラを作成
let once_handler = TypedEventHandler::new(
    "once_handler",
    move |event: &MyCustomEvent| {
        // 処理ロジック - 一度だけ実行される
    }
).once();

// ハンドラを登録
event_bus.register_handler(once_handler);
```

### 4. イベント履歴の活用

```rust
// イベントバスの取得
let event_bus = resources.get::<TypedEventBus>();

// 特定型の最後のイベントを取得
if let Some(last_event) = event_bus.get_last_event::<MyCustomEvent>() {
    // 最後に発行されたイベントを使用
    println!("最後のイベント: {:?}", last_event);
}

// 特定型の履歴を取得
let history = event_bus.get_event_history::<MyCustomEvent>();
for event in history {
    // 履歴内の各イベントを処理
}
```

### 5. Best Practices

#### 型安全性の活用

- `TypedEvent`トレイトを実装して型安全性を確保する
- ジェネリックな処理には`TypedEvent`境界を使用

```rust
fn process_events<E: TypedEvent>(events: &[E]) {
    for event in events {
        println!("イベント型: {}", E::type_name());
        // 型安全な処理
    }
}
```

#### パフォーマンスの最適化

- 頻繁に発行されるイベントのクローンを最小化
- ホットパスでのイベント作成を避ける
- 大きなデータはリファレンスで保持

#### デバッグとテスト

- 開発時はイベント履歴を活用
- テスト環境ではモックイベントバスを使用
- イベントのシリアライズ/デシリアライズをテスト

## マイグレーションガイド

### 従来の`Event`からの移行

```rust
// 古い実装
pub struct OldEvent { /* フィールド */ }
impl Event for OldEvent {
    fn name(&self) -> &'static str { "OldEvent" }
    fn as_any(&self) -> &dyn std::any::Any { self }
}

// 新しい実装
pub struct NewEvent { /* フィールド */ }
impl_event!(NewEvent, "NewEvent"); // 基本Event実装
impl_typed_event!(NewEvent);      // TypedEvent拡張
```

## エラー処理

一般的なエラーとその解決方法：

1. **実装の衝突**:
   ```
   error[E0119]: conflicting implementations of trait `TypedEvent` for type `MyEvent`
   ```
   解決策: 重複した実装を削除し、単一のマクロ呼び出しのみを使用

2. **型変換の失敗**:
   ```
   error[E0277]: the trait bound `YourEvent: TypedEvent` is not satisfied
   ```
   解決策: 該当のイベントに`impl_typed_event!`マクロを適用 