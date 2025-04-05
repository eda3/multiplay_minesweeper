/**
 * イベントシステムトレイト
 * 
 * イベントバスを利用するシステムの基本機能を提供するトレイト
 */
use crate::events::Event;
use crate::events::event_handler::EventHandler;
use crate::resources::ResourceManager;
use crate::resources::EventBusResource;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// イベントシステムトレイト
/// イベントバスを利用するシステムの基本機能を提供する
pub trait EventSystemTrait {
    /// システムが使用するイベントハンドラIDを保存するためのマップを取得
    fn get_handler_ids(&self) -> &Arc<Mutex<HashMap<String, u64>>>;
    
    /// システムが使用するイベントハンドラIDを保存するためのマップの可変参照を取得
    fn get_handler_ids_mut(&mut self) -> &mut Arc<Mutex<HashMap<String, u64>>>;
    
    /// イベントを発行
    fn publish_event<T: Event>(&self, event: T, resources: &ResourceManager) {
        if let Ok(event_bus_rc) = resources.get::<EventBusResource>() {
            let event_bus = event_bus_rc.borrow();
            if let Some(event_bus_res) = event_bus.downcast_ref::<EventBusResource>() {
                event_bus_res.publish(event);
            }
        }
    }
    
    /// イベントを購読し、ハンドラIDを保存
    fn subscribe_event<T: Event, F>(&self, event_name: &str, handler_name: &str, handler: F, resources: &ResourceManager) -> Option<EventHandler<T>>
    where
        F: Fn(&T) + Send + Sync + 'static,
    {
        if let Ok(event_bus_rc) = resources.get::<EventBusResource>() {
            let event_bus = event_bus_rc.borrow();
            if let Some(event_bus_res) = event_bus.downcast_ref::<EventBusResource>() {
                let event_handler = event_bus_res.subscribe(handler_name, handler);
                
                // ハンドラIDを保存
                if let Ok(mut guard) = self.get_handler_ids().lock() {
                    guard.insert(format!("{}_{}", event_name, handler_name), event_handler.id);
                }
                
                return Some(event_handler);
            }
        }
        None
    }
    
    /// 購読を解除
    fn unsubscribe_event<T: Event>(&self, event_name: &str, handler_name: &str, resources: &ResourceManager) -> bool {
        let handler_id = {
            if let Ok(guard) = self.get_handler_ids().lock() {
                guard.get(&format!("{}_{}", event_name, handler_name)).cloned()
            } else {
                None
            }
        };
        
        if let Some(id) = handler_id {
            if let Ok(event_bus_rc) = resources.get::<EventBusResource>() {
                let event_bus = event_bus_rc.borrow();
                if let Some(event_bus_res) = event_bus.downcast_ref::<EventBusResource>() {
                    event_bus_res.unregister_handler::<T>(id);
                    
                    // ハンドラIDを削除
                    if let Ok(mut guard) = self.get_handler_ids().lock() {
                        guard.remove(&format!("{}_{}", event_name, handler_name));
                    }
                    
                    return true;
                }
            }
        }
        false
    }
    
    /// システムの終了時に全てのイベント購読を解除
    fn cleanup_events(&mut self, resources: &ResourceManager) {
        if let Ok(event_bus_rc) = resources.get::<EventBusResource>() {
            let event_bus = event_bus_rc.borrow();
            if let Some(event_bus_res) = event_bus.downcast_ref::<EventBusResource>() {
                // 全てのハンドラIDを取得
                let handler_ids = if let Ok(guard) = self.get_handler_ids().lock() {
                    guard.clone()
                } else {
                    return;
                };
                
                for (key, id) in handler_ids {
                    // 実際の型がわからないため、低レベルAPIを使用
                    event_bus_res.event_bus.unregister_handler_any(id);
                }
                
                // ハンドラIDマップをクリア
                if let Ok(mut guard) = self.get_handler_ids().lock() {
                    guard.clear();
                }
            }
        }
    }
}

/// イベントシステムトレイトのデフォルト実装のためのユーティリティ構造体
pub struct EventSystem {
    /// イベントハンドラIDを保存するマップ
    handler_ids: Arc<Mutex<HashMap<String, u64>>>,
}

impl EventSystem {
    /// 新しいEventSystemを作成
    pub fn new() -> Self {
        Self {
            handler_ids: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

impl Default for EventSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl EventSystemTrait for EventSystem {
    fn get_handler_ids(&self) -> &Arc<Mutex<HashMap<String, u64>>> {
        &self.handler_ids
    }
    
    fn get_handler_ids_mut(&mut self) -> &mut Arc<Mutex<HashMap<String, u64>>> {
        &mut self.handler_ids
    }
} 