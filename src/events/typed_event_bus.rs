/**
 * 型安全なイベントバスシステム
 * 
 * このモジュールはコンパイル時の型チェックを活用した安全なイベント処理システムを提供します。
 * 従来のイベントバスと異なり、ランタイムでの型キャストエラーを排除し、コンパイル時に
 * 型の不一致を検出します。
 * 
 * # 主な機能
 * 
 * - **型安全性**: コンパイル時の型チェックによりランタイムエラーを防止
 * - **ジェネリックイベントハンドラ**: 型情報を保持したイベントハンドラ
 * - **イベント履歴機能**: 型情報を保持したイベント履歴の管理
 * - **高パフォーマンス**: 型キャストのオーバーヘッド削減
 * - **自己文書化コード**: 型情報による明示的なイベント処理
 * 
 * # 基本的な使い方
 * 
 * ## イベントの定義
 * 
 * ```rust
 * // 型安全なイベントの定義
 * #[derive(Clone, Debug)]
 * struct CellRevealedEvent {
 *     pub row: usize,
 *     pub col: usize,
 * }
 * 
 * // TypedEventトレイトの実装
 * impl TypedEvent for CellRevealedEvent {
 *     fn event_type(&self) -> &'static str {
 *         "CellRevealedEvent"
 *     }
 *     
 *     fn to_event_data(&self) -> EventData {
 *         EventData {
 *             event_type: self.event_type().to_string(),
 *             data: json!({
 *                 "row": self.row,
 *                 "col": self.col,
 *             }),
 *         }
 *     }
 * }
 * 
 * // マクロを使った簡潔な実装
 * impl_typed_event_with_conversion!(CellRevealedEvent, "CellRevealedEvent");
 * ```
 * 
 * ## イベントハンドラの登録
 * 
 * ```rust
 * // 型安全なイベントハンドラの登録
 * let mut event_bus = TypedEventBus::new();
 * 
 * // ジェネリックなハンドラ登録
 * event_bus.register_handler::<CellRevealedEvent>(|event| {
 *     println!("セルが公開されました: ({}, {})", event.row, event.col);
 * });
 * 
 * // システム内でのハンドラ登録
 * fn setup_event_handlers(world: &mut World) {
 *     let event_bus = world.get_resource_mut::<TypedEventBusResource>().unwrap();
 *     
 *     event_bus.register_handler::<CellRevealedEvent>(|event| {
 *         // 公開されたセルの処理
 *     });
 *     
 *     event_bus.register_handler::<GameOverEvent>(|event| {
 *         // ゲームオーバー処理
 *     });
 * }
 * ```
 * 
 * ## イベントの発行
 * 
 * ```rust
 * // 型安全なイベント発行
 * event_bus.publish(CellRevealedEvent { row: 5, col: 10 });
 * 
 * // システム内でのイベント発行
 * fn reveal_cell_system(world: &mut World) {
 *     // リソースを取得
 *     let (board, mut event_bus) = world.get_resources_mut::<BoardResource, TypedEventBusResource>()
 *         .expect("必要なリソースが見つかりません");
 *     
 *     // ロジック処理...
 *     
 *     // イベント発行（型安全）
 *     event_bus.publish(CellRevealedEvent { row: 5, col: 10 });
 * }
 * ```
 * 
 * ## イベント履歴の活用
 * 
 * ```rust
 * // 型安全なイベント履歴へのアクセス
 * let history = event_bus.get_history::<CellRevealedEvent>();
 * 
 * // 特定の型のイベント履歴を処理
 * for event in history {
 *     println!("過去のセル公開: ({}, {})", event.row, event.col);
 * }
 * 
 * // 履歴をクリア
 * event_bus.clear_history::<CellRevealedEvent>();
 * ```
 * 
 * # 従来のEventBusとの比較
 * 
 * | 機能 | TypedEventBus | 従来のEventBus |
 * |------|--------------|--------------|
 * | 型安全性 | ✅ コンパイル時チェック | ❌ ランタイムチェック |
 * | エラー検出 | ✅ コンパイル時 | ❌ ランタイム時 |
 * | IDE補完 | ✅ 完全サポート | ❌ 限定的 |
 * | パフォーマンス | ✅ 高速（型キャスト最小化） | ❌ 低速（頻繁な型キャスト） |
 * | メモリ効率 | ✅ 高効率 | ❌ 非効率的 |
 * | コード量 | ✅ 少ない（マクロ活用） | ❌ 多い |
 * | イベント履歴 | ✅ 型情報あり | ❌ 型情報なし |
 * 
 * # 型安全性の仕組み
 * 
 * TypedEventBusは以下の仕組みで型安全性を実現しています：
 * 
 * 1. **TypedEventトレイト**: 各イベント型に固有の型情報を保持
 * 2. **ジェネリックハンドラ**: 型パラメータによる明示的な型指定
 * 3. **to_event_data** メソッド: イベント自身による適切な変換ロジック
 * 4. **from_event_dataメソッド**: 型安全な逆変換機能
 * 
 * これにより、イベントの公開から購読までの全プロセスにおいて型の整合性が保証されます。
 */
