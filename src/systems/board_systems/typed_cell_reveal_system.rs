/**
 * 型安全なセル公開システム
 * 
 * TypedEventSystemTraitを使用した型安全なセル公開処理を行うシステム
 */
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::collections::VecDeque;

use crate::resources::{ResourceManager, BoardResource, GameStateResource};
use crate::events::board_events::{
    CellRevealedEvent, MultipleCellsRevealedEvent, MineExplodedEvent,
    GameProgressEvent, BulkCellStateChangeEvent
};
use crate::events::game_events::{GameEndEvent, GameStateChangeEvent, GameState};
use crate::events::input_events::MouseClickEvent;
use crate::events::typed_event::TypedEvent;
use crate::events::TypedEventBus;
use crate::events::EventData;
use crate::events::typed_event_bus::EventPriority;
use crate::models::cell::CellValue;
use crate::models::coordinate::Coordinate;
use crate::systems::typed_event_system_trait::{TypedEventSystemTrait, TypedEventSystem};
use crate::ecs::system::System;
use crate::ecs::system::SystemResult;
use crate::entities::EntityManager;
use crate::board::Board;

/// イベント処理リクエストの種類
#[derive(Debug, Clone)]
enum EventRequest {
    /// マウスクリックイベント
    MouseClick(MouseClickEvent),
    /// セル公開イベント
    CellRevealed(CellRevealedEvent),
    /// 不明なイベント
    Unknown,
}

/// イベント処理キュー
#[derive(Debug, Default)]
struct EventQueue {
    /// 処理待ちイベントのキュー
    queue: VecDeque<EventRequest>,
    /// 最後に処理したイベントのタイムスタンプ
    last_processed_timestamp: u64,
}

impl EventQueue {
    /// 新しいイベントキューを作成
    fn new() -> Self {
        Self {
            queue: VecDeque::new(),
            last_processed_timestamp: 0,
        }
    }
    
    /// イベントをキューに追加
    fn enqueue(&mut self, request: EventRequest) {
        self.queue.push_back(request);
    }
    
    /// イベントをキューから取得
    fn dequeue(&mut self) -> Option<EventRequest> {
        self.queue.pop_front()
    }
    
    /// 最後に処理したイベントのタイムスタンプを更新
    fn update_timestamp(&mut self, timestamp: u64) {
        self.last_processed_timestamp = timestamp;
    }
    
    /// 最後に処理したイベントのタイムスタンプを取得
    fn get_last_timestamp(&self) -> u64 {
        self.last_processed_timestamp
    }
}

/// 型安全なセル公開システム
pub struct TypedCellRevealSystem {
    /// システム名
    name: String,
    /// 型安全なイベントシステム
    event_system: TypedEventSystem,
    /// イベントハンドラの初期化済みフラグ
    initialized: bool,
    /// システムが有効かどうか
    enabled: bool,
    /// イベント処理キュー
    event_queue: Arc<Mutex<EventQueue>>,
}

impl TypedCellRevealSystem {
    /// 新しいセル公開システムを作成
    pub fn new() -> Self {
        Self {
            name: "TypedCellRevealSystem".to_string(),
            event_system: TypedEventSystem::new(),
            initialized: false,
            enabled: true,
            event_queue: Arc::new(Mutex::new(EventQueue::new())),
        }
    }
    
    /// イベントハンドラの初期化
    fn initialize_handlers(&mut self, resources: &ResourceManager) {
        if self.initialized {
            return;
        }
        
        // イベント処理キューへの参照を取得
        let event_queue = self.event_queue.clone();
        
        // マウスクリックイベントを購読：キューにイベントを追加するだけのスレッドセーフな実装
        self.subscribe_typed_event::<MouseClickEvent, _>(
            "MouseClick",
            "CellRevealHandler",
            move |event| {
                if let Ok(mut queue) = event_queue.lock() {
                    queue.enqueue(EventRequest::MouseClick(event.clone()));
                }
            },
            resources
        );
        
        // イベント処理キューへの参照を取得
        let event_queue = self.event_queue.clone();
        
        // セル公開イベントを購読：キューにイベントを追加するだけのスレッドセーフな実装
        self.subscribe_typed_event::<CellRevealedEvent, _>(
            "CellRevealed",
            "RevealHandler",
            move |event| {
                if let Ok(mut queue) = event_queue.lock() {
                    queue.enqueue(EventRequest::CellRevealed(event.clone()));
                }
            },
            resources
        );
        
        self.initialized = true;
    }
    
