/**
 * 型付きイベントシステムトレイト
 * 
 * 型安全なイベントバスを利用するシステムの基本機能を提供するトレイト。
 * 型情報を保持したままイベントの発行と購読を行う。
 */
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::events::typed_event::{TypedEvent, HandlerId};
use crate::events::typed_handler::TypedEventHandler;
use crate::resources::ResourceManager;
use crate::resources::typed_event_bus_resource::TypedEventBusResource;

/// 型付きイベントシステムトレイト
/// イベントバスを利用するシステムの基本機能を提供する
pub trait TypedEventSystemTrait {
    /// システムが使用するイベントハンドラIDを保存するためのマップを取得
    fn get_handler_ids(&self) -> &Arc<Mutex<HashMap<String, HandlerId>>>;
    
    /// システムが使用するイベントハンドラIDを保存するためのマップの可変参照を取得
    fn get_handler_ids_mut(&mut self) -> &mut Arc<Mutex<HashMap<String, HandlerId>>>;
    
    /// イベントを発行（型安全バージョン）
    fn publish_typed_event<E: TypedEvent>(&self, event: E, resources: &ResourceManager) {
        if let Ok(event_bus_rc) = resources.get::<TypedEventBusResource>() {
            let event_bus = event_bus_rc.borrow();
            if let Some(event_bus_res) = event_bus.downcast_ref::<TypedEventBusResource>() {
                event_bus_res.publish(event);
            }
        }
    }
    
    /// イベントを購読（型安全バージョン）
    fn subscribe_typed_event<E: TypedEvent, F>(
        &self,
        event_name: &str,
        handler_name: &str,
        handler: F,
        resources: &ResourceManager
    ) -> Option<TypedEventHandler<E>>
    where
        F: Fn(&E) + Send + Sync + 'static,
    {
        if let Ok(event_bus_rc) = resources.get::<TypedEventBusResource>() {
            let event_bus = event_bus_rc.borrow();
            if let Some(event_bus_res) = event_bus.downcast_ref::<TypedEventBusResource>() {
                let handler_key = format!("{}_{}", event_name, handler_name);
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
    
    /// イベント購読を解除（型安全バージョン）
    fn unsubscribe_typed_event<E: TypedEvent>(&self, event_name: &str, handler_name: &str, resources: &ResourceManager) -> bool {
        let handler_key = format!("{}_{}", event_name, handler_name);
        let handler_id = {
            if let Ok(guard) = self.get_handler_ids().lock() {
                guard.get(&handler_key).cloned()
            } else {
                None
            }
        };
        
        if let Some(id) = handler_id {
            if let Ok(event_bus_rc) = resources.get::<TypedEventBusResource>() {
                let event_bus = event_bus_rc.borrow();
                if let Some(event_bus_res) = event_bus.downcast_ref::<TypedEventBusResource>() {
                    event_bus_res.unregister_handler::<E>(id);
                    
                    // ハンドラIDを削除
                    if let Ok(mut guard) = self.get_handler_ids().lock() {
                        guard.remove(&handler_key);
                    }
                    
                    return true;
                }
            }
        }
        false
    }
    
    /// システムの終了時に全てのイベント購読を解除（型安全バージョン）
    fn cleanup_typed_events(&mut self, resources: &ResourceManager) {
        if let Ok(event_bus_rc) = resources.get::<TypedEventBusResource>() {
            let event_bus = event_bus_rc.borrow();
            if let Some(event_bus_res) = event_bus.downcast_ref::<TypedEventBusResource>() {
                // すべてのハンドラを削除（このメソッドはダウンキャストを使わずに削除が可能）
                event_bus_res.clear_handlers();
                
                // ハンドラIDマップをクリア
                if let Ok(mut guard) = self.get_handler_ids().lock() {
                    guard.clear();
                }
            }
        }
    }
}

/// 型付きイベントシステムトレイトのデフォルト実装のためのユーティリティ構造体
pub struct TypedEventSystem {
    /// イベントハンドラIDを保存するマップ
    handler_ids: Arc<Mutex<HashMap<String, HandlerId>>>,
}

impl TypedEventSystem {
    /// 新しいTypedEventSystemを作成
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

impl TypedEventSystemTrait for TypedEventSystem {
    fn get_handler_ids(&self) -> &Arc<Mutex<HashMap<String, HandlerId>>> {
        &self.handler_ids
    }
    
    fn get_handler_ids_mut(&mut self) -> &mut Arc<Mutex<HashMap<String, HandlerId>>> {
        &mut self.handler_ids
    }
} 