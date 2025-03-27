/**
 * セル公開システム
 * 
 * セルをクリックして内容を公開するシステム
 */
use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
use wasm_bindgen::JsValue;

use crate::entities::EntityManager;
use crate::entities::EntityId;
use crate::systems::system_registry::DeltaTime;
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

/// セル公開システム
/// 指定されたセルを公開する
pub fn cell_reveal_system(
    entity_manager: &mut EntityManager,
    resources: &mut HashMap<&'static str, Rc<RefCell<dyn std::any::Any>>>,
    _delta_time: DeltaTime,
) -> Result<(), JsValue> {
    // 必要なリソースを取得
    let board_config = resources.get("board_config").and_then(|res| {
        res.borrow().downcast_ref::<BoardConfigResource>().cloned()
    });
    
    let board_state = resources.get("board_state").and_then(|res| {
        res.borrow().downcast_ref::<BoardStateResource>().cloned()
    });
    
    // 両方のリソースが存在する場合のみ処理を続行
    if let (Some(config), Some(mut state)) = (board_config, board_state) {
        // ゲームが終了していない場合のみ処理を実行
        if !state.is_game_over && !state.is_win {
            // ここでは実際に処理するセルはイベントシステムから指定される想定
            // このモジュール内のreveal_cellを使用して外部から呼び出す
            
            // 状態に変更があった場合、リソースを更新
            if let Some(board_state_rc) = resources.get("board_state") {
                if let Some(mut board_state_mut) = board_state_rc.borrow_mut().downcast_mut::<BoardStateResource>() {
                    *board_state_mut = state;
                }
            }
        }
    }
    
    Ok(())
}

/// 指定されたセルを公開する
/// 返値: セルが公開されたかどうか
pub fn reveal_cell(
    row: usize,
    col: usize,
    entity_manager: &mut EntityManager,
    board_state: &mut BoardStateResource,
    board_config: &BoardConfigResource,
) -> Result<bool, JsValue> {
    // ゲームが終了している場合は何もしない
    if board_state.is_game_over || board_state.is_win {
        return Ok(false);
    }
    
    // 座標が有効かチェック
    if !board_config.is_valid_position(row, col) {
        return Ok(false);
    }
    
    // セルのエンティティを取得
    let cell_entity = match board_state.get_cell_entity(row, col) {
        Some(entity) => entity,
        None => return Ok(false),
    };
    
    // セルの状態を取得
    let cell_state = match entity_manager.get_component::<CellStateComponent>(cell_entity) {
        Some(state) => state.clone(),
        None => return Ok(false),
    };
    
    // すでに公開済み、またはフラグが立っている場合は何もしない
    if cell_state.state == CellState::Revealed || cell_state.state == CellState::Flagged {
        return Ok(false);
    }
    
    // 最初のクリックの場合
    if board_state.first_click {
        board_state.first_click = false;
        
        // 地雷の配置をここで行う
        // この実装は別のシステムで行われている想定
    }
    
    // セルの内容を取得
    let cell_content = match entity_manager.get_component::<CellContentComponent>(cell_entity) {
        Some(content) => content.clone(),
        None => return Ok(false),
    };
    
    // セルの状態を公開状態に変更
    if let Some(mut cell_state) = entity_manager.get_component_mut::<CellStateComponent>(cell_entity) {
        cell_state.state = CellState::Revealed;
    }
    
    // セルの内容に応じた処理
    match cell_content.value {
        CellValue::Mine => {
            // 地雷を踏んだ場合はゲームオーバー
            board_state.is_game_over = true;
            
            // すべての地雷を公開
            reveal_all_mines(entity_manager, board_state);
            
            return Ok(true);
        },
        CellValue::Empty(0) => {
            // 隣接地雷がない場合は周囲のセルも自動的に公開
            board_state.remaining_safe_cells -= 1;
            auto_reveal_adjacent_cells(row, col, entity_manager, board_state, board_config)?;
            
            // 勝利条件をチェック
            check_win_condition(board_state);
            
            return Ok(true);
        },
        CellValue::Empty(_) => {
            // 通常のセル
            board_state.remaining_safe_cells -= 1;
            
            // 勝利条件をチェック
            check_win_condition(board_state);
            
            return Ok(true);
        }
    }
}

