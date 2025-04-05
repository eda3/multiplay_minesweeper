/**
 * セル公開システム
 * 
 * セルをクリックして内容を公開するシステム
 */
use std::collections::{HashMap, HashSet, VecDeque};
use std::rc::Rc;
use std::cell::RefCell;
use std::any::Any;
use wasm_bindgen::JsValue;
use wasm_bindgen::prelude::*;

use crate::entities::EntityManager;
use crate::entities::EntityId;
use crate::systems::optimized::system_scheduler::DeltaTime;
use crate::resources::{
    BoardConfigResource,
    BoardStateResource,
    PlayerStateResource,
    MouseState,
    TimeResource,
    Resource
};
use crate::components::board_components::{CellStateComponent, CellState, CellContentComponent, GridPositionComponent};
use crate::models::CellValue;
use crate::ecs::system::{System, SystemResult};
use crate::resources::ResourceManager;
use crate::resources::{
    BoardResource,
    GameStateResource,
};
use crate::resources::BoardConfig;
use crate::board::Board;
use crate::utils::get_adjacent_offsets;

/// セル公開システム - セルを公開する処理を担当
pub fn cell_reveal_system(
    resources: &mut ResourceManager,
    _delta_time: DeltaTime
) -> Result<(), JsValue> {
    // セル公開ロジックをここに実装
    Ok(())
}

/// 指定されたセルを公開する
/// 返値: セルが公開されたかどうか
pub fn reveal_cell(
    row: usize,
    col: usize,
    entity_manager: &mut EntityManager,
    board: &mut Board,
    is_first_click: bool
) -> Result<bool, JsValue> {
    // ゲームオーバーや勝利状態では何もしない
    if board.game_over || board.win {
        return Ok(false);
    }

    // インデックスを計算
    let index = row * board.width + col;
    
    // 既に開いているセルや旗が立てられているセルは無視
    if board.revealed[index] || board.flagged[index] {
        return Ok(false);
    }

    // 最初のクリックの場合、地雷を再配置
    if is_first_click {
        // 最初のクリックでは地雷に当たらないようにする
        board.initialize();
        board.first_click = false;
    }

    // セルを開く
    board.revealed[index] = true;

    // 地雷をクリックした場合
    if let CellValue::Mine = board.cells[index] {
        // ゲームオーバー
        board.game_over = true;
        board.reveal_all_mines();
        return Ok(true); // 爆発を示すtrueを返す
    }

    // 残りの安全なセル数を減らす
    board.remaining_safe_cells -= 1;

    // 周囲の地雷がない場合は周囲のセルも開く
    if let CellValue::Empty(0) = board.cells[index] {
        // 再帰処理を非再帰（イテレーティブ）に変更
        reveal_connected_cells_iterative(row, col, entity_manager, board)?;
    }

    // 勝利条件をチェック
    if check_win_condition(board) {
        board.win = true;
        board.reveal_all_mines();
    }

    Ok(false) // 爆発しなかったのでfalseを返す
}

