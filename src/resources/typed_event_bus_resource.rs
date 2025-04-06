/**
 * 型安全なイベントバスリソース
 * 
 * リソースとして使用できる型安全なイベントバス
 * 依存解決を容易にし、システム間で共有できるようにする
 */
use std::any::Any;
use crate::events::typed_event::TypedEvent;
use crate::events::typed_event_bus::{TypedEventBus, EventPriority};
use crate::events::typed_handler::{TypedEventHandler, EventControl};

#[derive(Debug)]
pub struct TypedEventBusResource {
    /// 型安全なイベントバス
    event_bus: TypedEventBus,
}

impl TypedEventBusResource {
    /// 新しい型安全なイベントバスリソースを作成
    pub fn new() -> Self {
        Self {
            event_bus: TypedEventBus::new(100), // 100件のイベント履歴を保持
        }
    }
    
    /// イベントバスの参照を取得
    pub fn get_event_bus(&self) -> &TypedEventBus {
        &self.event_bus
    }
    
    /// イベントバスの可変参照を取得
    pub fn get_event_bus_mut(&mut self) -> &mut TypedEventBus {
        &mut self.event_bus
    }
    
    /// 全てのハンドラを削除
    pub fn clear_handlers(&mut self) {
        let mut handlers = self.event_bus.handlers.write().unwrap();
        handlers.clear();
        
        let mut global_processors = self.event_bus.global_processors.write().unwrap();
        global_processors.clear();
        
        log::info!("全てのイベントハンドラがクリアされました");
    }
    
    /// イベントを購読
    pub fn subscribe<E: TypedEvent, F>(&self, name: &str, handler: F) -> TypedEventHandler<E>
    where 
        F: Fn(&E) -> EventControl + Send + Sync + 'static,
    {
        self.event_bus.subscribe(name, handler)
    }
    
    /// イベントハンドラを削除
    pub fn unregister_handler<E: TypedEvent>(&self, handler_id: crate::events::typed_event::HandlerId) {
        self.event_bus.unregister_handler::<E>(handler_id);
    }
    
    /// イベントを発行
    pub fn publish<E: TypedEvent>(&self, event: E) {
        self.event_bus.publish(event);
    }
    
    /// ハンドラ数を取得
    pub fn handler_count<E: TypedEvent>(&self) -> usize {
        self.event_bus.handler_count::<E>()
    }
    
    /// イベントを発行する（優先度付き）
    pub fn publish_with_priority<E: TypedEvent>(&self, event: E, priority: EventPriority) {
        self.event_bus.publish_with_priority(event, priority);
    }
    
    /// 複数のイベントをまとめて発行する
    pub fn publish_batch<E: TypedEvent>(&self, events: Vec<E>, priority: EventPriority) {
        // バッチモードを開始
        self.event_bus.start_batch_mode();
        
        // イベントをまとめて発行
        for event in events {
            self.event_bus.publish_with_priority(event, priority);
        }
        
        // バッチモードを終了
        self.event_bus.end_batch_mode();
    }
    
    /// リソースとして使用するためのAnyトレイト実装
    pub fn as_any(&self) -> &dyn Any {
        self
    }
    
    /// リソースとして使用するためのAnyトレイト実装（可変）
    pub fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl Default for TypedEventBusResource {
    fn default() -> Self {
        Self {
            // デフォルトの設定でイベントバスを初期化
            event_bus: TypedEventBus::new(100), // 100件のイベント履歴を保持
        }
    }
} 