/// 隣接するセルを自動的に公開する（再帰的に）
fn auto_reveal_adjacent_cells(
    row: usize,
    col: usize,
    entity_manager: &mut EntityManager,
    board_state: &mut BoardStateResource,
    board_config: &BoardConfigResource,
) -> Result<(), JsValue> {
    // 隣接するセルの座標を取得
    let adjacent_positions = board_state.get_adjacent_positions(row, col, board_config);
    
    // 各隣接セルに対して処理
    for (adj_row, adj_col) in adjacent_positions {
        // 隣接セルのエンティティを取得
        if let Some(adj_entity) = board_state.get_cell_entity(adj_row, adj_col) {
            // すでに公開済みのセルは処理しない
            if let Some(adj_state) = entity_manager.get_component::<CellStateComponent>(adj_entity) {
                if adj_state.state == CellState::Revealed {
                    continue;
                }
                
                // フラグが立っているセルは公開しない
                if adj_state.state == CellState::Flagged {
                    continue;
                }
                
                // セルの内容を確認
                if let Some(adj_content) = entity_manager.get_component::<CellContentComponent>(adj_entity) {
                    match adj_content.value {
                        CellValue::Empty(0) => {
                            // 隣接地雷がない場合は公開して再帰的に処理
                            if let Some(mut adj_state) = entity_manager.get_component_mut::<CellStateComponent>(adj_entity) {
                                // 未公開のセルのみ処理
                                if adj_state.state != CellState::Revealed {
                                    adj_state.state = CellState::Revealed;
                                    board_state.remaining_safe_cells -= 1;
                                    
                                    // 再帰的に隣接セルを公開
                                    auto_reveal_adjacent_cells(adj_row, adj_col, entity_manager, board_state, board_config)?;
                                }
                            }
                        },
                        CellValue::Empty(_) => {
                            // 数字のセルは公開するだけ
                            if let Some(mut adj_state) = entity_manager.get_component_mut::<CellStateComponent>(adj_entity) {
                                // 未公開のセルのみ処理
                                if adj_state.state != CellState::Revealed {
                                    adj_state.state = CellState::Revealed;
                                    board_state.remaining_safe_cells -= 1;
                                }
                            }
                        },
                        CellValue::Mine => {
                            // 地雷は何もしない
                        }
                    }
                }
            }
        }
    }
    
    Ok(())
}

/// すべての地雷を公開する
fn reveal_all_mines(
    entity_manager: &mut EntityManager,
    board_state: &mut BoardStateResource,
) {
    // すべてのセルエンティティを走査
    for (_, entity_id) in board_state.cell_grid.iter() {
        // 地雷を含むセルを特定
        if let Some(content) = entity_manager.get_component::<CellContentComponent>(*entity_id) {
            if let CellValue::Mine = content.value {
                // 地雷セルを公開
                if let Some(mut state) = entity_manager.get_component_mut::<CellStateComponent>(*entity_id) {
                    state.state = CellState::Revealed;
                }
            }
        }
    }
}

/// 勝利条件をチェック
fn check_win_condition(board_state: &mut BoardStateResource) {
    // 残りの安全なセルがなければ勝利
    if board_state.remaining_safe_cells == 0 {
        board_state.is_win = true;
    }
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
        let player_state = match resources.get::<PlayerStateResource>() {
            Some(state) => state,
            None => return SystemResult::Ok, // プレイヤー状態がなければ何もしない
        };
        
        let board_state = match resources.get::<BoardStateResource>() {
            Some(state) => state,
            None => return SystemResult::Ok, // ボード状態がなければ何もしない
        };
        
        // ゲームが終了している場合は何もしない
        if board_state.is_game_over || board_state.is_win {
            return SystemResult::Ok;
        }
        
        // マウスの左ボタンが押されていない場合は何もしない
        if player_state.mouse_state != MouseState::LeftDown {
            return SystemResult::Ok;
        }
        
        // ボード設定を取得
        let board_config = match resources.get::<BoardConfigResource>() {
            Some(config) => config,
            None => return SystemResult::Ok, // ボード設定がなければ何もしない
        };
        
        // マウス座標からセルの位置を計算
        let cell_size = 30.0; // 本来はBoardConfigResourceから取得
        let col = (player_state.mouse_x as f64 / cell_size) as usize;
        let row = (player_state.mouse_y as f64 / cell_size) as usize;
        
        // 範囲外のクリックは無視
        if row >= board_config.height || col >= board_config.width {
            return SystemResult::Ok;
        }
        
        // 可変参照へ変換
        let mut board_state = match resources.get_mut::<BoardStateResource>() {
            Some(state) => state,
            None => return SystemResult::Ok,
        };
        
        // セル公開処理を実行
        match reveal_cell(row, col, entity_manager, &mut board_state, &board_config) {
            Ok(_) => (),
            Err(_) => return SystemResult::Error,
        }
        
        SystemResult::Ok
    }
} 