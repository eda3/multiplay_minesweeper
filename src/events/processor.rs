use std::any::Any;
use crate::events::typed_handler::EventControl;

/// イベントプロセッサID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ProcessorId(pub u32);

/// イベントプロセッサトレイト
pub trait EventProcessor: Send + Sync {
    /// イベントを処理
    fn process_event(&self, event: &dyn Any) -> EventControl;
}

/// イベントプロセッサコレクション
pub struct EventProcessorCollection {
    processors: Vec<Box<dyn EventProcessor + 'static>>,
    next_id: ProcessorId,
}

impl EventProcessorCollection {
    pub fn new() -> Self {
        Self { 
            processors: Vec::new(),
            next_id: ProcessorId(0),
        }
    }

    pub fn add_processor<P: EventProcessor + 'static>(&mut self, processor: P) -> ProcessorId {
        let id = self.next_id;
        self.next_id = ProcessorId(id.0 + 1);
        
        self.processors.push(Box::new(processor));
        id
    }
    
    pub fn remove_processor(&mut self, processor_id: ProcessorId) {
        // ProcessorIdの順番に基づいてプロセッサーを削除
        if processor_id.0 < self.processors.len() as u32 {
            self.processors.remove(processor_id.0 as usize);
        }
    }

    pub fn process_event<E: Any>(&self, event: &E) -> EventControl {
        for processor in &self.processors {
            let result = processor.process_event(event);
            if let EventControl::Break = result {
                return EventControl::Break;
            }
        }
        EventControl::Continue
    }
    
    pub fn len(&self) -> usize {
        self.processors.len()
    }
    
    pub fn is_empty(&self) -> bool {
        self.processors.is_empty()
    }
    
    pub fn clear(&mut self) {
        self.processors.clear();
        self.next_id = ProcessorId(0);
    }
}

impl<'a> IntoIterator for &'a EventProcessorCollection {
    type Item = &'a Box<dyn EventProcessor + 'static>;
    type IntoIter = std::slice::Iter<'a, Box<dyn EventProcessor + 'static>>;

    fn into_iter(self) -> Self::IntoIter {
        self.processors.iter()
    }
} 