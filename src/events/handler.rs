use std::any::{Any, TypeId};

use crate::events::typed_event::{HandlerId, generate_id};
use crate::events::typed_handler::AnyHandlerCollection;

/// ハンドラコレクションの実装
pub struct HandlerCollection {
    handlers: Vec<Box<dyn Fn(&dyn Any) + Send + Sync>>,
    next_id: u64,
    type_id: TypeId,
}

impl HandlerCollection {
    pub fn new() -> Self {
        Self {
            handlers: Vec::new(),
            next_id: 0,
            type_id: TypeId::of::<()>(), // デフォルトはユニット型
        }
    }
    
    pub fn new_with_type<T: 'static>() -> Self {
        Self {
            handlers: Vec::new(),
            next_id: 0,
            type_id: TypeId::of::<T>(),
        }
    }

    pub fn add_handler<F>(&mut self, handler: F) -> HandlerId
    where
        F: Fn(&dyn Any) + Send + Sync + 'static,
    {
        let id = self.next_id;
        self.next_id += 1;
        
        self.handlers.push(Box::new(handler));
        
        HandlerId {
            type_id: self.type_id,
            id,
        }
    }
    
    pub fn remove_handler(&mut self, handler_id: HandlerId) {
        // HandlerIdの順番に基づいてハンドラを削除
        if handler_id.id < self.handlers.len() as u64 {
            self.handlers.remove(handler_id.id as usize);
        }
    }

    pub fn handle_event(&self, event: &dyn Any) {
        for handler in &self.handlers {
            handler(event);
        }
    }
    
    pub fn len(&self) -> usize {
        self.handlers.len()
    }
    
    pub fn is_empty(&self) -> bool {
        self.handlers.is_empty()
    }
    
    pub fn clear(&mut self) {
        self.handlers.clear();
        self.next_id = 0;
    }
}

impl AnyHandlerCollection for HandlerCollection {
    fn handle_any(&self, event: &dyn Any) {
        self.handle_event(event);
    }
    
    fn remove_handler(&mut self, handler_id: HandlerId) -> bool {
        if handler_id.id < self.handlers.len() as u64 {
            self.handlers.remove(handler_id.id as usize);
            true
        } else {
            false
        }
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
    
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
    
    fn len(&self) -> usize {
        self.handlers.len()
    }
} 