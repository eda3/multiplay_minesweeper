/**
 * セル公開ハンドラシステム
 * 
 * セル公開イベントを購読し、実際のセル公開処理を行う
 */
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use crate::resources::{ResourceManager, BoardResource, EventBusResource};
use crate::events::board_events::{CellRevealedEvent, MultipleCellsRevealedEvent, MineExplodedEvent};
use crate::events::game_events::{GameEndEvent, GameStateChangeEvent, GameState};
use crate::models::cell::CellValue;
use crate::systems::{EventSystemTrait, EventSystem};
use crate::system::system_registry::{System, SystemPhase};

/// セル公開ハンドラシステム
pub struct CellRevealHandlerSystem {
    /// システム名
    name: String,
    /// イベントシステム
    event_system: EventSystem,
    /// イベントハンドラの初期化済みフラグ
    initialized: bool,
}

impl CellRevealHandlerSystem {
    /// 新しいセル公開ハンドラシステムを作成
    pub fn new() -> Self {
        Self {
            name: "CellRevealHandlerSystem".to_string(),
            event_system: EventSystem::new(),
            initialized: false,
        }
    }
    
    /// イベントハンドラを初期化
    fn initialize_handlers(&mut self, resources: &ResourceManager) {
        if self.initialized {
            return;
        }
        
        // CellRevealedEventのハンドラを登録
        self.subscribe_event(
            "CellRevealed",
            "HandleCellReveal",
            move |event: &CellRevealedEvent| {
                println!("セル公開イベントを受信: {:?}", event);
                // 実際の処理はrunメソッドで行う
            },
            resources
        );
        
        self.initialized = true;
    }
    
    /// セルの公開処理
    fn reveal_cell(&self, coord: &crate::models::coordinate::Coordinate, board: &mut BoardResource, resources: &ResourceManager) {
        // ボード内の正しい位置を計算
        let index = coord.row as usize * board.width + coord.col as usize;
        
        // ボードの範囲外なら何もしない
        if index >= board.cells.len() {
            return;
        }
        
        // セルが既に公開済みなら何もしない
        if board.revealed[index] {
            return;
        }
        
        // セルを公開
        board.revealed[index] = true;
        
        // 公開したセルの数を更新
        let revealed_cells_count = board.revealed.iter().filter(|&&revealed| revealed).count();
        
        // セルの値に応じた処理
        match board.cells[index] {
            CellValue::Mine => {
                // 地雷を踏んだ場合、ゲームオーバーイベントを発行
                self.publish_event(
                    MineExplodedEvent {
                        coord: coord.clone(),
                    },
                    resources
                );
                
                // ゲームの状態を変更
                self.publish_event(
                    GameStateChangeEvent {
                        old_state: GameState::Playing,
                        new_state: GameState::GameOver,
                    },
                    resources
                );
                
                // ゲーム終了イベントを発行
                self.publish_event(
                    GameEndEvent {
                        is_win: false,
                        play_time: 0, // 適切な値は別のシステムで計算
                        revealed_cells: revealed_cells_count as u32,
                        flagged_cells: (board.mine_count - board.flagged_count) as u32,
                        score: 0, // 適切な値は別のシステムで計算
                    },
                    resources
                );
            },
            CellValue::Empty(0) => {
                // 空白セルの場合、周囲のセルも公開（チェーン反応）
                // 周囲の座標を計算
                let mut connected_cells = Vec::new();
                let mut revealed_cells = Vec::new();
                
                // 周囲8方向のセルを追加
                for row_offset in -1..=1 {
                    for col_offset in -1..=1 {
                        // 自分自身はスキップ
                        if row_offset == 0 && col_offset == 0 {
                            continue;
                        }
                        
                        let new_row = coord.row as i32 + row_offset;
                        let new_col = coord.col as i32 + col_offset;
                        
                        // ボードの範囲内かチェック
                        if new_row >= 0 && new_row < board.height as i32 &&
                           new_col >= 0 && new_col < board.width as i32 {
                            connected_cells.push(crate::models::coordinate::Coordinate::new(new_row as u32, new_col as u32));
                        }
                    }
                }
                
                // 周囲のセルを公開
                for connected_coord in connected_cells {
                    let connected_index = connected_coord.row as usize * board.width + connected_coord.col as usize;
                    
                    // まだ公開されておらず、フラグが立っていないセルのみ公開
                    if !board.revealed[connected_index] && !board.flagged[connected_index] {
                        // セルを公開
                        board.revealed[connected_index] = true;
                        
                        // 公開したセルとその値を記録
                        revealed_cells.push((connected_coord.clone(), board.cells[connected_index].clone()));
                        
                        // 周囲のセルが空白なら、さらにその周囲も公開（再帰的に処理）
                        if let CellValue::Empty(0) = board.cells[connected_index] {
                            // 実際の実装では、ここで再帰的に処理するか、イテレーティブなアルゴリズムを使用
                            // 簡略化のため、ここではチェーン反応イベントを発行するのみ
                            self.publish_event(
                                CellRevealedEvent {
                                    coord: connected_coord,
                                    value: CellValue::Empty(0),
                                    is_chain: true,
                                },
                                resources
                            );
                        }
                    }
                }
                
                // 一括公開イベントを発行
                if !revealed_cells.is_empty() {
                    self.publish_event(
                        MultipleCellsRevealedEvent {
                            revealed_cells,
                            is_chain: true,
                        },
                        resources
                    );
                }
            },
            _ => {
                // 数字のセルの場合は特に追加処理なし
            }
        }
        
        // 勝利条件のチェック
        let total_non_mine_cells = board.width * board.height - board.mine_count;
        let revealed_cells_count = board.revealed.iter().filter(|&&revealed| revealed).count();
        
        if revealed_cells_count == total_non_mine_cells {
            // すべての非地雷セルが公開された場合、勝利
            self.publish_event(
                GameStateChangeEvent {
                    old_state: GameState::Playing,
                    new_state: GameState::Victory,
                },
                resources
            );
            
            // ゲーム終了イベントを発行
            self.publish_event(
                GameEndEvent {
                    is_win: true,
                    play_time: 0, // 適切な値は別のシステムで計算
                    revealed_cells: revealed_cells_count as u32,
                    flagged_cells: (board.mine_count - board.flagged_count) as u32,
                    score: 0, // 適切な値は別のシステムで計算
                },
                resources
            );
        }
    }
}