/// 連鎖的なセル公開を非再帰的に処理する関数
fn reveal_connected_cells_iterative(
    start_row: usize,
    start_col: usize,
    entity_manager: &mut EntityManager,
    board: &mut Board
) -> Result<(), JsValue> {
    // キューを使って処理するセルを管理（事前に容量確保）
    let estimated_capacity = (board.width * board.height) / 4; // 盤面の25%の容量を事前確保
    let mut queue = VecDeque::with_capacity(estimated_capacity);
    
    // 処理済みセルを記録するセット（こちらも事前に容量確保）
    let mut visited = HashSet::with_capacity(estimated_capacity);
    
    // 開始セルをキューに追加
    queue.push_back((start_row, start_col));
    
    // 隣接セル座標用のバッファを事前に確保（再利用）
    let mut adjacent_buffer = Vec::with_capacity(8);
    
    // インデックス計算をキャッシュするためのルックアップテーブル
    // 各行の先頭インデックスを事前計算
    let mut row_offset_cache = Vec::with_capacity(board.height);
    for row in 0..board.height {
        row_offset_cache.push(row * board.width);
    }
    
    // よく使う値をローカル変数にキャッシュ
    let width = board.width;
    let height = board.height;
    
    // バッチ処理のために開くセルを蓄積
    let mut cells_to_reveal = Vec::with_capacity(estimated_capacity);
    
    while let Some((row, col)) = queue.pop_front() {
        // セルのインデックスを事前計算したキャッシュを使って計算（乗算を回避）
        let index = row_offset_cache[row] + col;
        
        // 既に処理済みならスキップ - キャッシュヒット率を上げるため早めにチェック
        if !visited.insert(index) {
            continue;
        }
        
        // 周囲のセルを取得して処理
        adjacent_buffer.clear(); // バッファを再利用
        get_adjacent_cells_cached(row, col, width, height, &row_offset_cache, &mut adjacent_buffer);
        
        // 隣接セルの一括処理の準備
        let mut local_cells_to_enqueue = Vec::with_capacity(8);
        
        // 隣接セルの一括処理
        for &(adj_row, adj_col, adj_index) in &adjacent_buffer {
            // 既に処理済みならスキップ（早期チェック）
            if visited.contains(&adj_index) {
                continue;
            }
            
            // 既に開いているセルや旗が立てられているセルは無視
            if board.revealed[adj_index] || board.flagged[adj_index] {
                continue;
            }
            
            // セルを蓄積（バッチ処理用）
            cells_to_reveal.push(adj_index);
            
            // 周囲に地雷がない空のセルなら、後でキューに追加するリストに加える
            if let CellValue::Empty(0) = board.cells[adj_index] {
                local_cells_to_enqueue.push((adj_row, adj_col));
            }
        }
        
        // 一括で処理する
        for &idx in &cells_to_reveal {
            board.revealed[idx] = true;
            board.remaining_safe_cells -= 1;
        }
        cells_to_reveal.clear();
        
        // バッチでキューに追加
        queue.extend(local_cells_to_enqueue);
    }
    
    Ok(())
}

/// 指定したセルの周囲8方向のセル座標を最適化して取得
/// 事前に確保された配列に結果を格納
fn get_adjacent_cells_optimized(
    row: usize, 
    col: usize, 
    width: usize, 
    height: usize,
    result: &mut Vec<(usize, usize)>
) {
    // 範囲チェックを最小限にするため、範囲内にあることが明らかな場合は直接追加
    let row_top = row > 0;
    let row_bottom = row < height - 1;
    let col_left = col > 0;
    let col_right = col < width - 1;
    
    // 上段
    if row_top {
        if col_left {
            result.push((row - 1, col - 1));
        }
        result.push((row - 1, col));
        if col_right {
            result.push((row - 1, col + 1));
        }
    }
    
    // 中段
    if col_left {
        result.push((row, col - 1));
    }
    if col_right {
        result.push((row, col + 1));
    }
    
    // 下段
    if row_bottom {
        if col_left {
            result.push((row + 1, col - 1));
        }
        result.push((row + 1, col));
        if col_right {
            result.push((row + 1, col + 1));
        }
    }
}

