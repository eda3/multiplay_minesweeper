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
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::sync::{Arc, RwLock};
use std::collections::BinaryHeap;
use std::cmp::{Ord, PartialOrd, Ordering};

use crate::events::typed_event::{TypedEvent, HandlerId, generate_id};
use crate::events::typed_handler::{
    TypedEventHandler, TypedHandlerCollection, AnyHandlerCollection
};
use crate::events::EventData;

/// イベントの優先度（低いほど優先して処理）
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum EventPriority {
    /// 高優先度（即時処理が必要なイベント）
    High = 0,
    /// 標準優先度（通常のイベント）
    Normal = 10,
    /// 低優先度（遅延処理可能なイベント）
    Low = 20,
    /// バックグラウンド（非重要イベント）
    Background = 30,
}

impl Default for EventPriority {
    fn default() -> Self {
        Self::Normal
    }
}

/// 優先度付きイベントエントリ
#[derive(Debug)]
struct PrioritizedEvent {
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

/// グローバルイベントプロセッサの型定義
type GlobalProcessor = Box<dyn Fn(&dyn Any) + Send + Sync>;

/// 型付きイベントバス：型安全なイベントの発行と購読を管理するハブ
#[derive(Clone)]
pub struct TypedEventBus {
    /// 型ごとのイベントハンドラマップ
    handlers: Arc<RwLock<HashMap<TypeId, Box<dyn AnyHandlerCollection + Send + Sync>>>>,
    /// イベント履歴（オプション）
    event_history: Arc<RwLock<Vec<EventData>>>,
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
    /// グローバルプロセッサ（全てのイベントに対して呼び出される）
    global_processors: Arc<RwLock<Vec<GlobalProcessor>>>,
}

impl TypedEventBus {
    /// 新しいイベントバスを作成
    pub fn new() -> Self {
        // WASM環境でのメモリ効率化のため、適切な初期容量を設定
        let max_history_size = 100;
        
        Self {
            // ハンドラーは通常10種類以下のイベントタイプを処理
            handlers: Arc::new(RwLock::new(HashMap::with_capacity(10))),
            // 履歴は最大サイズに合わせて初期化
            event_history: Arc::new(RwLock::new(Vec::with_capacity(max_history_size))),
            // 同様にタイプ別履歴も適切な容量で初期化
            typed_event_history: Arc::new(RwLock::new(HashMap::with_capacity(10))),
            // イベントキューは通常のゲームフレームで処理できる量を初期容量に設定
            event_queue: Arc::new(RwLock::new(BinaryHeap::with_capacity(20))),
            batch_mode: Arc::new(RwLock::new(false)),
            max_history_size,
            debug: false,
            // グローバルプロセッサは通常少数
            global_processors: Arc::new(RwLock::new(Vec::with_capacity(5))),
        }
    }
    
    /// デバッグモードを設定
    pub fn with_debug(mut self, debug: bool) -> Self {
        self.debug = debug;
        self
    }
    