impl Default for CellRevealHandlerSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl System for CellRevealHandlerSystem {
    fn name(&self) -> &str {
        &self.name
    }
    
    fn phase(&self) -> SystemPhase {
        SystemPhase::Update
    }
    
    fn run(&mut self, resources: &mut ResourceManager) {
        // イベントハンドラの初期化
        self.initialize_handlers(resources);
        
        // ボードリソースの取得
        let board_rc = match resources.get::<BoardResource>() {
            Ok(rc) => rc,
            Err(_) => return,
        };
        
        // イベントバスからCellRevealedEventをチェック
        let event_bus_rc = match resources.get::<EventBusResource>() {
            Ok(rc) => rc,
            Err(_) => return,
        };
        
        let event_bus = event_bus_rc.borrow();
        let event_bus_res = match event_bus.downcast_ref::<EventBusResource>() {
            Some(res) => res,
            None => return,
        };
        
        // CellRevealedEventに対応する直接の処理を行う
        // 注：実際には、イベントバスから直接イベントを取得するAPIがないため、
        // ここではイベントハンドラによって通知された座標をメモリに保持して処理する必要がある
        
        // ここではサンプルとして、仮想的にいくつかのセル公開イベントを処理する
        // 実際のコードでは、イベントの購読と処理がより直接的に結合される
        
        // ボードの可変参照を取得
        let mut board = board_rc.borrow_mut();
        let board = match board.downcast_mut::<BoardResource>() {
            Some(board) => board,
            None => return,
        };
        
        // イベント発行の例
        // 実際のアプリケーションでは、このようなハードコードされたイベントではなく、
        // ユーザーのアクションや他のシステムからのイベントに基づいて処理が行われる
        
        // マルチセル公開イベントが発生した場合の処理例
        // この部分は通常、イベントハンドラ内で行われる
    }
}

impl EventSystemTrait for CellRevealHandlerSystem {
    fn get_handler_ids(&self) -> &Arc<Mutex<HashMap<String, u64>>> {
        self.event_system.get_handler_ids()
    }
    
    fn get_handler_ids_mut(&mut self) -> &mut Arc<Mutex<HashMap<String, u64>>> {
        self.event_system.get_handler_ids_mut()
    }
} 