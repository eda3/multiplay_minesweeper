/**
 * 型付きイベントバスリソース
 * 
 * 型安全なイベントバスをECSリソースとして扱うためのラッパー。
 * リソースマネージャー経由でアクセス可能なイベントバスを提供。
 */
use crate::events::typed_event::TypedEvent;
use crate::events::typed_handler::TypedEventHandler;
use crate::events::typed_event_bus::TypedEventBus;
use crate::events::EventData;

/// 型付きイベントバスリソース - ECSリソースとしてのイベントバスラッパー
#[derive(Debug, Clone)]
pub struct TypedEventBusResource {
    /// 内部のイベントバス
    pub event_bus: TypedEventBus,
}

impl TypedEventBusResource {
    /// 新しいイベントバスリソースを作成
    pub fn new() -> Self {
        Self {
            event_bus: TypedEventBus::new(),
        }
    }
    
    /// デバッグモードを設定したイベントバスリソースを作成
    pub fn with_debug(debug: bool) -> Self {
        Self {
            event_bus: TypedEventBus::new().with_debug(debug),
        }
    }
    
    /// イベントを発行
    pub fn publish<E: TypedEvent>(&self, event: E) {
        self.event_bus.publish(event);
    }
    
    /// イベントを購読
    pub fn subscribe<E: TypedEvent, F>(&self, name: &str, handler: F) -> TypedEventHandler<E>
    where
        F: Fn(&E) + Send + Sync + 'static,
    {
        self.event_bus.subscribe(name, handler)
    }
    
    /// イベントハンドラを登録
    pub fn register_handler<E: TypedEvent>(&self, handler: TypedEventHandler<E>) {
        self.event_bus.register_handler(handler);
    }
    
    /// イベントハンドラを削除
    pub fn unregister_handler<E: TypedEvent>(&self, handler_id: crate::events::typed_event::HandlerId) {
        self.event_bus.unregister_handler::<E>(handler_id);
    }
    
    /// イベント履歴を取得
    pub fn get_history(&self) -> Vec<EventData> {
        self.event_bus.get_history()
    }
    
    /// イベント履歴をクリア
    pub fn clear_history(&self) {
        self.event_bus.clear_history();
    }
    
    /// すべてのハンドラを削除
    pub fn clear_handlers(&self) {
        self.event_bus.clear_handlers();
    }
    
    /// 特定のイベント型に対するハンドラ数を取得
    pub fn handler_count<E: TypedEvent>(&self) -> usize {
        self.event_bus.handler_count::<E>()
    }
}

impl Default for TypedEventBusResource {
    fn default() -> Self {
        Self::new()
    }
} 