/**
 * 型付きイベントバス
 * 
 * 型安全なイベントの発行と購読を管理する中心的なコンポーネント。
 * コンパイル時の型チェックを最大限活用し、実行時エラーを防ぐ。
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
        Self {
            handlers: Arc::new(RwLock::new(HashMap::new())),
            event_history: Arc::new(RwLock::new(Vec::new())),
            typed_event_history: Arc::new(RwLock::new(HashMap::new())),
            event_queue: Arc::new(RwLock::new(BinaryHeap::new())),
            batch_mode: Arc::new(RwLock::new(false)),
            max_history_size: 100,
            debug: false,
            global_processors: Arc::new(RwLock::new(Vec::new())),
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
        // まず、イベント自身のto_event_dataメソッドを試す
        // これが最も型安全なアプローチ
        let event_data = event.to_event_data();
        if event_data.is_some() {
            return event_data;
        }
        
        // フォールバック: 型名によるマッチングを行う
        // これはリフレクションを使った実装で、理想的にはto_event_dataの実装が望ましい
        let type_name = std::any::type_name::<E>();
        
        match type_name {
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
                    .map(|e| EventData::Keyboard(e.clone()))
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
            // ... 他のイベント ...
            
            _ => None,
        }
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