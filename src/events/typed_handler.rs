/**
 * 型付きイベントハンドラ
 * 
 * コンパイル時の型安全性を強化したイベントハンドラ。
 * イベントハンドラの型情報を保持し、ダウンキャストを最小限に抑える。
 */
use std::fmt::{Debug, Formatter};
use std::sync::{Arc, Mutex};
use std::any::Any;

use crate::events::typed_event::{TypedEvent, HandlerId, generate_id};

/// 型付きイベントハンドラ関数の型定義
pub type TypedEventHandlerFn<E> = Arc<dyn Fn(&E) + Send + Sync + 'static>;

/// 型付きイベントハンドラ
/// 具体的なイベント型に対するハンドラ
pub struct TypedEventHandler<E: TypedEvent> {
    /// ハンドラの一意なID
    pub id: HandlerId,
    /// ハンドラの名前（デバッグ用）
    pub name: String,
    /// イベント処理関数
    handler: TypedEventHandlerFn<E>,
    /// 優先度（高い値ほど先に実行）
    pub priority: i32,
    /// 有効・無効状態
    pub enabled: Arc<Mutex<bool>>,
    /// 一度だけ実行するかどうか
    pub once: bool,
}

impl<E: TypedEvent> TypedEventHandler<E> {
    /// 新しいイベントハンドラを作成
    pub fn new<F>(name: &str, handler: F) -> Self 
    where
        F: Fn(&E) + Send + Sync + 'static
    {
        let id = generate_id();
        Self {
            id: HandlerId::new::<E>(id),
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
    pub fn handle(&self, event: &E) {
        if self.is_enabled() {
            (self.handler)(event);
            
            if self.once {
                self.set_enabled(false);
            }
        }
    }
    
    /// ハンドラ関数を取得
    pub fn handler(&self) -> Arc<dyn Fn(&E) + Send + Sync> {
        self.handler.clone()
    }
}

impl<E: TypedEvent> Debug for TypedEventHandler<E> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TypedEventHandler")
            .field("id", &self.id)
            .field("name", &self.name)
            .field("event_type", &E::type_name())
            .field("priority", &self.priority)
            .field("enabled", &self.is_enabled())
            .field("once", &self.once)
            .finish()
    }
}

impl<E: TypedEvent> Clone for TypedEventHandler<E> {
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

/// 型を消去するトレイト（内部用）
/// 異なる型のハンドラを同じコレクションで管理するため
pub trait AnyHandler: Send + Sync {
    /// ハンドラIDを取得
    fn id(&self) -> HandlerId;
    
    /// ハンドラ名を取得
    fn name(&self) -> &str;
    
    /// 優先度を取得
    fn priority(&self) -> i32;
    
    /// 有効かどうか
    fn is_enabled(&self) -> bool;
    
    /// 有効・無効を設定
    fn set_enabled(&self, enabled: bool);
    
    /// 一度だけ実行か
    fn is_once(&self) -> bool;
    
    /// イベントを処理（Any型経由）
    fn handle_any(&self, event: &dyn Any);
    
    /// ハンドラのクローンを作成
    fn box_clone(&self) -> Box<dyn AnyHandler>;
}

impl<E: TypedEvent> AnyHandler for TypedEventHandler<E> {
    fn id(&self) -> HandlerId {
        self.id
    }
    
    fn name(&self) -> &str {
        &self.name
    }
    
    fn priority(&self) -> i32 {
        self.priority
    }
    
    fn is_enabled(&self) -> bool {
        self.is_enabled()
    }
    
    fn set_enabled(&self, enabled: bool) {
        self.set_enabled(enabled);
    }
    
    fn is_once(&self) -> bool {
        self.once
    }
    
    fn handle_any(&self, event: &dyn Any) {
        if let Some(typed_event) = event.downcast_ref::<E>() {
            self.handle(typed_event);
        }
    }
    
    fn box_clone(&self) -> Box<dyn AnyHandler> {
        Box::new(self.clone())
    }
}

/// 型付きハンドラのコレクション
/// 同じ型のイベントを処理するハンドラのコレクション
pub struct TypedHandlerCollection<E: TypedEvent> {
    /// ハンドラのリスト
    handlers: Vec<TypedEventHandler<E>>,
}

impl<E: TypedEvent> TypedHandlerCollection<E> {
    /// 新しいコレクションを作成
    pub fn new() -> Self {
        Self {
            handlers: Vec::new(),
        }
    }
    
    /// ハンドラを追加
    pub fn add(&mut self, handler: TypedEventHandler<E>) {
        // 優先度に基づいて挿入位置を決定
        let pos = self.handlers
            .iter()
            .position(|h| h.priority < handler.priority)
            .unwrap_or(self.handlers.len());
        
        self.handlers.insert(pos, handler);
    }
    
    /// 指定IDのハンドラを削除
    pub fn remove(&mut self, handler_id: HandlerId) -> bool {
        let len = self.handlers.len();
        self.handlers.retain(|h| h.id != handler_id);
        len != self.handlers.len()
    }
    
    /// イベントを処理
    pub fn handle(&self, event: &E) {
        for handler in &self.handlers {
            handler.handle(event);
        }
    }
    
    /// ハンドラ数を取得
    pub fn len(&self) -> usize {
        self.handlers.len()
    }
    
    /// コレクションが空かどうか
    pub fn is_empty(&self) -> bool {
        self.handlers.is_empty()
    }
    
    /// すべてのハンドラを削除
    pub fn clear(&mut self) {
        self.handlers.clear();
    }
}

impl<E: TypedEvent> Default for TypedHandlerCollection<E> {
    fn default() -> Self {
        Self::new()
    }
}

/// 型消去されたハンドラコレクションのトレイト
pub trait AnyHandlerCollection: Send + Sync {
    /// イベントを処理
    fn handle_any(&self, event: &dyn Any);
    
    /// 指定IDのハンドラを削除
    fn remove(&mut self, handler_id: HandlerId) -> bool;
    
    /// すべてのハンドラを削除
    fn clear(&mut self);
    
    /// ハンドラ数を取得
    fn len(&self) -> usize;
    
    /// コレクションが空かどうか
    fn is_empty(&self) -> bool;
    
    /// 型消去されたコレクションをクローン
    fn box_clone(&self) -> Box<dyn AnyHandlerCollection>;
    
    /// 任意の型へのダウンキャスト用にAny型へのアクセスを提供
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

impl<E: TypedEvent> AnyHandlerCollection for TypedHandlerCollection<E> {
    fn handle_any(&self, event: &dyn Any) {
        if let Some(typed_event) = event.downcast_ref::<E>() {
            self.handle(typed_event);
        }
    }
    
    fn remove(&mut self, handler_id: HandlerId) -> bool {
        TypedHandlerCollection::<E>::remove(self, handler_id)
    }
    
    fn clear(&mut self) {
        TypedHandlerCollection::<E>::clear(self)
    }
    
    fn len(&self) -> usize {
        self.handlers.len()
    }
    
    fn is_empty(&self) -> bool {
        self.handlers.is_empty()
    }
    
    fn box_clone(&self) -> Box<dyn AnyHandlerCollection> {
        Box::new(TypedHandlerCollection {
            handlers: self.handlers.clone(),
        })
    }
    
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self as &mut dyn Any
    }
} 