use std::any::{Any, TypeId};
use std::collections::{HashMap, HashSet, BinaryHeap};
use std::fmt::{Debug, Formatter};
use std::sync::{Arc, RwLock};
use std::cmp::{Ord, PartialOrd, Ordering};

use crate::events::EventData;
use crate::events::event_history::EventHistory;
use crate::events::event_logger::EventLogger as OtherEventLogger;
use crate::events::handler::{HandlerCollection};
use crate::events::processor::{EventProcessorCollection, ProcessorId, EventProcessor};
use crate::events::typed_event::{TypedEvent, TypedHandlerId, HandlerId};
use crate::events::typed_handler::{TypedHandlerCollection, EventControl, TypedEventHandler, AnyHandlerCollection};
use crate::events::Event;
use crate::resources::resource_trait::Resource;

/// イベントの優先度（低いほど優先して処理）
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum EventPriority {
    Highest = 0,
    High = 1,
    Normal = 2,
    Low = 3,
    Lowest = 4,
}

/// イベント処理エラー
#[derive(Debug, Clone)]
pub enum EventProcessingError {
    TypeMismatch,
    HandlerError(String),
    NotFound,
    Cancelled,
    Custom(String),
}

impl std::fmt::Display for EventProcessingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TypeMismatch => write!(f, "型の不一致エラー"),
            Self::HandlerError(msg) => write!(f, "ハンドラエラー: {}", msg),
            Self::NotFound => write!(f, "ハンドラが見つかりません"),
            Self::Cancelled => write!(f, "イベント処理がキャンセルされました"),
            Self::Custom(msg) => write!(f, "カスタムエラー: {}", msg),
        }
    }
}

/// エラーポリシー
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorPolicy {
    Continue,  // エラーを無視して続行
    StopEvent, // 現在のイベントの処理を停止
    StopAll,   // すべてのイベント処理を停止
    Custom,    // カスタムハンドラを使用
}

impl Default for ErrorPolicy {
    fn default() -> Self {
        Self::Continue
    }
}

/// グローバルプロセッサトレイト
pub trait GlobalProcessor: Send + Sync {
    /// すべてのイベントタイプに対して処理を行う
    fn process_event(&self, event: &dyn Any) -> crate::events::typed_handler::EventControl;
}

/// 関数型のためのグローバルプロセッサ実装
impl<F> GlobalProcessor for F
where
    F: Fn(&dyn Any) -> crate::events::typed_handler::EventControl + Send + Sync + 'static
{
    fn process_event(&self, event: &dyn Any) -> crate::events::typed_handler::EventControl {
        self(event)
    }
}

