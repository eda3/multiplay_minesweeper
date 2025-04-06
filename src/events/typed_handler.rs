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

/// イベント処理結果の型定義
pub type EventResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

/// イベント処理の継続制御
pub enum EventControl {
    /// イベント処理を継続
    Continue,
    /// イベント処理を中断
    Break,
}

/// 型付きイベントハンドラ
/// 具体的なイベント型に対するハンドラ
pub struct TypedEventHandler<E: TypedEvent> {
    /// ハンドラの一意なID
    pub id: HandlerId,
    /// ハンドラの名前
    pub name: String,
    /// 実際のハンドラ関数
    pub handler: Box<dyn Fn(&E) -> EventControl + Send + Sync>,
    /// 優先度（低いほど先に処理）
    pub priority: u8,
    /// 有効状態
    pub enabled: Arc<Mutex<bool>>,
    /// 一度だけ実行するフラグ
    pub once: bool,
    /// エラーハンドラ
    error_handler: Option<Arc<dyn Fn(Box<dyn std::error::Error + Send + Sync>) + Send + Sync>>,
}

impl<E: TypedEvent> TypedEventHandler<E> {
    /// 新しい型付きイベントハンドラを作成
    pub fn new<F>(name: &str, handler: F) -> Self 
    where
        F: Fn(&E) -> EventControl + Send + Sync + 'static
    {
        Self {
            id: HandlerId::new::<E>(generate_id()),
            name: name.to_string(),
            handler: Box::new(handler),
            priority: 0,
            enabled: Arc::new(Mutex::new(true)),
            once: false,
            error_handler: None,
        }
    }
    
    /// 優先度を設定
    pub fn with_priority(mut self, priority: i32) -> Self {
        self.priority = priority as u8;
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
    
    /// エラーハンドラを設定
    pub fn with_error_handler<F>(mut self, handler: F) -> Self
    where
        F: Fn(Box<dyn std::error::Error + Send + Sync>) + Send + Sync + 'static
    {
        self.error_handler = Some(Arc::new(handler));
        self
    }
    
    /// エラーを処理
    fn handle_error(&self, error: Box<dyn std::error::Error + Send + Sync>) {
        if let Some(handler) = &self.error_handler {
            handler(error);
        } else {
            // デフォルトのエラー処理（ログ出力など）
            log::error!("ハンドラ '{}' でエラーが発生: {}", self.name, error);
        }
    }
    
    /// イベントを処理
    pub fn handle(&self, event: &E) -> EventControl {
        if *self.enabled.lock().unwrap() {
            // 実行して結果を返す
            let result = (self.handler)(event);
            
            // ワンショットフラグが立っている場合は無効化
            if self.once {
                *self.enabled.lock().unwrap() = false;
            }
            
            result
        } else {
            // 無効なハンドラは何もせず続行
            EventControl::Continue
        }
    }
    
    /// ハンドラ関数を取得
    pub fn handler(&self) -> &(dyn Fn(&E) -> EventControl + Send + Sync) {
        &*self.handler
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
            .field("has_error_handler", &self.error_handler.is_some())
            .finish()
    }
}

impl<E: TypedEvent> Clone for TypedEventHandler<E> {
    fn clone(&self) -> Self {
        unimplemented!("TypedEventHandlerをクローンできません - IDを使用して参照してください")
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
    fn handle_any(&self, event: &dyn Any) -> EventControl;
    
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
        self.priority as i32
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
    
    fn handle_any(&self, event: &dyn Any) -> EventControl {
        if let Some(typed_event) = event.downcast_ref::<E>() {
            self.handle(typed_event)
        } else {
            // 型が一致しない場合は処理をスキップして継続
            EventControl::Continue
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
        
        // ハンドラを挿入
        self.handlers.insert(pos, handler);
    }
    
    /// ハンドラを削除
    pub fn remove(&mut self, handler_id: HandlerId) -> bool {
        let len = self.handlers.len();
        self.handlers.retain(|h| h.id != handler_id);
        self.handlers.len() < len
    }
    
    /// イベントを処理
    pub fn handle(&self, event: &E) {
        for handler in &self.handlers {
            match handler.handle(event) {
                EventControl::Continue => continue,
                EventControl::Break => break,
            }
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

/// すべての型のハンドラに対応するトレイト
pub trait AnyHandlerCollection: Send + Sync {
    /// 型を消去したイベントを処理
    fn handle_any(&self, event: &dyn Any);
    
    /// ハンドラを削除
    fn remove_handler(&mut self, handler_id: HandlerId) -> bool;
    
    /// Any型への変換
    fn as_any(&self) -> &dyn Any;
    
    /// Any型への可変参照を取得
    fn as_any_mut(&mut self) -> &mut dyn Any;
    
    /// ハンドラの数を取得
    fn len(&self) -> usize;
    
    /// ハンドラが空かどうかを確認
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// TypedHandlerCollectionに対するAnyHandlerCollectionの実装
impl<E: TypedEvent> AnyHandlerCollection for TypedHandlerCollection<E> {
    fn handle_any(&self, event: &dyn Any) {
        // ダウンキャストを試みる
        if let Some(typed_event) = event.downcast_ref::<E>() {
            self.handle(typed_event);
        }
    }
    
    fn remove_handler(&mut self, handler_id: HandlerId) -> bool {
        self.remove(handler_id)
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

/// 型消去されたハンドラコレクションをダウンキャストするユーティリティ関数
pub fn downcast_handler_collection<E: TypedEvent>(
    collection: &mut Box<dyn AnyHandlerCollection + Send + Sync>
) -> Option<&mut TypedHandlerCollection<E>> {
    collection.as_any_mut().downcast_mut::<TypedHandlerCollection<E>>()
} 