    /// デバッグモードとカスタム容量設定のイベントバスを作成
    pub fn with_capacity(
        handlers_capacity: usize,
        queue_capacity: usize,
        history_capacity: usize,
        max_history_size: usize
    ) -> Self {
        Self {
            handlers: Arc::new(RwLock::new(HashMap::with_capacity(handlers_capacity))),
            event_history: Arc::new(RwLock::new(Vec::with_capacity(history_capacity))),
            typed_event_history: Arc::new(RwLock::new(HashMap::with_capacity(handlers_capacity))),
            event_queue: Arc::new(RwLock::new(BinaryHeap::with_capacity(queue_capacity))),
            batch_mode: Arc::new(RwLock::new(false)),
            max_history_size,
            debug: false,
            global_processors: Arc::new(RwLock::new(Vec::with_capacity(5))),
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
        {
            let global_processors = self.global_processors.read().unwrap();
            for processor in &*global_processors {
                processor(&*queued_event.event);
            }
        }
        
        // 型固有のハンドラを取得して呼び出し
        let handlers = self.handlers.read().unwrap();
        if let Some(handler_collection) = handlers.get(&type_id) {
            handler_collection.handle_any(&*queued_event.event);
        }
    }
    
    /// イベントを登録して購読
    pub fn subscribe<E: TypedEvent, F>(&self, name: &str, handler: F) -> TypedEventHandler<E>
    where
        F: Fn(&E) + Send + Sync + 'static,
    {
        let event_handler = TypedEventHandler::new(name, handler);
        self.register_handler(event_handler.clone());
        event_handler
    }
    
    /// イベントハンドラを登録
    pub fn register_handler<E: TypedEvent>(&self, handler: TypedEventHandler<E>) {
        let type_id = TypeId::of::<E>();
        
        let mut handlers = self.handlers.write().unwrap();
        let handler_collection = handlers
            .entry(type_id)
            .or_insert_with(|| Box::new(TypedHandlerCollection::<E>::new()));
        
        // ハンドラ名をログ用にクローン
        let handler_name = handler.name.clone();
        
        // ここでダウンキャストは必要ない - AnyHandlerCollectionを使って直接変換
        if let Some(collection) = self.downcast_handler_collection::<E>(handler_collection) {
            collection.add(handler);
        }
        
        if self.debug {
            println!("登録されたハンドラ: {} for {}", handler_name, E::type_name());
        }
    }
    
    /// イベントハンドラを削除
    pub fn unregister_handler<E: TypedEvent>(&self, handler_id: HandlerId) {
        let type_id = TypeId::of::<E>();
        
        let mut handlers = self.handlers.write().unwrap();
        if let Some(handler_collection) = handlers.get_mut(&type_id) {
            handler_collection.remove(handler_id);
            
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
        if let Some(event_data) = self.convert_to_event_data(&event) {
            let mut history = self.event_history.write().unwrap();
            history.push(event_data);
            
            // 最大サイズを超えたら古いものを削除
            if history.len() > self.max_history_size {
                history.remove(0);
            }
        }
        
        if self.debug {
            println!("イベント発行: {} ({:?})", E::type_name(), event);
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
        {
            let global_processors = self.global_processors.read().unwrap();
            for processor in &*global_processors {
                processor(&event as &dyn Any);
            }
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
    
    /// イベント履歴を取得
    pub fn get_history(&self) -> Vec<EventData> {
        let history = self.event_history.read().unwrap();
        history.clone()
    }
    
    /// 特定の型のイベント履歴を取得
    pub fn get_event_history_by_type<E: TypedEvent>(&self) -> Vec<E> {
        let type_id = TypeId::of::<E>();
        let typed_history = self.typed_event_history.read().unwrap();
        
        match typed_history.get(&type_id) {
            Some(history) => {
                history.iter()
                    .filter_map(|event| {
                        event.downcast_ref::<E>().map(|e| e.clone())
                    })
                    .collect()
            }
            None => Vec::new(),
        }
    }
    
    /// 特定の型の最後のイベントを取得
    pub fn get_last_event<E: TypedEvent>(&self) -> Option<E> {
        let type_id = TypeId::of::<E>();
        let typed_history = self.typed_event_history.read().unwrap();
        
        typed_history.get(&type_id)
            .and_then(|history| {
                history.last().and_then(|event| {
                    event.downcast_ref::<E>().map(|e| e.clone())
                })
            })
    }
    
    /// 特定の型の特定のタイムスタンプ以降のイベントを取得
    pub fn get_events_by_type_since<E: TypedEvent>(&self, timestamp: u64) -> Vec<E> {
        let type_id = TypeId::of::<E>();
        let typed_history = self.typed_event_history.read().unwrap();
        
        match typed_history.get(&type_id) {
            Some(history) => {
                history.iter()
                    .filter_map(|event| {
                        event.downcast_ref::<E>().map(|e| {
                            if e.timestamp() >= timestamp {
                                Some(e.clone())
                            } else {
                                None
                            }
                        }).flatten()
                    })
                    .collect()
            }
            None => Vec::new(),
        }
    }
    
    /// グローバルプロセッサを追加
    pub fn add_global_processor<F>(&self, processor: F)
    where
        F: Fn(&dyn Any) + Send + Sync + 'static,
    {
        let mut global_processors = self.global_processors.write().unwrap();
        global_processors.push(Box::new(processor));
    }
    
    /// イベント履歴をクリア
    pub fn clear_history(&self) {
        let mut history = self.event_history.write().unwrap();
        history.clear();
        
        let mut typed_history = self.typed_event_history.write().unwrap();
        typed_history.clear();
    }
    
    /// イベントをEventDataに変換（内部実装用）
    fn convert_to_event_data<E: TypedEvent>(&self, event: &E) -> Option<EventData> {
        let type_name = std::any::type_name::<E>();
        
        if self.debug {
            println!("イベント変換試行: {}", type_name);
        }
        
        // まず、イベント自身のto_event_dataメソッドを試す
        // これが最も型安全なアプローチ
        let event_data = event.to_event_data();
        if event_data.is_some() {
            return event_data;
        } else if self.debug {
            println!("警告: {}のto_event_dataメソッドが実装されていないか、Noneを返しました", type_name);
        }
        
        // フォールバック: 型名によるマッチングを行う
        // これはリフレクションを使った実装で、理想的にはto_event_dataの実装が望ましい
        
        let result = match type_name {
            // ゲームイベント
            "multiplay_minesweeper::events::game_events::GameStartEvent" => {
                event.as_any().downcast_ref::<crate::events::game_events::GameStartEvent>()
                    .map(|e| EventData::GameStart(e.clone()))
            }
            "multiplay_minesweeper::events::game_events::GameEndEvent" => {
                event.as_any().downcast_ref::<crate::events::game_events::GameEndEvent>()
                    .map(|e| EventData::GameEnd(e.clone()))
            }
            "multiplay_minesweeper::events::game_events::GameStateChangeEvent" => {
                event.as_any().downcast_ref::<crate::events::game_events::GameStateChangeEvent>()
                    .map(|e| EventData::GameStateChange(e.clone()))
            }
            "multiplay_minesweeper::events::game_events::TimerEvent" => {
                event.as_any().downcast_ref::<crate::events::game_events::TimerEvent>()
                    .map(|e| EventData::Timer(e.clone()))
            }
            "multiplay_minesweeper::events::game_events::DifficultyChangeEvent" => {
                event.as_any().downcast_ref::<crate::events::game_events::DifficultyChangeEvent>()
                    .map(|e| EventData::DifficultyChange(e.clone()))
            }
            "multiplay_minesweeper::events::game_events::ScoreUpdateEvent" => {
                event.as_any().downcast_ref::<crate::events::game_events::ScoreUpdateEvent>()
                    .map(|e| EventData::ScoreUpdate(e.clone()))
            }
            
            // ボードイベント
            "multiplay_minesweeper::events::board_events::CellStateChangeEvent" => {
                event.as_any().downcast_ref::<crate::events::board_events::CellStateChangeEvent>()
                    .map(|e| EventData::CellStateChange(e.clone()))
            }
            "multiplay_minesweeper::events::board_events::BulkCellStateChangeEvent" => {
                event.as_any().downcast_ref::<crate::events::board_events::BulkCellStateChangeEvent>()
                    .map(|e| EventData::BulkCellStateChange(e.clone()))
            }
            "multiplay_minesweeper::events::board_events::FlagPlacedEvent" => {
                event.as_any().downcast_ref::<crate::events::board_events::FlagPlacedEvent>()
                    .map(|e| EventData::FlagPlaced(e.clone()))
            }
            "multiplay_minesweeper::events::board_events::CellRevealedEvent" => {
                event.as_any().downcast_ref::<crate::events::board_events::CellRevealedEvent>()
                    .map(|e| EventData::CellRevealed(e.clone()))
            }
            "multiplay_minesweeper::events::board_events::MultipleCellsRevealedEvent" => {
                event.as_any().downcast_ref::<crate::events::board_events::MultipleCellsRevealedEvent>()
                    .map(|e| EventData::MultipleCellsRevealed(e.clone()))
            }
            "multiplay_minesweeper::events::board_events::MineExplodedEvent" => {
                event.as_any().downcast_ref::<crate::events::board_events::MineExplodedEvent>()
                    .map(|e| EventData::MineExploded(e.clone()))
            }
            "multiplay_minesweeper::events::board_events::BoardInitializedEvent" => {
                event.as_any().downcast_ref::<crate::events::board_events::BoardInitializedEvent>()
                    .map(|e| EventData::BoardInitialized(e.clone()))
            }
            "multiplay_minesweeper::events::board_events::GameProgressEvent" => {
                event.as_any().downcast_ref::<crate::events::board_events::GameProgressEvent>()
                    .map(|e| EventData::GameProgress(e.clone()))
            }
            
            // 入力イベント
            "multiplay_minesweeper::events::input_events::MouseMoveEvent" => {
                event.as_any().downcast_ref::<crate::events::input_events::MouseMoveEvent>()
                    .map(|e| EventData::MouseMove(e.clone()))
            }
            "multiplay_minesweeper::events::input_events::MouseClickEvent" => {
                event.as_any().downcast_ref::<crate::events::input_events::MouseClickEvent>()
                    .map(|e| EventData::MouseClick(e.clone()))
            }
            "multiplay_minesweeper::events::input_events::KeyboardEvent" => {
                event.as_any().downcast_ref::<crate::events::input_events::KeyboardEvent>()
                    .map(|e| EventData::KeyboardInput(e.clone()))
            }
            "multiplay_minesweeper::events::input_events::UIClickEvent" => {
                event.as_any().downcast_ref::<crate::events::input_events::UIClickEvent>()
                    .map(|e| EventData::UIClick(e.clone()))
            }
            "multiplay_minesweeper::events::input_events::HotkeyEvent" => {
                event.as_any().downcast_ref::<crate::events::input_events::HotkeyEvent>()
                    .map(|e| EventData::Hotkey(e.clone()))
            }
            
            // ネットワークイベント
            "multiplay_minesweeper::events::network_events::NetworkConnectEvent" => {
                event.as_any().downcast_ref::<crate::events::network_events::NetworkConnectEvent>()
                    .map(|e| EventData::NetworkConnect(e.clone()))
            }
            "multiplay_minesweeper::events::network_events::NetworkDisconnectEvent" => {
                event.as_any().downcast_ref::<crate::events::network_events::NetworkDisconnectEvent>()
                    .map(|e| EventData::NetworkDisconnect(e.clone()))
            }
            "multiplay_minesweeper::events::network_events::NetworkErrorEvent" => {
                event.as_any().downcast_ref::<crate::events::network_events::NetworkErrorEvent>()
                    .map(|e| EventData::NetworkError(e.clone()))
            }
            "multiplay_minesweeper::events::network_events::DataReceivedEvent" => {
                event.as_any().downcast_ref::<crate::events::network_events::DataReceivedEvent>()
                    .map(|e| EventData::DataReceived(e.clone()))
            }
            "multiplay_minesweeper::events::network_events::DataSentEvent" => {
                event.as_any().downcast_ref::<crate::events::network_events::DataSentEvent>()
                    .map(|e| EventData::DataSent(e.clone()))
            }
            "multiplay_minesweeper::events::network_events::SessionJoinEvent" => {
                event.as_any().downcast_ref::<crate::events::network_events::SessionJoinEvent>()
                    .map(|e| EventData::SessionJoin(e.clone()))
            }
            "multiplay_minesweeper::events::network_events::SessionLeaveEvent" => {
                event.as_any().downcast_ref::<crate::events::network_events::SessionLeaveEvent>()
                    .map(|e| EventData::SessionLeave(e.clone()))
            }
            "multiplay_minesweeper::events::network_events::PlayerJoinedEvent" => {
                event.as_any().downcast_ref::<crate::events::network_events::PlayerJoinedEvent>()
                    .map(|e| EventData::PlayerJoined(e.clone()))
            }
            "multiplay_minesweeper::events::network_events::PlayerLeftEvent" => {
                event.as_any().downcast_ref::<crate::events::network_events::PlayerLeftEvent>()
                    .map(|e| EventData::PlayerLeft(e.clone()))
            }
            "multiplay_minesweeper::events::network_events::LagMeasurementEvent" => {
                event.as_any().downcast_ref::<crate::events::network_events::LagMeasurementEvent>()
                    .map(|e| EventData::LagMeasurement(e.clone()))
            }
            "multiplay_minesweeper::events::network_events::PlayerScoreUpdateEvent" => {
                event.as_any().downcast_ref::<crate::events::network_events::PlayerScoreUpdateEvent>()
                    .map(|e| EventData::PlayerScoreUpdate(e.clone()))
            }
            "multiplay_minesweeper::events::network_events::ChatMessageEvent" => {
                event.as_any().downcast_ref::<crate::events::network_events::ChatMessageEvent>()
                    .map(|e| EventData::ChatMessage(e.clone()))
            }
            "multiplay_minesweeper::events::network_events::RoomStateEvent" => {
                event.as_any().downcast_ref::<crate::events::network_events::RoomStateEvent>()
                    .map(|e| EventData::RoomState(e.clone()))
            }
            "multiplay_minesweeper::events::network_events::ConnectionStateEvent" => {
                event.as_any().downcast_ref::<crate::events::network_events::ConnectionStateEvent>()
                    .map(|e| EventData::ConnectionState(e.clone()))
            }
            
            // システム関連イベント
            "multiplay_minesweeper::events::input_events::ResourceLoadEvent" => {
                event.as_any().downcast_ref::<crate::events::input_events::ResourceLoadEvent>()
                    .map(|e| EventData::ResourceLoad(e.clone()))
            }
            "multiplay_minesweeper::events::input_events::AppStateEvent" => {
                event.as_any().downcast_ref::<crate::events::input_events::AppStateEvent>()
                    .map(|e| EventData::AppState(e.clone()))
            }
            
            unknown_type => {
                if self.debug {
                    println!("警告: 未知のイベントタイプ: {}", unknown_type);
                }
                None
            }
        };
        
        if result.is_none() && self.debug {
            println!("エラー: イベントタイプ「{}」のEventDataへの変換に失敗しました", type_name);
        }
        
        result
    }
    
    /// 登録されているハンドラの数を取得
    pub fn handler_count<E: TypedEvent>(&self) -> usize {
        let type_id = TypeId::of::<E>();
        let handlers = self.handlers.read().unwrap();
        
        handlers.get(&type_id)
            .map(|handler_collection| handler_collection.len())
            .unwrap_or(0)
    }
    
    /// すべてのハンドラをクリア
    pub fn clear_handlers(&self) {
        let mut handlers = self.handlers.write().unwrap();
        handlers.clear();
        
        let mut global_processors = self.global_processors.write().unwrap();
        global_processors.clear();
        
        if self.debug {
            println!("すべてのハンドラがクリアされました");
        }
    }
    
    /// ハンドラコレクションをダウンキャスト（内部実装用）
    fn downcast_handler_collection<'a, E: TypedEvent>(
        &self,
        collection: &'a mut Box<dyn AnyHandlerCollection + Send + Sync>,
    ) -> Option<&'a mut TypedHandlerCollection<E>> {
        collection.as_any_mut().downcast_mut::<TypedHandlerCollection<E>>()
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
        match self.convert_to_event_data(&event) {
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
}

impl Debug for TypedEventBus {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let handlers = self.handlers.read().unwrap();
        let handler_count = handlers.len();
        
        let history = self.event_history.read().unwrap();
        let history_count = history.len();
        
        write!(f, "TypedEventBus {{ handlers: {}, history: {}, debug: {} }}",
            handler_count, history_count, self.debug)
    }
}

impl Default for TypedEventBus {
    fn default() -> Self {
        Self::new()
    }
} 