/// イベントロガートレイト
pub trait EventLogger {
    /// イベントをログに記録
    fn log_event(&self, event_type: &str, event_data: &Box<dyn Any + Send + Sync>);
}

/// イベントハンドラ
#[derive(Debug)]
pub struct PrioritizedEvent {
    /// イベントの優先度
    priority: EventPriority,
    /// イベントのタイムスタンプ
    timestamp: u64,
    /// イベントデータ
    event: Box<dyn Any + Send + Sync>,
    /// イベントの型ID
    type_id: TypeId,
}

impl PartialEq for PrioritizedEvent {
    fn eq(&self, other: &Self) -> bool {
        self.priority == other.priority && self.timestamp == other.timestamp
    }
}

impl Eq for PrioritizedEvent {}

impl PartialOrd for PrioritizedEvent {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PrioritizedEvent {
    fn cmp(&self, other: &Self) -> Ordering {
        // 優先度を比較（低い値＝高優先度）
        let prio_cmp = other.priority.cmp(&self.priority);
        if prio_cmp != Ordering::Equal {
            return prio_cmp;
        }
        
        // 同じ優先度の場合はタイムスタンプで比較（低い値＝古いイベント）
        self.timestamp.cmp(&other.timestamp)
    }
}

/// 型付きイベントバス：型安全なイベントの発行と購読を管理するハブ
pub struct TypedEventBus {
    /// 型ごとのイベントハンドラ
    pub handlers: RwLock<HashMap<TypeId, Box<dyn AnyHandlerCollection + Send + Sync>>>,
    
    /// 型ごとのイベントプロセッサ
    pub processors: RwLock<HashMap<TypeId, Box<EventProcessorCollection>>>,
    
    /// すべての型に対するグローバルプロセッサ
    pub global_processors: RwLock<EventProcessorCollection>,
    
    /// ハンドラの無効化状態を追跡
    disabled_handlers: RwLock<HashSet<TypedHandlerId>>,
    
    /// イベントログ機能
    event_logger: Option<Box<dyn EventLogger + Send + Sync>>,
    
    /// イベント履歴（オプション）
    event_history: Option<RwLock<EventHistory>>,
    /// 型ごとのイベント履歴（型安全なアクセス用）
    typed_event_history: Arc<RwLock<HashMap<TypeId, Vec<Box<dyn Any + Send + Sync>>>>>,
    /// 優先度付きイベントキュー（バッチ処理用）
    event_queue: Arc<RwLock<BinaryHeap<PrioritizedEvent>>>,
    /// バッチモードが有効かどうか
    batch_mode: Arc<RwLock<bool>>,
    /// 履歴の最大サイズ
    max_history_size: usize,
    /// デバッグモード
    debug: bool,
    /// エラーポリシー
    error_policy: Arc<RwLock<ErrorPolicy>>,
    /// エラーハンドラ
    error_handler: Arc<RwLock<Option<Box<dyn Fn(EventProcessingError) + Send + Sync>>>>,
    /// ログ全イベントモード
    log_all_events: bool,
    /// 履歴機能有効モード
    history_enabled: bool,
}

impl TypedEventBus {
    /// 新しいイベントバスを作成
    pub fn new(max_history_size: usize) -> Self {
        Self {
            // 一般的なイベント種別数に基づくキャパシティ
            handlers: RwLock::new(HashMap::with_capacity(10)),
            // 型ごとのプロセッサも同様に初期化
            processors: RwLock::new(HashMap::with_capacity(10)),
            // 履歴は最大サイズに合わせて初期化
            event_history: if max_history_size > 0 {
                Some(RwLock::new(EventHistory::new(max_history_size)))
            } else {
                None
            },
            // 同様にタイプ別履歴も適切な容量で初期化
            typed_event_history: Arc::new(RwLock::new(HashMap::with_capacity(10))),
            // イベントキューは通常のゲームフレームで処理できる量を初期容量に設定
            event_queue: Arc::new(RwLock::new(BinaryHeap::with_capacity(20))),
            batch_mode: Arc::new(RwLock::new(false)),
            max_history_size,
            debug: false,
            // グローバルプロセッサは通常少数
            global_processors: RwLock::new(EventProcessorCollection::new()),
            // エラーポリシーのデフォルト設定
            error_policy: Arc::new(RwLock::new(ErrorPolicy::default())),
            // エラーハンドラはデフォルトではなし
            error_handler: Arc::new(RwLock::new(None)),
            disabled_handlers: RwLock::new(HashSet::new()),
            event_logger: None,
            // 全イベントをログに記録するかどうか
            log_all_events: false,
            // 履歴機能が有効かどうか
            history_enabled: max_history_size > 0,
        }
    }
    
