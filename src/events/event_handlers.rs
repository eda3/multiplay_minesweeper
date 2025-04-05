/**
 * イベントハンドラーの定義
 * 
 * 型安全なイベント処理のための機能を提供
 */
use std::any::TypeId;
use std::marker::PhantomData;
use crate::events::event_trait::Event;
use crate::entities::EntityManager;

/// イベントコールバックトレイト
pub trait EventCallback: 'static {
    /// イベントの型ID
    fn event_type_id(&self) -> TypeId;
    
    /// イベントを処理
    fn call(&self, event: &dyn Event, entities: &mut EntityManager);
}

/// 型付きイベントコールバック
pub struct TypedEventCallback<E, F>
where
    E: Event,
    F: Fn(&E, &mut EntityManager) + 'static,
{
    /// コールバック関数
    pub callback: F,
    /// イベント型マーカー
    _marker: PhantomData<E>,
}

impl<E, F> TypedEventCallback<E, F>
where
    E: Event,
    F: Fn(&E, &mut EntityManager) + 'static,
{
    /// 新しいイベントコールバックを作成
    pub fn new(callback: F) -> Self {
        Self {
            callback,
            _marker: PhantomData,
        }
    }
}

impl<E, F> EventCallback for TypedEventCallback<E, F>
where
    E: Event,
    F: Fn(&E, &mut EntityManager) + 'static,
{
    fn event_type_id(&self) -> TypeId {
        TypeId::of::<E>()
    }
    
    fn call(&self, event: &dyn Event, entities: &mut EntityManager) {
        // イベントのダウンキャスト
        if let Some(typed_event) = event.as_any().downcast_ref::<E>() {
            (self.callback)(typed_event, entities);
        }
    }
}

/// イベントハンドラー
/// 
/// 複数のコールバックを登録して、特定の型のイベントを処理するためのコンテナ
pub struct EventHandler<E: Event> {
    /// コールバックのリスト
    callbacks: Vec<Box<dyn Fn(&E, &mut EntityManager) + 'static>>,
}

impl<E: Event> EventHandler<E> {
    /// 新しいイベントハンドラーを作成
    pub fn new() -> Self {
        Self {
            callbacks: Vec::new(),
        }
    }
    
    /// コールバックを追加
    pub fn add<F>(&mut self, callback: F)
    where
        F: Fn(&E, &mut EntityManager) + 'static,
    {
        self.callbacks.push(Box::new(callback));
    }
    
    /// イベントを処理
    pub fn handle(&self, event: &E, entities: &mut EntityManager) {
        for callback in &self.callbacks {
            callback(event, entities);
        }
    }
}

impl<E: Event> Default for EventHandler<E> {
    fn default() -> Self {
        Self::new()
    }
}

/// コールバック生成マクロ
#[macro_export]
macro_rules! event_callback {
    ($event_type:ty, $callback:expr) => {
        crate::events::event_handlers::TypedEventCallback::<$event_type, _>::new($callback)
    };
} 