    /// マウスクリックイベントの処理
    fn handle_mouse_click(&self, event: &MouseClickEvent, resources: &ResourceManager) {
        // 左クリックのみ処理
        if event.is_right_click {  // 右クリックの場合はフラグ処理（処理しない）
            return;
        }
        
        // ボードリソースの取得
        let board_rc = match resources.get::<BoardResource>() {
            Ok(rc) => rc,
            Err(_) => return,
        };
        
        let mut board = board_rc.borrow_mut();
        if let Some(board) = board.downcast_mut::<BoardResource>() {
            // クリック座標からセル座標を計算
            let cell_size = board.cell_size as f64;
            let col = (event.x as f64 / cell_size) as u32;
            let row = (event.y as f64 / cell_size) as u32;
            
            // 範囲外チェック
            if row >= board.height as u32 || col >= board.width as u32 {
                return;
            }
            
            // 最初のクリック処理
            let is_first_click = board.first_click;
            if is_first_click {
                board.initialize();
                board.first_click = false;
            }
            
            // セルを公開
            let coord = Coordinate::new(row, col);
            self.reveal_cell(&coord, board, resources);
        }
    }
    
    /// セル公開イベントの処理
    fn handle_cell_revealed(&self, event: &CellRevealedEvent, resources: &ResourceManager) {
        // レイトゲーム実装
        // 連鎖反応時の特別な処理をここに実装
        self.handle_event(event, resources);
    }
    
    /// イベント処理の共通メソッド
    fn handle_event(&self, event: &CellRevealedEvent, resources: &ResourceManager) {
        // ボードリソースの取得
        if let Ok(board_rc) = resources.get::<BoardResource>() {
            let mut board = board_rc.borrow_mut();
            if let Some(board) = board.downcast_mut::<BoardResource>() {
                // 座標の取得
                let row = event.coord.row as usize;
                let col = event.coord.col as usize;
                
                // 境界チェック
                if row >= board.height || col >= board.width {
                    return;
                }
                
                // インデックスの計算
                let index = row * board.width + col;
                
                // 既に公開済みならスキップ
                if board.revealed[index] {
                    return;
                }
                
                // セルを公開
                board.revealed[index] = true;
                
                // セルの値に応じた処理
                match board.cells[index] {
                    CellValue::Mine => {
                        // 地雷だった場合、ゲームオーバー
                        board.game_over = true;
                        
                        // 地雷爆発イベントを発行（別メソッドで実装）
                    },
                    CellValue::Empty(0) => {
                        // 空セルの場合、周囲のセルも公開
                        self.reveal_connected_cells_optimized(row, col, board, resources);
                    },
                    _ => {
                        // 通常の数字セルの場合は特に何もしない
                    }
                }
                
                // 勝利条件チェック
                self.check_win_condition(board, resources);
                
                // ゲーム進行状況の更新
                self.update_game_progress(board, resources);
            }
        }
    }
    
    /// セルを公開する処理
    fn reveal_cell(&self, coord: &Coordinate, board: &mut BoardResource, resources: &ResourceManager) {
        // ボード内の正しい位置を計算
        let index = coord.row as usize * board.width + coord.col as usize;
        
        // ボードの範囲外なら何もしない
        if index >= board.cells.len() {
            return;
        }
        
        // セルが既に公開済みまたはフラグが立っていれば何もしない
        if board.revealed[index] || board.flagged[index] {
            return;
        }
        
        // セルを公開
        board.revealed[index] = true;
        
        // セルの値に応じた処理
        match board.cells[index] {
            CellValue::Mine => {
                // 地雷の場合、ゲームオーバー
                board.game_over = true;
                
                // 地雷爆発イベントを発行
                self.publish_typed_event(
                    MineExplodedEvent {
                        coord: coord.clone(),
                    },
                    resources
                );
                
                // ゲーム状態を変更
                self.publish_typed_event(
                    GameStateChangeEvent {
                        old_state: GameState::Playing,
                        new_state: GameState::GameOver,
                    },
                    resources
                );
                
                // ゲーム終了イベントを発行
                self.publish_typed_event(
                    GameEndEvent {
                        is_win: false,
                        play_time: 0, // 適切な値は別のシステムで計算
                        revealed_cells: board.revealed.iter().filter(|&&r| r).count() as u32,
                        flagged_cells: board.flagged.iter().filter(|&&f| f).count() as u32,
                        score: 0, // 適切な値は別のシステムで計算
                    },
                    resources
                );
                
                // 全ての地雷を表示
                board.reveal_all_mines();
            },
            CellValue::Empty(0) => {
                // 空白セルの場合、チェーン反応を開始
                // イベントを発行
                self.publish_typed_event(
                    CellRevealedEvent {
                        coord: coord.clone(),
                        value: CellValue::Empty(0),
                        is_chain: true,
                    },
                    resources
                );
                
                // 隣接セルの公開はCellRevealedEventのハンドラで行う
            },
            value => {
                // 数字セルの場合、単体でイベント発行
                self.publish_typed_event(
                    CellRevealedEvent {
                        coord: coord.clone(),
                        value: value.clone(),
                        is_chain: false,
                    },
                    resources
                );
                
                // 進行状況更新
                self.update_game_progress(board, resources);
            }
        }
        
        // 勝利条件のチェック
        self.check_win_condition(board, resources);
    }
    
