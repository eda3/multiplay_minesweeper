/**
 * 型安全なイベント処理システムの基底クラス
 * 
 * すべてのイベント処理システムの共通の基盤となるクラス。
 * スレッドセーフなメッセージパッシングパターンを実装しています。
 */
use std::sync::{Arc, Mutex};
use std::collections::{HashMap, VecDeque};
use std::fmt::Debug;
use std::marker::PhantomData;

use crate::events::typed_event::{TypedEvent, HandlerId};
use crate::events::typed_event_bus::EventPriority;
use crate::resources::ResourceManager;
use crate::systems::typed_event_system_trait::{TypedEventSystemTrait, TypedEventSystem};
use crate::ecs::system::{System, SystemResult};
use crate::entities::EntityManager;

/// イベント処理状態
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventProcessingState {
    /// 成功
    Success,
    /// 無視された
    Ignored,
    /// エラー
    Error,
}

/// イベントリクエスト（型安全）
#[derive(Debug, Clone)]
pub struct EventRequest<T: Debug + Clone + Send> {
    /// イベントデータ
    pub event: T,
    /// イベントの優先度
    pub priority: EventPriority,
    /// イベントのタイムスタンプ
    pub timestamp: u64,
}

/// イベントキュー（型安全）
#[derive(Debug)]
pub struct EventQueue<T: Debug + Clone + Send> {
    /// 処理待ちイベントのキュー
    queue: VecDeque<EventRequest<T>>,
    /// 優先度に基づいて処理されたイベントのカウント
    high_priority_count: usize,
    normal_priority_count: usize,
    low_priority_count: usize,
    /// 最後に処理したイベントのタイムスタンプ
    last_processed_timestamp: u64,
}

impl<T: Debug + Clone + Send> EventQueue<T> {
    /// 新しいイベントキューを作成
    pub fn new() -> Self {
        Self {
            queue: VecDeque::new(),
            high_priority_count: 0,
            normal_priority_count: 0,
            low_priority_count: 0,
            last_processed_timestamp: 0,
        }
    }
    
    /// イベントをキューに追加
    pub fn enqueue(&mut self, event: T, priority: EventPriority, timestamp: u64) {
        let request = EventRequest {
            event,
            priority,
            timestamp,
        };
        
        // 優先度に応じてキューの適切な位置に追加
        match priority {
            EventPriority::Highest | EventPriority::High => {
                // 高優先度イベントは先頭に追加
                self.queue.push_front(request);
                self.high_priority_count += 1;
            },
            EventPriority::Normal => {
                // 通常優先度イベントは高優先度の後に追加
                let index = self.high_priority_count;
                self.queue.insert(index, request);
                self.normal_priority_count += 1;
            },
            EventPriority::Low => {
                // 低優先度イベントは最後に追加
                self.queue.push_back(request);
                self.low_priority_count += 1;
            },
            EventPriority::Lowest => {
                // 最低優先度イベントは最後に追加
                self.queue.push_back(request);
                self.low_priority_count += 1;
            },
        }
    }
    
    /// イベントをキューから取得
    pub fn dequeue(&mut self) -> Option<EventRequest<T>> {
        let request = self.queue.pop_front()?;
        
        // カウンタを更新
        match request.priority {
            EventPriority::Highest => self.high_priority_count = self.high_priority_count.saturating_sub(1),
            EventPriority::High => self.high_priority_count = self.high_priority_count.saturating_sub(1),
            EventPriority::Normal => self.normal_priority_count = self.normal_priority_count.saturating_sub(1),
            EventPriority::Low => self.low_priority_count = self.low_priority_count.saturating_sub(1),
            EventPriority::Lowest => self.low_priority_count = self.low_priority_count.saturating_sub(1),
        }
        
        // タイムスタンプを更新
        self.last_processed_timestamp = request.timestamp;
        
        Some(request)
    }
    
    /// キューに残っているイベント数を取得
    pub fn len(&self) -> usize {
        self.queue.len()
    }
    
    /// キューが空かどうか
    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }
    
    /// 最後に処理したイベントのタイムスタンプを取得
    pub fn get_last_timestamp(&self) -> u64 {
        self.last_processed_timestamp
    }
}

impl<T: Debug + Clone + Send> Default for EventQueue<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// 型安全イベント処理システムの基底クラス
pub struct BaseEventSystem<T: Debug + Clone + Send + 'static> {
    /// システム名
    name: String,
    /// 型安全なイベントシステム
    event_system: TypedEventSystem,
    /// イベントハンドラの初期化済みフラグ
    initialized: bool,
    /// システムが有効かどうか
    enabled: bool,
    /// イベント処理キュー
    event_queue: Arc<Mutex<EventQueue<T>>>,
}

impl<T: Debug + Clone + Send + 'static> BaseEventSystem<T> {
    /// 新しいイベント処理システムを作成
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            event_system: TypedEventSystem::new(),
            initialized: false,
            enabled: true,
            event_queue: Arc::new(Mutex::new(EventQueue::new())),
        }
    }
    
    /// イベントをキューに追加
    pub fn enqueue_event(&self, event: T, priority: EventPriority, timestamp: u64) -> bool {
        if let Ok(mut queue) = self.event_queue.lock() {
            queue.enqueue(event, priority, timestamp);
            true
        } else {
            false
        }
    }
    
    /// キューから1つのイベントを処理
    pub fn process_one_event(&self, resources: &ResourceManager, max_events: usize) -> Option<EventRequest<T>> {
        // キューからイベントを取得
        let request = {
            if let Ok(mut queue) = self.event_queue.lock() {
                queue.dequeue()
            } else {
                None
            }
        };
        
        request
    }
    
    /// キューにイベントが残っているかどうか
    pub fn has_pending_events(&self) -> bool {
        if let Ok(queue) = self.event_queue.lock() {
            !queue.is_empty()
        } else {
            false
        }
    }
    
    /// イベントキューを共有参照で取得
    pub fn get_event_queue(&self) -> &Arc<Mutex<EventQueue<T>>> {
        &self.event_queue
    }
    
    /// イベントキューを直接アクセスできるクローンを取得
    pub fn clone_event_queue(&self) -> Arc<Mutex<EventQueue<T>>> {
        self.event_queue.clone()
    }
}

impl<T: Debug + Clone + Send + 'static> Clone for BaseEventSystem<T> {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            event_system: TypedEventSystem::new(),
            initialized: self.initialized,
            enabled: self.enabled,
            event_queue: self.event_queue.clone(),
        }
    }
}

impl<T: Debug + Clone + Send + 'static> TypedEventSystemTrait for BaseEventSystem<T> {
    fn get_handler_ids(&self) -> &Arc<Mutex<HashMap<String, HandlerId>>> {
        self.event_system.get_handler_ids()
    }
    
    fn get_handler_ids_mut(&mut self) -> &mut Arc<Mutex<HashMap<String, HandlerId>>> {
        self.event_system.get_handler_ids_mut()
    }
}

impl<T: Debug + Clone + Send + 'static> System for BaseEventSystem<T> {
    fn update(&mut self, _entity_manager: &mut EntityManager, _resources: &mut ResourceManager) -> SystemResult {
        // このメソッドは継承クラスでオーバーライドすることを想定
        // デフォルトでは何もしない
        SystemResult::Ok
    }
    
    fn name(&self) -> &str {
        &self.name
    }
    
    fn enabled(&self) -> bool {
        self.enabled
    }
    
    fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
} 