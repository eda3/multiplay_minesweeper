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

use crate::events::typed_event::{TypedEvent, HandlerId, generate_id};
use crate::events::typed_handler::{
    TypedEventHandler, TypedHandlerCollection, AnyHandlerCollection
};
use crate::events::EventData;

/// 型付きイベントバス：型安全なイベントの発行と購読を管理するハブ
#[derive(Clone)]
pub struct TypedEventBus {
    /// 型ごとのイベントハンドラマップ
    handlers: Arc<RwLock<HashMap<TypeId, Box<dyn AnyHandlerCollection + Send + Sync>>>>,
    /// イベント履歴（オプション）
    event_history: Arc<RwLock<Vec<EventData>>>,
    /// 履歴の最大サイズ
    max_history_size: usize,
    /// デバッグモード
    debug: bool,
}

impl TypedEventBus {
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
        let type_id = TypeId::of::<E>();
        
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
        
        // ハンドラを取得して呼び出し
        let handlers = self.handlers.read().unwrap();
        if let Some(handler_collection) = handlers.get(&type_id) {
            handler_collection.handle_any(&event as &dyn Any);
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
            
            // その他のイベント型を必要に応じて追加
            // デバッグモードの場合はマッピングされなかった型を出力
            _ => {
                if self.debug {
                    println!("警告: EventDataへの変換が未実装の型: {}", type_name);
                    println!("ヒント: イベント型に to_event_data() メソッドを実装してください");
                }
                None
            }
        }
    }
    
    /// 特定のイベント型に対するハンドラ数を取得
    pub fn handler_count<E: TypedEvent>(&self) -> usize {
        let type_id = TypeId::of::<E>();
        let handlers = self.handlers.read().unwrap();
        
        handlers.get(&type_id)
            .map(|handler_collection| handler_collection.len())
            .unwrap_or(0)
    }
    
    /// すべてのハンドラを削除
    pub fn clear_handlers(&self) {
        let mut handlers = self.handlers.write().unwrap();
        for (_, handler_collection) in handlers.iter_mut() {
            handler_collection.clear();
        }
        
        if self.debug {
            println!("すべてのハンドラがクリアされました");
        }
    }
    
    /// 型付きハンドラコレクションへのダウンキャスト（内部メソッド）
    fn downcast_handler_collection<'a, E: TypedEvent>(
        &self,
        collection: &'a mut Box<dyn AnyHandlerCollection + Send + Sync>,
    ) -> Option<&'a mut TypedHandlerCollection<E>> {
        // AnyHandlerCollectionはdyn Anyをimplementしているため、
        // 直接ダウンキャストできるようにする
        let collection_ref = collection.as_any_mut();
        collection_ref.downcast_mut::<TypedHandlerCollection<E>>()
    }
}

impl Debug for TypedEventBus {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let handlers = self.handlers.read().unwrap();
        let handler_count: usize = handlers.values().map(|v| v.len()).sum();
        
        f.debug_struct("TypedEventBus")
            .field("handler_count", &handler_count)
            .field("type_count", &handlers.len())
            .field("history_size", &self.event_history.read().unwrap().len())
            .field("max_history_size", &self.max_history_size)
            .field("debug_mode", &self.debug)
            .finish()
    }
}

impl Default for TypedEventBus {
    fn default() -> Self {
        Self::new()
    }
} 