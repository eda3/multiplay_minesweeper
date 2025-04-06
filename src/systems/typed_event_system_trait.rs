/**
 * 型安全なイベントシステムのトレイト
 * 
 * 型安全なイベントの購読と発行のためのトレイト
 */
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::events::typed_event::{TypedEvent, HandlerId};
use crate::events::typed_event_bus::EventPriority;
use crate::events::typed_handler::{TypedEventHandler, AnyHandler, EventControl};
use crate::resources::{ResourceManager, TypedEventBusResource};

/// 型安全なイベントハンドラを管理するシステム
pub struct TypedEventSystem {
    /// イベントハンドラのID
    handler_ids: Arc<Mutex<HashMap<String, HandlerId>>>,
}

impl TypedEventSystem {
    /// 新しい型安全なイベントシステムを作成
    pub fn new() -> Self {
        Self {
            handler_ids: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

impl Default for TypedEventSystem {
    fn default() -> Self {
        Self::new()
    }
}

/// 型安全なイベントシステムのトレイト
pub trait TypedEventSystemTrait: Send + Sync {
    /// ハンドラIDの参照を取得
    fn get_handler_ids(&self) -> &Arc<Mutex<HashMap<String, HandlerId>>>;
    
    /// ハンドラIDの可変参照を取得
    fn get_handler_ids_mut(&mut self) -> &mut Arc<Mutex<HashMap<String, HandlerId>>>;
    
    /// イベントを購読する
    fn subscribe_typed_event<E: TypedEvent, F>(&mut self, 
        event_type_name: &str, 
        handler_name: &str, 
        handler: F, 
        resources: &ResourceManager
    ) -> Option<TypedEventHandler<E>>
    where 
        F: Fn(&E) -> EventControl + Send + Sync + 'static,
    {
        let handler_key = format!("{}_{}", event_type_name, handler_name);
        
        if let Ok(event_bus_rc) = resources.get::<TypedEventBusResource>() {
            let event_bus = event_bus_rc.borrow();
            if let Some(event_bus_res) = event_bus.downcast_ref::<TypedEventBusResource>() {
                // イベントハンドラを登録
                let event_handler = event_bus_res.subscribe(handler_name, handler);
                
                // ハンドラIDを保存
                if let Ok(mut guard) = self.get_handler_ids().lock() {
                    guard.insert(handler_key, event_handler.id);
                }
                
                return Some(event_handler);
            }
        }
        
        None
    }
    
    /// イベントの購読を解除する
    fn unsubscribe_typed_event<E: TypedEvent>(&mut self, 
        event_type_name: &str, 
        handler_name: &str, 
        resources: &ResourceManager
    ) -> bool {
        let handler_key = format!("{}_{}", event_type_name, handler_name);
        
        if let Ok(mut guard) = self.get_handler_ids_mut().lock() {
            if let Some(id) = guard.remove(&handler_key) {
                // イベントバスからハンドラを削除
                if let Ok(event_bus_rc) = resources.get::<TypedEventBusResource>() {
                    let event_bus = event_bus_rc.borrow();
                    if let Some(event_bus_res) = event_bus.downcast_ref::<TypedEventBusResource>() {
                        event_bus_res.unregister_handler::<E>(id);
                        return true;
                    }
                }
            }
        }
        
        false
    }
    
    /// イベントを発行する（型安全バージョン）
    fn publish_typed_event<E: TypedEvent>(&self, 
        event: E, 
        resources: &ResourceManager
    ) {
        if let Ok(event_bus_rc) = resources.get::<TypedEventBusResource>() {
            let event_bus = event_bus_rc.borrow();
            if let Some(event_bus_res) = event_bus.downcast_ref::<TypedEventBusResource>() {
                event_bus_res.publish(event);
            }
        }
    }
    
    /// イベントを発行する（優先度付き）
    fn publish_typed_event_with_priority<E: TypedEvent>(&self, 
        event: E, 
        priority: EventPriority, 
        resources: &ResourceManager
    ) {
        if let Ok(event_bus_rc) = resources.get::<TypedEventBusResource>() {
            let event_bus = event_bus_rc.borrow();
            if let Some(event_bus_res) = event_bus.downcast_ref::<TypedEventBusResource>() {
                event_bus_res.publish_with_priority(event, priority);
            }
        }
    }
    
    /// 複数のイベントをまとめて発行
    fn publish_typed_event_batch<E: TypedEvent>(&self, 
        events: Vec<E>, 
        priority: EventPriority, 
        resources: &ResourceManager
    ) {
        if let Ok(event_bus_rc) = resources.get::<TypedEventBusResource>() {
            let event_bus = event_bus_rc.borrow();
            if let Some(event_bus_res) = event_bus.downcast_ref::<TypedEventBusResource>() {
                event_bus_res.publish_batch(events, priority);
            }
        }
    }
    
    /// すべてのイベントハンドラを削除
    fn clear_all_handlers(&self, resources: &ResourceManager) {
        if let Ok(event_bus_rc) = resources.get::<TypedEventBusResource>() {
            let mut event_bus = event_bus_rc.borrow_mut();
            if let Some(event_bus_res) = event_bus.downcast_mut::<TypedEventBusResource>() {
                // すべてのハンドラを削除
                event_bus_res.clear_handlers();
            }
        }
        
        // ローカルのハンドラIDもクリア
        if let Ok(mut guard) = self.get_handler_ids().lock() {
            guard.clear();
        }
    }
}

impl TypedEventSystemTrait for TypedEventSystem {
    fn get_handler_ids(&self) -> &Arc<Mutex<HashMap<String, HandlerId>>> {
        &self.handler_ids
    }
    
    fn get_handler_ids_mut(&mut self) -> &mut Arc<Mutex<HashMap<String, HandlerId>>> {
        &mut self.handler_ids
    }
} 