/**
 * セル公開システム
 * 
 * セルをクリックして内容を公開するシステム
 */
use std::collections::HashMap;
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
        // 周囲のセルを再帰的に開く
        let adjacents = get_adjacent_cells(row, col, board.width, board.height);
        for (adj_row, adj_col) in adjacents {
            reveal_cell(adj_row, adj_col, entity_manager, board, false)?;
        }
    }

    // 勝利条件をチェック
    if check_win_condition(board) {
        board.win = true;
        board.reveal_all_mines();
    }

    Ok(false) // 爆発しなかったのでfalseを返す
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
                if player.mouse_state.get_state() != MouseState::LeftDown {
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