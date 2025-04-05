/**
 * イベントバス
 * 
 * イベントの発行と購読を管理する中心的なコンポーネント。
 * 型安全なイベント配信システムを提供。
 */
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::sync::{Arc, RwLock};

use crate::events::event_trait::Event;
use crate::events::event_handler::{EventHandler, EventHandlerFn};
use crate::events::EventData;

/// イベントバス：イベントの発行と購読を管理するハブ
#[derive(Clone)]
pub struct EventBus {
    /// 型ごとのイベントハンドラマップ
    handlers: Arc<RwLock<HashMap<TypeId, Vec<Box<dyn Any + Send + Sync>>>>>,
    /// イベント履歴（オプション）
    event_history: Arc<RwLock<Vec<EventData>>>,
    /// 履歴の最大サイズ
    max_history_size: usize,
    /// デバッグモード
    debug: bool,
}

impl EventBus {
    /// 新しいイベントバスを作成
    pub fn new() -> Self {
        Self {
            handlers: Arc::new(RwLock::new(HashMap::new())),
            event_history: Arc::new(RwLock::new(Vec::new())),
            max_history_size: 100,
            debug: false,
        }
    }
    
    /// デバッグモードを設定
    pub fn with_debug(mut self, debug: bool) -> Self {
        self.debug = debug;
        self
    }
    
    /// 履歴サイズを設定
    pub fn with_history_size(mut self, size: usize) -> Self {
        self.max_history_size = size;
        self
    }
    
    /// イベントを登録して購読
    pub fn subscribe<T: Event, F>(&self, name: &str, handler: F) -> EventHandler<T>
    where
        F: Fn(&T) + Send + Sync + 'static,
    {
        let event_handler = EventHandler::new(name, handler);
        self.register_handler(event_handler.clone());
        event_handler
    }
    
    /// イベントハンドラを登録
    pub fn register_handler<T: Event>(&self, handler: EventHandler<T>) {
        let type_id = TypeId::of::<T>();
        
        let mut handlers = self.handlers.write().unwrap();
        let type_handlers = handlers.entry(type_id).or_insert_with(Vec::new);
        
        // ハンドラを優先度順に挿入（高い優先度が先）
        let boxed_handler = Box::new(handler.clone()) as Box<dyn Any + Send + Sync>;
        
        // 既存のハンドラをEventHandler<T>型に変換し、優先度を比較
        let pos = type_handlers.iter()
            .filter_map(|h| h.downcast_ref::<EventHandler<T>>())
            .position(|h| h.priority < handler.priority)
            .unwrap_or(type_handlers.len());
        
        type_handlers.insert(pos, boxed_handler);
        
        if self.debug {
            println!("登録されたハンドラ: {} for {}", handler.name, std::any::type_name::<T>());
        }
    }
    
    /// イベントハンドラを削除
    pub fn unregister_handler<T: Event>(&self, handler_id: u64) {
        let type_id = TypeId::of::<T>();
        
        let mut handlers = self.handlers.write().unwrap();
        if let Some(type_handlers) = handlers.get_mut(&type_id) {
            type_handlers.retain(|h| {
                h.downcast_ref::<EventHandler<T>>()
                    .map(|handler| handler.id != handler_id)
                    .unwrap_or(true)
            });
            
            if self.debug {
                println!("削除されたハンドラ: ID {} for {}", handler_id, std::any::type_name::<T>());
            }
        }
    }
    
    /// イベントの発行
    pub fn publish<T: Event>(&self, event: T) {
        let type_id = TypeId::of::<T>();
        
        // イベント履歴に追加（適切なEventDataへの変換が必要）
        if let Some(event_data) = self.convert_to_event_data(event.clone()) {
            let mut history = self.event_history.write().unwrap();
            history.push(event_data);
            
            // 最大サイズを超えたら古いものを削除
            if history.len() > self.max_history_size {
                history.remove(0);
            }
        }
        
        if self.debug {
            println!("イベント発行: {} ({:?})", event.name(), event);
        }
        
        // ハンドラを取得して呼び出し
        let handlers = self.handlers.read().unwrap();
        if let Some(type_handlers) = handlers.get(&type_id) {
            // イベントが変更されないのでクローンして各ハンドラに渡す
            for handler_any in type_handlers {
                if let Some(handler) = handler_any.downcast_ref::<EventHandler<T>>() {
                    if handler.is_enabled() {
                        handler.handle(&event);
                    }
                }
            }
        }
    }
    
    /// イベント履歴を取得
    pub fn get_history(&self) -> Vec<EventData> {
        let history = self.event_history.read().unwrap();
        history.clone()
    }
    
    /// イベント履歴をクリア
    pub fn clear_history(&self) {
        let mut history = self.event_history.write().unwrap();
        history.clear();
    }
    
    /// イベントをEventDataに変換（内部実装用）
    fn convert_to_event_data<T: Event>(&self, _event: T) -> Option<EventData> {
        // 各イベント型に対応する変換を行う
        // これは大規模なマッチ式になる可能性があるため、必要に応じて個別のメソッドに分割する
        None // 実際の実装では適切な変換を行う
    }
    
    /// 特定のイベント型に対するハンドラ数を取得
    pub fn handler_count<T: Event>(&self) -> usize {
        let type_id = TypeId::of::<T>();
        let handlers = self.handlers.read().unwrap();
        
        handlers.get(&type_id)
            .map(|type_handlers| type_handlers.len())
            .unwrap_or(0)
    }
    
    /// すべてのハンドラを削除
    pub fn clear_handlers(&self) {
        let mut handlers = self.handlers.write().unwrap();
        handlers.clear();
        
        if self.debug {
            println!("すべてのハンドラがクリアされました");
        }
    }
}

impl Debug for EventBus {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let handlers = self.handlers.read().unwrap();
        let handler_count: usize = handlers.values().map(|v| v.len()).sum();
        
        f.debug_struct("EventBus")
            .field("handler_count", &handler_count)
            .field("type_count", &handlers.len())
            .field("history_size", &self.event_history.read().unwrap().len())
            .field("max_history_size", &self.max_history_size)
            .field("debug_mode", &self.debug)
            .finish()
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
} 