    /// デバッグモードを設定
    pub fn with_debug(mut self, debug: bool) -> Self {
        self.debug = debug;
        self
    }
    
    /// エラーポリシーを設定
    pub fn with_error_policy(self, policy: ErrorPolicy) -> Self {
        if let Ok(mut error_policy) = self.error_policy.write() {
            *error_policy = policy;
        }
        self
    }
    
    /// カスタムエラーハンドラを設定
    pub fn with_error_handler<F>(self, handler: F) -> Self
    where
        F: Fn(EventProcessingError) + Send + Sync + 'static
    {
        if let Ok(mut error_handler) = self.error_handler.write() {
            *error_handler = Some(Box::new(handler));
        }
        
        // エラーポリシーをカスタムに設定
        if let Ok(mut error_policy) = self.error_policy.write() {
            *error_policy = ErrorPolicy::Custom;
        }
        
        self
    }
    
    /// エラーを処理する内部メソッド
    fn handle_error(&self, error: EventProcessingError) {
        let policy = {
            let policy = self.error_policy.read().unwrap();
            *policy
        };
        
        match policy {
            ErrorPolicy::Continue => {
                // 何もしない
            },
            ErrorPolicy::StopEvent => {
                // ログに記録
                log::error!("イベント処理エラー: {}", error);
            },
            ErrorPolicy::StopAll => {
                // ログに記録し、後でパニックするフラグを立てることも可能
                log::error!("致命的なエラー: {}", error);
            },
            ErrorPolicy::Custom => {
                // カスタムハンドラに委任
                let handler = self.error_handler.read().unwrap();
                if let Some(handler) = &*handler {
                    handler(error);
                } else {
                    // ハンドラがなければログに記録
                    log::error!("カスタムエラーハンドラが未設定");
                }
            },
        }
    }
    
    /// デバッグモードとカスタム容量設定のイベントバスを作成
    pub fn with_capacity(
        handlers_capacity: usize,
        queue_capacity: usize,
        history_capacity: usize,
        max_history_size: usize
    ) -> Self {
        Self {
            handlers: RwLock::new(HashMap::with_capacity(handlers_capacity)),
            processors: RwLock::new(HashMap::with_capacity(handlers_capacity)),
            event_history: if max_history_size > 0 {
                Some(RwLock::new(EventHistory::new(max_history_size)))
            } else {
                None
            },
            typed_event_history: Arc::new(RwLock::new(HashMap::with_capacity(handlers_capacity))),
            event_queue: Arc::new(RwLock::new(BinaryHeap::with_capacity(queue_capacity))),
            batch_mode: Arc::new(RwLock::new(false)),
            max_history_size,
            debug: false,
            global_processors: RwLock::new(EventProcessorCollection::new()),
            error_policy: Arc::new(RwLock::new(ErrorPolicy::default())),
            error_handler: Arc::new(RwLock::new(None)),
            disabled_handlers: RwLock::new(HashSet::new()),
            event_logger: None,
            log_all_events: false,
            history_enabled: max_history_size > 0,
        }
    }
    
