/**
 * 型付きイベントトレイト
 * 
 * コンパイル時の型安全性を強化したイベントトレイト。
 * イベントの型情報を保持し、型消去を最小限に抑える。
 */
use std::any::{Any, TypeId};
use std::fmt::Debug;
use crate::events::event_trait::Event;

/// 型情報を保持するイベントトレイト
/// すべてのイベントは自動的にこのトレイトを実装する
pub trait TypedEvent: Event + 'static {
    /// イベント型の静的な識別子を取得
    fn type_id() -> TypeId where Self: Sized {
        TypeId::of::<Self>()
    }
    
    /// イベント型の名前を取得
    fn type_name() -> &'static str where Self: Sized {
        std::any::type_name::<Self>()
    }
    
    /// イベントの具体的な型を文字列で取得
    fn concrete_type_name(&self) -> &'static str {
        std::any::type_name::<Self>()
    }
    
    /// イベントをEventDataに変換
    /// 各イベント型で個別に実装することで型安全性を向上させる
    fn to_event_data(&self) -> Option<crate::events::EventData> {
        // デフォルト実装ではNoneを返す
        // 具体的なイベント型でオーバーライドする
        None
    }
    
    /// イベントのタイムスタンプを取得
    /// デフォルトでは0を返す（タイムスタンプを持たないイベント用）
    fn timestamp(&self) -> u64 {
        0
    }
}

/// TypedEventトレイトの自動実装マクロ
/// 特定の型に対してトレイトを実装する
#[macro_export]
macro_rules! impl_typed_event {
    ($type:ty) => {
        impl TypedEvent for $type {}
    };
}

/// TypedEventトレイトの自動実装マクロ（to_event_dataをカスタマイズ）
#[macro_export]
macro_rules! impl_typed_event_with_conversion {
    ($type:ty, $conversion:expr) => {
        impl TypedEvent for $type {
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