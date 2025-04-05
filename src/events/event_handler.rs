/**
 * イベントハンドラ
 * 
 * イベントの処理を行うためのハンドラ定義。クロージャベースの柔軟なAPIを提供。
 */
use std::fmt::{Debug, Formatter};
use std::sync::{Arc, Mutex};
use crate::events::event_trait::Event;

/// イベントハンドラ関数の型定義
pub type EventHandlerFn<T> = Arc<dyn Fn(&T) + Send + Sync>;

/// イベントハンドラ構造体
pub struct EventHandler<T: Event> {
    /// ハンドラID
    pub id: u64,
    /// ハンドラ名
    pub name: String,
    /// イベント処理関数
    handler: EventHandlerFn<T>,
    /// 優先度（高い値ほど先に実行）
    pub priority: i32,
    /// 有効・無効状態
    pub enabled: Arc<Mutex<bool>>,
    /// 一度だけ実行するかどうか
    pub once: bool,
}

impl<T: Event> EventHandler<T> {
    /// 新しいイベントハンドラを作成
    pub fn new<F>(name: &str, handler: F) -> Self 
    where
        F: Fn(&T) + Send + Sync + 'static
    {
        static NEXT_ID: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
        
        Self {
            id: NEXT_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
            name: name.to_string(),
            handler: Arc::new(handler),
            priority: 0,
            enabled: Arc::new(Mutex::new(true)),
            once: false,
        }
    }
    
    /// 優先度を設定
    pub fn with_priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }
    
    /// 一度だけ実行するように設定
    pub fn once(mut self) -> Self {
        self.once = true;
        self
    }
    
    /// ハンドラの有効・無効を設定
    pub fn set_enabled(&self, enabled: bool) {
        if let Ok(mut guard) = self.enabled.lock() {
            *guard = enabled;
        }
    }
    
    /// ハンドラが有効かどうかを確認
    pub fn is_enabled(&self) -> bool {
        self.enabled.lock().map(|guard| *guard).unwrap_or(false)
    }
    
    /// イベントを処理
    pub fn handle(&self, event: &T) {
        if self.is_enabled() {
            (self.handler)(event);
            
            if self.once {
                self.set_enabled(false);
            }
        }
    }
    
    /// ハンドラ関数を取得
    pub fn handler(&self) -> Arc<dyn Fn(&T) + Send + Sync> {
        self.handler.clone()
    }
}

impl<T: Event> Debug for EventHandler<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EventHandler")
            .field("id", &self.id)
            .field("name", &self.name)
            .field("priority", &self.priority)
            .field("enabled", &self.is_enabled())
            .field("once", &self.once)
            .finish()
    }
}

impl<T: Event> Clone for EventHandler<T> {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            name: self.name.clone(),
            handler: self.handler.clone(),
            priority: self.priority,
            enabled: self.enabled.clone(),
            once: self.once,
        }
    }
}

impl<T: Event> PartialEq for EventHandler<T> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl<T: Event> Eq for EventHandler<T> {} 