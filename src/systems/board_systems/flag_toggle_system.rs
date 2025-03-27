/**
 * フラグトグルシステム
 * 
 * セルの右クリックでフラグの表示・非表示を切り替える
 */
use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
use wasm_bindgen::JsValue;

use crate::entities::EntityManager;
use crate::systems::system_registry::DeltaTime;
use crate::resources::{
    BoardConfigResource,
    BoardStateResource,
    PlayerStateResource,
    MouseState,
    TimeResource,
    Resource
};
use crate::components::board_components::{CellStateComponent, CellState, CellContentComponent};
use crate::ecs::system::{System, SystemResult};
use crate::resources::ResourceManager;

/// フラグトグルシステム
/// 指定されたセルのフラグ状態を切り替える
pub fn flag_toggle_system(
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
            // このモジュール内のtoggle_flagを使用して外部から呼び出す
            
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

/// 指定されたセルのフラグ状態を切り替える
/// 返値: フラグ状態が変更されたかどうか
pub fn toggle_flag(
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
    
    // すでに公開済みの場合は何もしない
    if cell_state.state == CellState::Revealed {
        return Ok(false);
    }
    
    // フラグ状態を切り替え
    if let Some(mut cell_state) = entity_manager.get_component_mut::<CellStateComponent>(cell_entity) {
        match cell_state.state {
            CellState::Flagged => {
                // フラグ -> 疑問符 (フラグを外す)
                cell_state.state = CellState::Questioned;
                board_state.flagged_count -= 1;
            },
            CellState::Questioned => {
                // 疑問符 -> 通常 (疑問符を外す)
                cell_state.state = CellState::Hidden;
            },
            CellState::Hidden => {
                // 通常 -> フラグ
                cell_state.state = CellState::Flagged;
                board_state.flagged_count += 1;
            },
            _ => {}
        }
        
        return Ok(true);
    }
    
    Ok(false)
}

/**
 * フラグトグルシステム
 * 
 * セルの右クリックでフラグの表示・非表示を切り替える
 */
pub struct FlagToggleSystem {
    // 必要なステート
}

impl FlagToggleSystem {
    pub fn new() -> Self {
        Self {
            // 初期化
        }
    }
}

impl System for FlagToggleSystem {
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
        
        // マウスの右ボタンが押されていない場合は何もしない
        if player_state.mouse_state != MouseState::RightDown {
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
        
        // 可変参照へ変換（参照カウンタを増やさないため）
        let mut board_state = match resources.get_mut::<BoardStateResource>() {
            Some(state) => state,
            None => return SystemResult::Ok,
        };
        
        // ゲームが終了していたら何もしない
        if board_state.is_game_over || board_state.is_win {
            return SystemResult::Ok;
        }
        
        // クリックされたセルのエンティティIDを取得
        let cell_entity = match board_state.get_cell_entity(row, col) {
            Some(entity) => entity,
            None => return SystemResult::Ok, // セルが存在しない場合
        };
        
        // セルの状態コンポーネントを取得
        let cell_state = match entity_manager.get_component::<CellStateComponent>(cell_entity) {
            Some(state) => state,
            None => return SystemResult::Ok, // コンポーネントがない場合
        };
        
        // セルが既に公開されている場合は何もしない
        if cell_state.state == CellState::Revealed {
            return SystemResult::Ok;
        }
        
        // セルの状態を変更（フラグを切り替え）
        if let Some(mut cell_state) = entity_manager.get_component_mut::<CellStateComponent>(cell_entity) {
            match cell_state.state {
                CellState::Flagged => {
                    cell_state.state = CellState::Questioned;
                    board_state.flagged_count -= 1;
                },
                CellState::Questioned => {
                    cell_state.state = CellState::Hidden;
                },
                CellState::Hidden => {
                    cell_state.state = CellState::Flagged;
                    board_state.flagged_count += 1;
                },
                _ => {}
            }
        }
        
        SystemResult::Ok
    }
} 