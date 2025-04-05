/**
 * イベントバスリソース
 * 
 * イベントバスをECSリソースとして扱うためのラッパー
 */
use crate::events::EventBus;
use crate::events::Event;
use crate::events::event_handler::EventHandler;

/// イベントバスをECSリソースとして扱うためのラッパー
#[derive(Debug, Clone)]
pub struct EventBusResource {
    /// 内部のイベントバス
    pub event_bus: EventBus,
}

impl EventBusResource {
    /// 新しいイベントバスリソースを作成
    pub fn new() -> Self {
        Self {
            event_bus: EventBus::new(),
        }
    }
    
    /// デバッグモードを設定したイベントバスリソースを作成
    pub fn with_debug(debug: bool) -> Self {
        Self {
            event_bus: EventBus::new().with_debug(debug),
        }
    }
    
    /// イベントを発行
    pub fn publish<T: Event>(&self, event: T) {
        self.event_bus.publish(event);
    }
    
    /// イベントを購読
    pub fn subscribe<T: Event, F>(&self, name: &str, handler: F) -> EventHandler<T>
    where
        F: Fn(&T) + Send + Sync + 'static,
    {
        self.event_bus.subscribe(name, handler)
    }
    
    /// イベントハンドラを登録
    pub fn register_handler<T: Event>(&self, handler: EventHandler<T>) {
        self.event_bus.register_handler(handler);
    }
    
    /// イベントハンドラを削除
    pub fn unregister_handler<T: Event>(&self, handler_id: u64) {
        self.event_bus.unregister_handler::<T>(handler_id);
    }
    
    /// イベント履歴を取得
    pub fn get_history(&self) -> Vec<crate::events::EventData> {
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
}

impl Default for EventBusResource {
    fn default() -> Self {
        Self::new()
    }
} 