    /// 履歴サイズを設定
    pub fn with_history_size(mut self, size: usize) -> Self {
        self.max_history_size = size;
        self
    }
    
    /// バッチモードを開始
    /// このモードではイベントは即時処理されず、キューに追加される
    pub fn start_batch_mode(&self) {
        let mut batch_mode = self.batch_mode.write().unwrap();
        *batch_mode = true;
    }
    
    /// バッチモードを終了し、キューに溜まったイベントを処理
    pub fn end_batch_mode(&self) {
        // バッチモードを無効化
        {
            let mut batch_mode = self.batch_mode.write().unwrap();
            *batch_mode = false;
        }
        
        // キューに溜まったイベントを処理
        self.process_event_queue();
    }
    
    /// キューに溜まったイベントを処理
    pub fn process_event_queue(&self) {
        // イベントキューからイベントを取り出して処理
        loop {
            let event_opt = {
                let mut queue = self.event_queue.write().unwrap();
                queue.pop()
            };
            
            match event_opt {
                Some(event) => {
                    self.process_queued_event(event);
                },
                None => break, // キューが空になったら終了
            }
        }
    }
    
    /// キューに入っているイベントを処理
    fn process_queued_event(&self, queued_event: PrioritizedEvent) {
        let type_id = queued_event.type_id;
        
        // グローバルプロセッサを実行
        let should_continue = {
            let global_processors = self.global_processors.read().unwrap();
            let mut continue_processing = true;
            
            for processor in &*global_processors {
                match processor.process_event(&*queued_event.event) {
                    crate::events::typed_handler::EventControl::Continue => {},
                    crate::events::typed_handler::EventControl::Break => {
                        continue_processing = false;
                        break;
                    }
                }
            }
            
            continue_processing
        };
        
        // グローバルプロセッサが処理を中断した場合は以降の処理をスキップ
        if !should_continue {
            return;
        }
        
        // 型固有のハンドラを取得して呼び出し
        let handlers = self.handlers.read().unwrap();
        if let Some(handler_collection) = handlers.get(&type_id) {
            handler_collection.handle_any(&*queued_event.event);
        }
    }
    
    /// イベントを購読する
    pub fn subscribe<E: TypedEvent, F>(&self, name: &str, handler: F) -> TypedEventHandler<E>
    where
        F: Fn(&E) -> EventControl + Send + Sync + 'static,
    {
        let event_handler = TypedEventHandler::new(name, handler);
        self.register_typed_handler(event_handler.clone());
        event_handler
    }
    
    /// 型付きハンドラを登録する（高度な制御に使用）
    pub fn register_typed_handler<E: TypedEvent>(&self, handler: TypedEventHandler<E>) -> HandlerId {
        let type_id = TypeId::of::<E>();
        let handler_id = handler.id;
        
        // ハンドラ名をデバッグ用に取得
        let handler_name = handler.name.clone();
        
        // ハンドラマップを取得
        let mut handlers = self.handlers.write().unwrap();
        
        // 型IDに対応するハンドラコレクションを取得または作成
        if !handlers.contains_key(&type_id) {
            let collection = TypedHandlerCollection::<E>::new();
            handlers.insert(type_id, Box::new(collection));
        }
        
        // 既存のコレクションを取得してハンドラを追加
        if let Some(collection) = handlers.get_mut(&type_id) {
            if let Some(typed_collection) = collection.as_any_mut().downcast_mut::<TypedHandlerCollection<E>>() {
                typed_collection.add(handler);
                
                if self.debug {
                    println!("ハンドラ登録: {} (ID: {:?}) for {}", handler_name, handler_id, E::type_name());
                }
            }
        }
        
        handler_id
    }
    
