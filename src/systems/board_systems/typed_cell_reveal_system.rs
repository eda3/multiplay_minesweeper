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
    GameProgressEvent
};
use crate::events::game_events::{GameEndEvent, GameStateChangeEvent, GameState};
use crate::models::cell::CellValue;
use crate::models::coordinate::Coordinate;
use crate::systems::{TypedEventSystemTrait, TypedEventSystem};
use crate::ecs::system::System;
use crate::system::system_registry::SystemPhase;
use crate::ecs::system::SystemResult;
use crate::events::input_events::MouseClickEvent;

/// 型安全なセル公開システム
pub struct TypedCellRevealSystem {
    /// システム名
    name: String,
    /// 型安全なイベントシステム
    event_system: TypedEventSystem,
    /// イベントハンドラの初期化済みフラグ
    initialized: bool,
}

impl TypedCellRevealSystem {
    /// 新しいセル公開システムを作成
    pub fn new() -> Self {
        Self {
            name: "TypedCellRevealSystem".to_string(),
            event_system: TypedEventSystem::new(),
            initialized: false,
        }
    }
    
    /// イベントハンドラの初期化
    fn initialize_handlers(&mut self, resources: &ResourceManager) {
        if self.initialized {
            return;
        }
        
        // スレッド安全にするために参照を持つ代わりに、システム自体をクローンせず、
        // WASM環境に適したコールバックにする
        #[cfg(target_arch = "wasm32")]
        {
            // WASM向け実装（スレッド安全制約を回避）
            let system_name = self.name.clone();
            self.subscribe_typed_event::<MouseClickEvent, _>(
                "MouseClick",
                "CellRevealHandler",
                move |event| {
                    // ここではコールバック内でリソースを使わない
                    // 実際にはフラグを設定して別の方法で処理する必要がある
                    web_sys::console::log_1(&format!("[{}] マウスクリック: ({}, {})",
                        system_name, event.x, event.y).into());
                },
                resources
            );
            
            let system_name = self.name.clone();
            self.subscribe_typed_event::<CellRevealedEvent, _>(
                "CellRevealed",
                "RevealHandler",
                move |event| {
                    // ここではコールバック内でリソースを使わない
                    web_sys::console::log_1(&format!("[{}] セル公開: ({}, {})",
                        system_name, event.coord.row, event.coord.col).into());
                },
                resources
            );
        }
        
        #[cfg(not(target_arch = "wasm32"))]
        {
            // 非WASM環境向け（通常の実装）
            let system_ref = self.clone();
            self.subscribe_typed_event::<MouseClickEvent, _>(
                "MouseClick",
                "CellRevealHandler",
                move |event| {
                    system_ref.handle_mouse_click(event, resources);
                },
                resources
            );
            
            let system_ref = self.clone();
            self.subscribe_typed_event::<CellRevealedEvent, _>(
                "CellRevealed",
                "RevealHandler",
                move |event| {
                    system_ref.handle_cell_revealed(event, resources);
                },
                resources
            );
        }
        
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
    fn handle_cell_revealed(&self, event: &CellRevealedEvent, resources: &mut ResourceManager) {
        // ボードリソースの取得
        let board_rc = match resources.get_mut::<BoardResource>() {
            Ok(rc) => rc,
            Err(_) => return,
        };
        
        let mut board = board_rc.borrow_mut();
        if let Some(board) = board.downcast_mut::<BoardResource>() {
            // 既にゲームオーバーなら何もしない
            if board.game_over || board.win {
                return;
            }
            
            // ChainReactionでない場合、単一セルの処理のみ
            if !event.is_chain {
                // 既に処理済みなのでスキップ
                return;
            }
            
            // チェーン反応の場合、周囲のセルも公開
            self.reveal_connected_cells_optimized(
                event.coord.row as usize,
                event.coord.col as usize,
                board,
                resources
            );
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
        }
    }
}

impl Default for TypedCellRevealSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl System for TypedCellRevealSystem {
    fn name(&self) -> &str {
        &self.name
    }
    
    fn update(&mut self, _entity_manager: &mut crate::entities::EntityManager, resources: &mut ResourceManager) -> SystemResult {
        // イベントハンドラの初期化
        self.initialize_handlers(resources);
        
        // 実際の処理はイベントハンドラで行われるので、ここでは何もしない
        // ただし、WASMビルドでは特別な対応が必要かもしれない
        #[cfg(target_arch = "wasm32")]
        {
            // WASM環境ではイベントハンドラがリソースを直接使えないため、
            // ここでイベントキューをチェックして処理する
            // イベントキューのチェックロジックをここに実装
        }
        
        SystemResult::Ok
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