/// インデックスキャッシュを使用した隣接セル探索
pub fn get_adjacent_cells_cached(
    row: usize, 
    col: usize, 
    width: usize, 
    height: usize,
    row_offset_cache: &[usize],
    result: &mut Vec<(usize, usize, usize)> // (行, 列, インデックス)
) {
    // 範囲チェックを最小限にするため、範囲内にあることが明らかな場合は直接追加
    let row_top = row > 0;
    let row_bottom = row < height - 1;
    let col_left = col > 0;
    let col_right = col < width - 1;
    
    // 上段
    if row_top {
        let top_row = row - 1;
        let top_row_offset = row_offset_cache[top_row];
        
        if col_left {
            let idx = top_row_offset + (col - 1);
            result.push((top_row, col - 1, idx));
        }
        
        let idx = top_row_offset + col;
        result.push((top_row, col, idx));
        
        if col_right {
            let idx = top_row_offset + (col + 1);
            result.push((top_row, col + 1, idx));
        }
    }
    
    // 中段
    let current_row_offset = row_offset_cache[row];
    
    if col_left {
        let idx = current_row_offset + (col - 1);
        result.push((row, col - 1, idx));
    }
    
    if col_right {
        let idx = current_row_offset + (col + 1);
        result.push((row, col + 1, idx));
    }
    
    // 下段
    if row_bottom {
        let bottom_row = row + 1;
        let bottom_row_offset = row_offset_cache[bottom_row];
        
        if col_left {
            let idx = bottom_row_offset + (col - 1);
            result.push((bottom_row, col - 1, idx));
        }
        
        let idx = bottom_row_offset + col;
        result.push((bottom_row, col, idx));
        
        if col_right {
            let idx = bottom_row_offset + (col + 1);
            result.push((bottom_row, col + 1, idx));
        }
    }
}

pub fn check_win_condition(board: &mut Board) -> bool {
    // 全ての安全なセルが開かれているかチェック
    board.check_win_condition()
}

/// 指定したセルの周囲8方向のセル座標を取得
fn get_adjacent_cells(row: usize, col: usize, width: usize, height: usize) -> Vec<(usize, usize)> {
    let mut adjacent_cells = Vec::new();
    
    // 周囲8方向の座標オフセットを取得
    for (dr, dc) in get_adjacent_offsets().iter() {
        let new_row = row as isize + dr;
        let new_col = col as isize + dc;
        
        // 範囲内の座標のみを追加
        if new_row >= 0 && new_row < height as isize &&
           new_col >= 0 && new_col < width as isize {
            adjacent_cells.push((new_row as usize, new_col as usize));
        }
    }
    
    adjacent_cells
}

/**
 * セル公開システム
 * 
 * セルをクリックして内容を公開するシステム
 */
pub struct CellRevealSystem {
    // 必要なステート
}

impl CellRevealSystem {
    pub fn new() -> Self {
        Self {
            // 初期化
        }
    }
}

impl System for CellRevealSystem {
    fn update(&mut self, entity_manager: &mut EntityManager, resources: &mut ResourceManager) -> SystemResult {
        // プレイヤーの状態とボードの状態を取得
        let board_state = resources.get::<BoardResource>();
        let player_state = resources.get::<PlayerStateResource>();
        
        if let (Ok(board_state), Ok(player_state)) = (board_state, player_state) {
            // ボードとプレイヤーの状態の取得に成功した場合
            let board_state_ref = board_state.borrow();
            let player_state_ref = player_state.borrow();
            
            // 型変換にも成功した場合のみ処理を続行
            if let (Some(board), Some(player)) = (
                board_state_ref.downcast_ref::<BoardResource>(),
                player_state_ref.downcast_ref::<PlayerStateResource>()
            ) {
                // マウスが左クリックされた時だけ処理
                if player.mouse_state.get_state() != MouseState::LEFT_DOWN {
                    return SystemResult::Ok;
                }
                
                // マウス座標からセルの位置を計算
                let cell_size = board.cell_size as f64;
                let col = (player.mouse_state.x as f64 / cell_size) as usize;
                let row = (player.mouse_state.y as f64 / cell_size) as usize;
                
                // 範囲外チェック
                if row >= board.height || col >= board.width {
                    return SystemResult::Ok;
                }
                
                // 可変参照を取得して処理
                if let Ok(board_state_mut) = resources.get_mut::<BoardResource>() {
                    if let Some(mut board_mut) = board_state_mut.borrow_mut().downcast_mut::<BoardResource>() {
                        // 最初のクリックかどうかを取得
                        let first_click = board_mut.first_click;
                        
                        // セルを公開
                        match reveal_cell(row, col, entity_manager, board_mut, first_click) {
                            Ok(_) => (),
                            Err(_) => return SystemResult::Error,
                        }
                    }
                }
            }
        }
        
        SystemResult::Ok
    }
} 