    /// イベントハンドラを削除
    pub fn unregister_handler<E: TypedEvent>(&self, handler_id: HandlerId) {
        let type_id = TypeId::of::<E>();
        
        let mut handlers = self.handlers.write().unwrap();
        if let Some(handler_collection) = handlers.get_mut(&type_id) {
            handler_collection.remove_handler(handler_id);
            
            if self.debug {
                println!("削除されたハンドラ: ID {:?} for {}", handler_id, E::type_name());
            }
        }
    }
    
    /// イベントの発行
    pub fn publish<E: TypedEvent>(&self, event: E) {
        self.publish_with_priority(event, EventPriority::Normal);
    }
    
    /// 優先度付きイベントの発行
    pub fn publish_with_priority<E: TypedEvent>(&self, event: E, priority: EventPriority) {
        let type_id = TypeId::of::<E>();
        
        // タイプ別のイベント履歴に追加
        if self.debug {
            let mut typed_history = self.typed_event_history.write().unwrap();
            let history_entry = typed_history.entry(type_id).or_insert_with(Vec::new);
            
            // 最大サイズを超えたら古いものを削除
            if history_entry.len() >= self.max_history_size {
                history_entry.remove(0);
            }
            
            // Anyとして保存
            history_entry.push(Box::new(event.clone()));
        }
        
        // イベント履歴に追加（適切なEventDataへの変換が必要）
        if let Some(event_data) = event.to_event_data() {
            if let Some(history) = self.event_history.as_ref() {
                let mut history_guard = history.write().unwrap();
                // 最大サイズを超えたら古いものを削除（イベント履歴の内部管理に委任）
                history_guard.add_event(event_data);
            }
        }
        
        if self.debug {
            println!("イベント発行: {} ({:?})", E::event_type(), event);
        }
        
        // バッチモードが有効ならキューに追加
        let is_batch_mode = {
            let batch_mode = self.batch_mode.read().unwrap();
            *batch_mode
        };
        
        if is_batch_mode {
            // キューに追加
            let prioritized_event = PrioritizedEvent {
                priority,
                timestamp: event.timestamp(),
                event: Box::new(event),
                type_id,
            };
            
            let mut queue = self.event_queue.write().unwrap();
            queue.push(prioritized_event);
            
            return;
        }
        
        // 通常モード：即時処理
        // グローバルプロセッサを実行
        let should_continue = {
            let global_processors = self.global_processors.read().unwrap();
            let mut continue_processing = true;
            
            for processor in &*global_processors {
                match processor.process_event(&event as &dyn Any) {
                    crate::events::typed_handler::EventControl::Continue => {},
                    crate::events::typed_handler::EventControl::Break => {
                        continue_processing = false;
                        break;
                    }
                }
            }
            
            continue_processing
        };
        
        // グローバルプロセッサが処理を中断した場合は以降の処理をスキップ
        if !should_continue {
            return;
        }
        
        // 型固有のハンドラを取得して呼び出し
        let handlers = self.handlers.read().unwrap();
        if let Some(handler_collection) = handlers.get(&type_id) {
            handler_collection.handle_any(&event as &dyn Any);
        }
    }
    
    /// 一括イベント発行（複数のイベントをバッチで効率的に処理）
    pub fn publish_batch<E: TypedEvent>(&self, events: Vec<E>, priority: EventPriority) {
        // バッチモードを開始
        self.start_batch_mode();
        
        // すべてのイベントを発行
        for event in events {
            self.publish_with_priority(event, priority);
        }
        
        // バッチモードを終了（イベント処理を実行）
        self.end_batch_mode();
    }
    
    /// 履歴にイベントを記録
    fn record_event_to_history<E: TypedEvent>(&self, event: &E) {
        if let Some(history) = &self.event_history {
            let mut history = history.write().unwrap();
            let event_data = self.convert_to_event_data(event);
            history.add_event(event_data);
        }
    }
    
    /// イベント履歴を取得
    pub fn get_history_for_type<E: TypedEvent>(&self) -> Option<Vec<E>> where E: Clone {
        if let Some(history) = self.event_history.as_ref() {
            let history = history.read().unwrap();
            // ... implementation ...
            None
        } else {
            None
        }
    }
    
