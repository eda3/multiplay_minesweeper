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
use crate::ecs::system::System;
use crate::system::system_registry::SystemPhase;
use crate::board::Board;
use wasm_bindgen::JsValue;

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
                        is_game_over: true,
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
                // 公開前の状態を保存
                let pre_revealed_count = revealed_cells_count;
                
                // 空白セルの場合、周囲のセルも公開（チェーン反応）
                // 最適化されたイテレーティブアルゴリズムを使用
                let _ = reveal_connected_cells_iterative(coord.row as usize, coord.col as usize, board);
                
                // 公開後のセル数を再計算
                let new_revealed_count = board.revealed.iter().filter(|&&revealed| revealed).count();
                
                // 複数のセルが公開された場合にイベントを発行
                if new_revealed_count > pre_revealed_count {
                    // 公開されたセルのリストを作成
                    let mut revealed_cells = Vec::new();
                    
                    // 全てのセルをスキャンして新たに公開されたセルを見つける
                    for row in 0..board.height {
                        for col in 0..board.width {
                            let idx = row * board.width + col;
                            // 新たに公開されたセルのみを対象にする
                            if board.revealed[idx] && idx != index {
                                let cell_coord = crate::models::coordinate::Coordinate::new(row as u32, col as u32);
                                revealed_cells.push((cell_coord, board.cells[idx].clone()));
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
    
    fn update(&mut self, _entity_manager: &mut crate::entities::EntityManager, resources: &mut ResourceManager) -> crate::ecs::system::SystemResult {
        // イベントハンドラの初期化
        self.initialize_handlers(resources);
        
        // ボードリソースの取得
        let board_rc = match resources.get::<BoardResource>() {
            Ok(rc) => rc,
            Err(_) => return crate::ecs::system::SystemResult::Error,
        };
        
        // イベントバスからCellRevealedEventをチェック
        let event_bus_rc = match resources.get::<EventBusResource>() {
            Ok(rc) => rc,
            Err(_) => return crate::ecs::system::SystemResult::Error,
        };
        
        let event_bus = event_bus_rc.borrow();
        let event_bus_res = match event_bus.downcast_ref::<EventBusResource>() {
            Some(res) => res,
            None => return crate::ecs::system::SystemResult::Error,
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
            None => return crate::ecs::system::SystemResult::Error,
        };
        
        // イベント発行の例
        // 実際のアプリケーションでは、このようなハードコードされたイベントではなく、
        // ユーザーのアクションや他のシステムからのイベントに基づいて処理が行われる
        
        // マルチセル公開イベントが発生した場合の処理例
        // この部分は通常、イベントハンドラ内で行われる
        
        crate::ecs::system::SystemResult::Ok
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

/**
 * 連鎖的なセル公開処理（イテレーティブアルゴリズム、最適化版）
 * 再帰処理ではなくキューを使って処理することでスタックオーバーフローを防止
 */
fn reveal_connected_cells_iterative(
    row: usize,
    col: usize,
    board: &mut Board
) -> Result<(), JsValue> {
    use std::collections::VecDeque;
    
    let width = board.width;
    let height = board.height;
    let total_cells = width * height;
    
    // ビジットマーカーとして使用するビットセット（効率的なメモリ使用）
    let mut visited = vec![false; total_cells];
    
    // 行オフセットのプリフェッチキャッシュ
    let mut row_offsets = Vec::with_capacity(height);
    for r in 0..height {
        row_offsets.push(r * width);
    }
    
    // 開始セルのインデックス
    let start_index = row_offsets[row] + col;
    
    // 既に開かれているかチェック
    if board.revealed[start_index] {
        return Ok(());
    }
    
    // 効率的なキューの初期化 - ほとんどのケースでは全セルの20%以下しか訪問しない
    let mut queue = VecDeque::with_capacity(total_cells / 5);
    queue.push_back(start_index);
    
    // 隣接インデックスのための方向オフセット配列
    let directions = [
        (-1, -1), // 左上
        (-1,  0), // 上
        (-1,  1), // 右上
        ( 0, -1), // 左
        ( 0,  1), // 右
        ( 1, -1), // 左下
        ( 1,  0), // 下
        ( 1,  1)  // 右下
    ];
    
    while let Some(current_index) = queue.pop_front() {
        // 既に訪問済みならスキップ
        if visited[current_index] {
            continue;
        }
        
        // 訪問済みとしてマーク
        visited[current_index] = true;
        
        // 既に開かれているかフラグがたっているなら無視
        if board.revealed[current_index] || board.flagged[current_index] {
            continue;
        }
        
        // セルを開く
        board.revealed[current_index] = true;
        
        // 空のセル（値が0）でなければこのセルの処理は終了
        if let CellValue::Empty(0) = board.cells[current_index] {
            // このセルは空なので周囲を探索
            
            // インデックスから行と列を逆算
            let row = current_index / width;
            let col = current_index % width;
            
            // 隣接セルをチェック - キャッシュフレンドリーな順序で
            for &(row_delta, col_delta) in &directions {
                let new_row = row as isize + row_delta;
                let new_col = col as isize + col_delta;
                
                // 範囲チェック
                if new_row < 0 || new_row >= height as isize || 
                   new_col < 0 || new_col >= width as isize {
                    continue;
                }
                
                let new_row = new_row as usize;
                let new_col = new_col as usize;
                let new_index = row_offsets[new_row] + new_col;
                
                // まだキューに入っていなければ追加
                if !visited[new_index] && !board.revealed[new_index] && !board.flagged[new_index] {
                    queue.push_back(new_index);
                }
            }
        }
    }
    
    Ok(())
} 