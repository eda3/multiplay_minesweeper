/**
 * 型付きイベントトレイト
 * 
 * コンパイル時の型安全性を強化したイベントトレイト。
 * イベントの型情報を保持し、型消去を最小限に抑える。
 */
use std::any::{Any, TypeId};
use std::fmt::Debug;
// イベントデータのインポートはまだ必要ないのでコメントアウト
// use crate::events::event_data::EventData;

/// 型情報を持つイベントトレイト
/// すべてのイベントタイプはこのトレイトを実装する必要があります
pub trait TypedEvent: std::fmt::Debug + Clone + Send + Sync + 'static {
    /// イベントタイプ名を返す（関連関数）
    fn event_type() -> &'static str
    where 
        Self: Sized;
    
    /// イベント型の名前を取得
    fn type_name() -> &'static str where Self: Sized {
        std::any::type_name::<Self>()
    }
    
    /// イベントをEventDataに変換
    /// 各イベント型で個別に実装することで型安全性を向上させる
    fn to_event_data(&self) -> Option<crate::events::EventData> {
        // デフォルト実装はNoneを返す
        // 必要に応じて具体的なイベント型でオーバーライドする
        None
    }
    
    /// イベントのタイムスタンプを取得
    /// デフォルトでは0を返す（タイムスタンプを持たないイベント用）
    fn timestamp(&self) -> u64 {
        // TimestampedEvent実装がある場合はその値を返す
        // ここではデフォルト値を返す
        0
    }
    
    /// Any型へのダウンキャスト用のメソッド
    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// タイプIDに基づくイベントハンドラ識別子
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TypedHandlerId {
    /// イベントタイプ名
    pub event_type: String,
    /// ハンドラ名
    pub handler_name: String,
    /// 一意のID
    pub id: u64,
}

/// 型情報付きイベントのトレイト実装マクロ
#[macro_export]
macro_rules! impl_typed_event {
    ($type:ty) => {
        impl $crate::events::typed_event::TypedEvent for $type {
            fn event_type() -> &'static str {
                stringify!($type)
            }
        }
    };
}

/// TypedEventトレイトの自動実装マクロ（to_event_dataをカスタマイズ）
#[macro_export]
macro_rules! impl_typed_event_with_conversion {
    ($type:ty, $conversion:expr) => {
        impl TypedEvent for $type {
            fn event_type() -> &'static str {
                stringify!($type)
            }
            
            fn to_event_data(&self) -> Option<crate::events::EventData> {
                $conversion(self)
            }
        }
    };
}

// 注意: ブランケット実装は削除しました
// デフォルト実装を使いたい場合は個別にimpl_typed_eventマクロを使用してください

/// 型付きイベントの識別子
/// 型の情報とイベントIDを保持する
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TypedEventId {
    /// イベントの型ID
    pub type_id: TypeId,
    /// イベントの一意なID
    pub id: u64,
}

impl TypedEventId {
    /// 新しいイベントIDを作成
    pub fn new<E: TypedEvent>(id: u64) -> Self {
        Self {
            type_id: TypeId::of::<E>(),
            id,
        }
    }
    
    /// イベントIDが特定の型に関連しているか確認
    pub fn is_type<E: TypedEvent>(&self) -> bool {
        self.type_id == TypeId::of::<E>()
    }
    
    /// イベントの型名を取得（デバッグ用）
    pub fn type_name(&self) -> &'static str {
        // 警告: 実行時に型名を解決するのは非効率
        // デバッグ用途のみに使用する
        "<type-erased>"
    }
}

/// イベントハンドラの識別子
/// TypedEventIdと同じ構造だが、意味的に区別するために別の型として定義
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct HandlerId {
    /// ハンドラが処理するイベントの型ID
    pub type_id: TypeId,
    /// ハンドラの一意なID
    pub id: u64,
}

impl HandlerId {
    /// 新しいハンドラIDを作成
    pub fn new<E: TypedEvent>(id: u64) -> Self {
        Self {
            type_id: TypeId::of::<E>(),
            id,
        }
    }
    
    /// ハンドラIDが特定の型のイベントを処理するか確認
    pub fn handles_type<E: TypedEvent>(&self) -> bool {
        self.type_id == TypeId::of::<E>()
    }
}

// IDジェネレータ
static NEXT_ID: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);

/// 一意なIDを生成
pub fn generate_id() -> u64 {
    NEXT_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
}

/// タイムスタンプ付きイベントの共通トレイト
pub trait TimestampedEvent: TypedEvent {
    /// イベントのタイムスタンプを取得
    fn timestamp(&self) -> u64;
}

/// _timestampフィールドを持つ型に対するTimestampedEventトレイトの簡易実装マクロ
#[macro_export]
macro_rules! impl_timestamped_event {
    ($type:ty) => {
        impl $crate::events::typed_event::TimestampedEvent for $type {
            fn timestamp(&self) -> u64 {
                self._timestamp
            }
        }
    };
} 