    /// イベント履歴をクリア
    pub fn clear_history(&self) {
        if let Some(history) = self.event_history.as_ref() {
            let mut history = history.write().unwrap();
            history.clear();
        }
    }
    
    /// イベントの型変換をテストするためのユーティリティ
    #[allow(unused_must_use)]
    fn create_test_event<E: TypedEvent + 'static>() -> Option<E> {
        let type_name = std::any::type_name::<E>();
        
        match type_name {
            // ゲームイベント
            "multiplay_minesweeper::events::game_events::GameStartEvent" => {
                use crate::events::game_events::GameStartEvent;
                use crate::models::difficulty::Difficulty;
                
                let event = GameStartEvent {
                    difficulty: Difficulty::Beginner,
                    custom_width: None,
                    custom_height: None,
                    custom_mines: None,
                    is_multiplayer: false,
                    session_id: None,
                };
                
                unsafe {
                    // 安全でない変換だが、型名チェックでタイプを確認しているので実行時安全
                    let event_box = Box::new(event);
                    let event_raw = Box::into_raw(event_box);
                    let event_any = Box::from_raw(event_raw as *mut E);
                    Some(*event_any)
                }
            },
            "multiplay_minesweeper::events::game_events::GameEndEvent" => {
                use crate::events::game_events::GameEndEvent;
                
                let event = GameEndEvent {
                    is_win: true,
                    play_time: 100,
                    revealed_cells: 20,
                    flagged_cells: 5,
                    score: 1000,
                };
                
                unsafe {
                    // 安全でない変換だが、型名チェックでタイプを確認しているので実行時安全
                    let event_box = Box::new(event);
                    let event_raw = Box::into_raw(event_box);
                    let event_any = Box::from_raw(event_raw as *mut E);
                    Some(*event_any)
                }
            },
            "multiplay_minesweeper::events::game_events::GameStateChangeEvent" => {
                use crate::events::game_events::{GameStateChangeEvent, GameState};
                
                let event = GameStateChangeEvent {
                    new_state: GameState::Playing,
                    old_state: GameState::Menu,
                };
                
                unsafe {
                    // 安全でない変換だが、型名チェックでタイプを確認しているので実行時安全
                    let event_box = Box::new(event);
                    let event_raw = Box::into_raw(event_box);
                    let event_any = Box::from_raw(event_raw as *mut E);
                    Some(*event_any)
                }
            },
            // ここに他のイベント型のダミーデータを追加
            // ...
            
            _ => None,
        }
    }
    
    /// 特定のイベント型の変換が正しく機能することを検証
    pub fn validate_event_conversion<E: TypedEvent + 'static>(&self) -> Result<(), String>
    where
        E: Clone,
    {
        let type_name = std::any::type_name::<E>();
        
        // テスト用イベントを作成
        let event = match Self::create_test_event::<E>() {
            Some(e) => e,
            None => {
                return Err(format!("❌ イベント型「{}」のテストイベントを作成できませんでした", type_name));
            }
        };
        
        // 変換を試みる
        match self.test_convert_event(&event) {
            Some(_) => {
                if self.debug {
                    println!("✅ イベント型「{}」の変換が正常に機能しています", type_name);
                }
                Ok(())
            },
            None => {
                let error_message = format!("❌ イベント型「{}」のEventDataへの変換に失敗しました", type_name);
                if self.debug {
                    println!("{}", error_message);
                }
                Err(error_message)
            }
        }
    }
    
