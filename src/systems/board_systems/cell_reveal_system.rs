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
    board_state: &mut BoardResource,
    board_config: &BoardConfig,
) -> Result<bool, JsValue> {
    // ゲームが終了している条件をcheck_win_conditionの結果で判断
    if board_state.check_win_condition() {
        return Ok(false);
    }
    
    // 座標が有効かチェック
    let width = board_config.width;
    if row >= board_config.height || col >= width {
        return Ok(false);
    }
    
    // セルのインデックスを計算
    let cell_index = row * width + col;
    
    // すでに公開済み、またはフラグが立っている場合は何もしない
    if board_state.cells[cell_index].is_revealed() || board_state.cells[cell_index].is_flagged() {
        return Ok(false);
    }
    
    // 最初のクリックの場合
    if board_state.first_click {
        // ボードを初期化（地雷配置）
        board_state.initialize(cell_index);
    }
    
    // セルを公開
    let exploded = board_state.reveal_cell(cell_index);
    
    if exploded {
        // 地雷を踏んだ場合はすべての地雷を表示
        board_state.reveal_all_mines();
        return Ok(true);
    }
    
    // 勝利条件をチェック
    let is_win = board_state.check_win_condition();
    
    Ok(true)
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
                // ゲームが終了している場合は何もしない
                if board.check_win_condition() {
                    return SystemResult::Ok;
                }
                
                // マウスが左クリックされた時だけ処理
                if player.mouse_state.get_state() != MouseState::LeftDown {
                    return SystemResult::Ok;
                }
                
                // マウス座標からセルの位置を計算
                let cell_size = board.config.cell_size as f64;
                let col = (player.mouse_state.x as f64 / cell_size) as usize;
                let row = (player.mouse_state.y as f64 / cell_size) as usize;
                
                // 範囲外チェック
                if row >= board.config.height || col >= board.config.width {
                    return SystemResult::Ok;
                }
                
                // 可変参照を取得して処理
                if let Ok(board_state_mut) = resources.get_mut::<BoardResource>() {
                    if let Some(mut board_mut) = board_state_mut.borrow_mut().downcast_mut::<BoardResource>() {
                        // configを事前にコピー
                        let config = board_mut.config.clone();
                        
                        // セルを公開
                        match reveal_cell(row, col, entity_manager, &mut board_mut, &config) {
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