    /// 連鎖的なセル公開処理（最適化版）
    fn reveal_connected_cells_optimized(
        &self,
        row: usize,
        col: usize,
        board: &mut BoardResource,
        resources: &ResourceManager
    ) {
        let width = board.width;
        let height = board.height;
        let total_cells = width * height;
        
        // 訪問済みセルの追跡
        let mut visited = vec![false; total_cells];
        
        // 行オフセットのキャッシュ（メモリアクセスパターン最適化）
        let row_offsets: Vec<usize> = (0..height).map(|r| r * width).collect();
        
        // 開始セルのインデックス
        let start_index = row_offsets[row] + col;
        
        // 既に開かれているかチェック
        if board.revealed[start_index] {
            // 2回目の処理を防止
            return;
        }
        
        // キューの初期化 - サイズを予め確保して再アロケーションを減らす
        let mut queue = VecDeque::with_capacity(total_cells / 5);
        queue.push_back((row, col));
        visited[start_index] = true;
        
        // 公開されたセル情報を収集
        let mut revealed_cells = Vec::new();
        
        // 隣接セル用の方向オフセット
        let directions = [
            (-1, -1), (-1, 0), (-1, 1),
            (0, -1),           (0, 1),
            (1, -1),  (1, 0),  (1, 1)
        ];
        
        while let Some((r, c)) = queue.pop_front() {
            let idx = row_offsets[r] + c;
            
            // セルを公開
            if !board.revealed[idx] {
                board.revealed[idx] = true;
                
                // 公開されたセルを記録
                let cell_coord = Coordinate::new(r as u32, c as u32);
                revealed_cells.push((cell_coord, board.cells[idx].clone()));
                
                // 空白セルなら隣接セルも処理
                if let CellValue::Empty(0) = board.cells[idx] {
                    for &(dr, dc) in &directions {
                        let nr = r as isize + dr;
                        let nc = c as isize + dc;
                        
                        // 範囲チェック
                        if nr >= 0 && nr < height as isize && nc >= 0 && nc < width as isize {
                            let nr = nr as usize;
                            let nc = nc as usize;
                            let next_idx = row_offsets[nr] + nc;
                            
                            // 未訪問かつフラグなしのセルのみ処理
                            if !visited[next_idx] && !board.flagged[next_idx] {
                                visited[next_idx] = true;
                                queue.push_back((nr, nc));
                            }
                        }
                    }
                }
            }
        }
        
        // 複数のセルが公開された場合にイベントを発行
        if !revealed_cells.is_empty() {
            self.publish_typed_event(
                MultipleCellsRevealedEvent {
                    revealed_cells,
                    is_chain: true,
                },
                resources
            );
            
            // 進行状況更新
            self.update_game_progress(board, resources);
        }
    }
    
    /// 勝利条件のチェック
    fn check_win_condition(&self, board: &mut BoardResource, resources: &ResourceManager) {
        let total_non_mine_cells = board.width * board.height - board.mine_count;
        let revealed_cells_count = board.revealed.iter().filter(|&&revealed| revealed).count();
        
        if revealed_cells_count == total_non_mine_cells {
            // すべての非地雷セルが公開された場合、勝利
            board.win = true;
            
            self.publish_typed_event(
                GameStateChangeEvent {
                    old_state: GameState::Playing,
                    new_state: GameState::Victory,
                },
                resources
            );
            
            // ゲーム終了イベントを発行
            self.publish_typed_event(
                GameEndEvent {
                    is_win: true,
                    play_time: 0, // 適切な値は別のシステムで計算
                    revealed_cells: revealed_cells_count as u32,
                    flagged_cells: (board.mine_count - board.flagged_count) as u32,
                    score: 0, // 適切な値は別のシステムで計算
                },
                resources
            );
            
            // 全ての地雷にフラグを立てる
            board.reveal_all_mines();
        }
    }
    