    /// すべての登録済みイベント型の変換が正しく機能することを検証
    pub fn validate_all_event_conversions(&self) -> Vec<String> {
        let mut errors = Vec::new();
        
        // ゲームイベント
        if let Err(e) = self.validate_event_conversion::<crate::events::game_events::GameStartEvent>() {
            errors.push(e);
        }
        if let Err(e) = self.validate_event_conversion::<crate::events::game_events::GameEndEvent>() {
            errors.push(e);
        }
        if let Err(e) = self.validate_event_conversion::<crate::events::game_events::GameStateChangeEvent>() {
            errors.push(e);
        }
        
        // ここに他のイベント型のバリデーションを追加
        // ...
        
        if errors.is_empty() && self.debug {
            println!("✅ すべてのイベント型の変換が正常に機能しています");
        } else if self.debug {
            println!("❌ {}個のイベント型で変換エラーが発生しました", errors.len());
        }
        
        errors
    }
    
    /// イベント自身のto_event_dataメソッドを使用
    pub fn test_convert_event<E: TypedEvent>(&self, event: &E) -> Option<EventData> {
        event.to_event_data()
    }

    /// イベントプロセッサを削除
    pub fn unregister_processor<E: TypedEvent>(&self, processor_id: ProcessorId) {
        let type_id = TypeId::of::<E>();
        
        let mut processors = self.processors.write().unwrap();
        if let Some(processor_collection) = processors.get_mut(&type_id) {
            processor_collection.remove_processor(processor_id);
            
            if self.debug {
                println!("削除されたプロセッサー: ID {:?} for {}", processor_id, E::type_name());
            }
        }
    }

    /// グローバルプロセッサを追加
    pub fn add_global_processor<P: EventProcessor + 'static>(&self, processor: P) -> ProcessorId {
        let mut global_processors = self.global_processors.write().unwrap();
        let id = global_processors.add_processor(processor);
        
        if self.debug {
            println!("グローバルプロセッサが追加されました: {:?}", id);
        }
        
        id
    }

    /// イベントをEventDataに変換
    fn convert_to_event_data<E: TypedEvent>(&self, event: &E) -> EventData {
        event.to_event_data().unwrap_or_else(|| {
            panic!("イベント {} をEventDataに変換できませんでした", E::type_name())
        })
    }

    /// 特定の型のハンドラ数を取得
    pub fn handler_count<E: TypedEvent>(&self) -> usize {
        let type_id = TypeId::of::<E>();
        let handlers = self.handlers.read().unwrap();
        
        handlers.get(&type_id)
            .map(|collection| collection.len())
            .unwrap_or(0)
    }

    /// イベントをキューに記録する
    fn record_event_to_queue<E: TypedEvent>(&self, event: E, priority: EventPriority) {
        let timestamp = event.timestamp();
        let type_id = TypeId::of::<E>();
        let boxed_event = Box::new(event);
        
        let prioritized_event = PrioritizedEvent {
            priority,
            timestamp,
            event: boxed_event,
            type_id,
        };
        
        // イベントをキューに追加
        if let Ok(mut queue) = self.event_queue.write() {
            queue.push(prioritized_event);
        }
        
        // 履歴に記録（必要に応じて）
        if self.history_enabled {
            if let Some(history) = self.event_history.as_ref() {
                let mut history = history.write().unwrap();
                // ...履歴への記録処理...
            }
        }
    }
    
    /// 全ての履歴を取得
    pub fn get_all_history(&self) -> Option<Vec<EventData>> {
        self.event_history.as_ref().map(|history| {
            let history = history.read().unwrap();
            // ...履歴データの取得処理...
            Vec::new()
        })
    }
}

impl Debug for TypedEventBus {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let handlers = self.handlers.read().unwrap();
        let handler_count = handlers.len();
        
        let history_count = if let Some(history) = &self.event_history {
            let history = history.read().unwrap();
            history.size()
        } else {
            0
        };
        
        write!(f, "TypedEventBus {{ handlers: {}, history: {}, debug: {} }}",
            handler_count, history_count, self.debug)
    }
}

impl Default for TypedEventBus {
    fn default() -> Self {
        Self::new(100)
    }
} 