    /// ゲーム進行状況の更新
    fn update_game_progress(&self, board: &BoardResource, resources: &ResourceManager) {
        let total_non_mine_cells = board.width * board.height - board.mine_count;
        let revealed_cells_count = board.revealed.iter().filter(|&&revealed| revealed).count();
        let remaining = total_non_mine_cells - revealed_cells_count;
        
        // 進行率の計算
        let completion_rate = revealed_cells_count as f32 / total_non_mine_cells as f32;
        
        self.publish_typed_event(
            GameProgressEvent {
                revealed_count: revealed_cells_count as u32,
                remaining_non_mine_cells: remaining as u32,
                completion_rate,
            },
            resources
        );
    }
}

impl Clone for TypedCellRevealSystem {
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

impl Default for TypedCellRevealSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl System for TypedCellRevealSystem {
    fn update(&mut self, entity_manager: &mut EntityManager, resources: &mut ResourceManager) -> SystemResult {
        // イベントハンドラを初期化（まだ初期化されていない場合）
        if !self.initialized {
            self.initialize_handlers(resources);
        }
        
        // イベントバスから最新のイベントを取得し、キューに対して既に処理済みのイベントを更新
        if let Ok(event_bus_rc) = resources.get::<TypedEventBus>() {
            let event_bus = event_bus_rc.borrow();
            if let Some(event_bus) = event_bus.downcast_ref::<TypedEventBus>() {
                // キューからタイムスタンプを取得
                let last_timestamp = {
                    if let Ok(queue) = self.event_queue.lock() {
                        queue.get_last_timestamp()
                    } else {
                        0 // ロックが取得できない場合は0（全イベント処理）
                    }
                };
                
                // 最新のイベントを効率的に取得（最後のタイムスタンプ以降のみ）
                let reveal_events = event_bus.get_events_by_type_since::<CellRevealedEvent>(last_timestamp);
                if !reveal_events.is_empty() {
                    // バッチ処理のためにイベントバスのバッチモードを開始
                    event_bus.start_batch_mode();
                    
                    // イベントをキューに追加
                    let mut events_to_process = Vec::new();
                    for event in reveal_events {
                        if event.is_chain {
                            // 連鎖反応イベントは低優先度で処理
                            events_to_process.push((event, EventPriority::Low));
                        } else {
                            // 通常のクリックイベントは高優先度で処理
                            events_to_process.push((event, EventPriority::High));
                        }
                    }
                    
                    // イベントを処理（優先度順）
                    for (event, priority) in events_to_process {
                        event_bus.publish_with_priority(event, priority);
                    }
                    
                    // バッチモードを終了して一括処理
                    event_bus.end_batch_mode();
                }
            }
        }
        
        // キューからイベントを取り出して処理
        let mut event_count = 0;
        const MAX_EVENTS_PER_UPDATE: usize = 5; // 1フレームで処理する最大イベント数
        
        while event_count < MAX_EVENTS_PER_UPDATE {
            // キューからイベントを取得
            let event_request = {
                if let Ok(mut queue) = self.event_queue.lock() {
                    queue.dequeue()
                } else {
                    None
                }
            };
            
            // イベントが取得できなければループを抜ける
            let event_request = match event_request {
                Some(req) => req,
                None => break,
            };
            
            // イベントの種類に応じた処理
            match event_request {
                EventRequest::MouseClick(event) => {
                    self.handle_mouse_click(&event, resources);
                },
                EventRequest::CellRevealed(event) => {
                    self.handle_event(&event, resources);
                },
                EventRequest::Unknown => {
                    // 不明なイベントは無視
                }
            }
            
            event_count += 1;
        }
        
        // イベントを処理した結果、キューに残りがある場合は再実行が必要
        let needs_rerun = {
            if let Ok(queue) = self.event_queue.lock() {
                !queue.queue.is_empty()
            } else {
                false
            }
        };
        
        if needs_rerun {
            SystemResult::NeedRerun
        } else {
            SystemResult::Ok
        }
    }
    
    fn name(&self) -> &str {
        "TypedCellRevealSystem"
    }
    
    fn enabled(&self) -> bool {
        self.enabled
    }
    
    fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
}

impl TypedEventSystemTrait for TypedCellRevealSystem {
    fn get_handler_ids(&self) -> &Arc<Mutex<HashMap<String, crate::events::typed_event::HandlerId>>> {
        self.event_system.get_handler_ids()
    }
    
    fn get_handler_ids_mut(&mut self) -> &mut Arc<Mutex<HashMap<String, crate::events::typed_event::HandlerId>>> {
        self.event_system.get_handler_ids_mut()
    }
} 