/**
 * ボード初期化システム
 * 
 * ボードとセルのエンティティを初期化するシステム
 */
use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
use wasm_bindgen::JsValue;
use rand::Rng;

use crate::entities::EntityManager;
use crate::systems::system_registry::DeltaTime;
use crate::resources::{BoardConfigResource, BoardStateResource};
use crate::components::{CellStateComponent, CellContentComponent, GridPositionComponent};
use crate::entity::EntityId;
use crate::models::CellValue;

/// ボード初期化システム
/// ボードとセルのエンティティを初期化
pub fn board_init_system(
    entity_manager: &mut EntityManager,
    resources: &mut HashMap<&'static str, Rc<RefCell<dyn std::any::Any>>>,
    _delta_time: DeltaTime,
) -> Result<(), JsValue> {
    // BoardConfigResourceとBoardStateResourceを取得
    let board_config = resources.get("board_config").and_then(|res| {
        res.borrow().downcast_ref::<BoardConfigResource>().cloned()
    });
    
    let mut board_state_updated = false;
    let board_state = resources.get("board_state").and_then(|res| {
        res.borrow().downcast_ref::<BoardStateResource>().cloned()
    });
    
    // 設定とボード状態がある場合のみ初期化を行う
    if let (Some(config), Some(mut state)) = (board_config, board_state) {
        // ボードが未初期化の場合、初期化を行う
        if state.cell_grid.is_empty() {
            initialize_board(&mut state, &config, entity_manager)?;
            board_state_updated = true;
        }
        
        // 状態が更新された場合、リソースを更新
        if board_state_updated {
            if let Some(board_state_rc) = resources.get("board_state") {
                if let Some(mut board_state_mut) = board_state_rc.borrow_mut().downcast_mut::<BoardStateResource>() {
                    *board_state_mut = state;
                }
            }
        }
    }
    
    Ok(())
}

/// ボードの初期化処理
fn initialize_board(
    board_state: &mut BoardStateResource,
    board_config: &BoardConfigResource,
    entity_manager: &mut EntityManager,
) -> Result<(), JsValue> {
    let width = board_config.width;
    let height = board_config.height;
    
    // セルグリッドを初期化
    board_state.initialize_grid(width, height);
    board_state.first_click = false;
    board_state.is_game_over = false;
    board_state.is_win = false;
    board_state.remaining_safe_cells = width * height - board_config.mine_count;
    board_state.flagged_count = 0;
    
    // 各セルのエンティティを作成
    for row in 0..height {
        for col in 0..width {
            // セルエンティティを作成
            let cell_entity = entity_manager.create_entity();
            
            // 位置コンポーネントを追加
            let position = GridPositionComponent {
                row,
                col,
            };
            entity_manager.add_component(cell_entity, position);
            
            // セル状態コンポーネントを追加
            let cell_state = CellStateComponent::default();
            entity_manager.add_component(cell_entity, cell_state);
            
            // セルの内容コンポーネントを作成
            let cell_content = CellContentComponent {
                value: CellValue::Empty(0)
            };
            entity_manager.add_component(cell_entity, cell_content);
            
            // グリッドに登録
            board_state.cell_grid[row][col] = cell_entity;
        }
    }
    
    Ok(())
}

/// 指定した位置を避けて地雷を配置
pub fn place_mines(
    board_state: &mut BoardStateResource,
    board_config: &BoardConfigResource,
    entity_manager: &mut EntityManager,
    avoid_row: usize,
    avoid_col: usize,
) -> Result<(), JsValue> {
    let width = board_config.width;
    let height = board_config.height;
    let mine_count = board_config.mine_count;
    let total_cells = width * height;
    
    if mine_count >= total_cells {
        return Err(JsValue::from_str("地雷の数がセルの総数を超えています"));
    }
    
    // 地雷の配置位置をランダムに決定
    let mut rng = rand::thread_rng();
    let mut mine_positions = Vec::new();
    
    // 避けるべき位置とその周囲
    let mut avoid_positions = Vec::new();
    if board_config.safe_first_click {
        // 避ける位置とその周囲8マスを追加
        for r in avoid_row.saturating_sub(1)..=std::cmp::min(avoid_row + 1, height - 1) {
            for c in avoid_col.saturating_sub(1)..=std::cmp::min(avoid_col + 1, width - 1) {
                avoid_positions.push((r, c));
            }
        }
    } else {
        // 最初のクリック位置のみ避ける
        avoid_positions.push((avoid_row, avoid_col));
    }
    
    // 地雷を配置
    let mut mines_placed = 0;
    while mines_placed < mine_count {
        let row = rng.gen_range(0..height);
        let col = rng.gen_range(0..width);
        
        // この位置が避けるべき位置でなく、既に地雷が配置されていない場合
        if !avoid_positions.contains(&(row, col)) && !mine_positions.contains(&(row, col)) {
            mine_positions.push((row, col));
            mines_placed += 1;
        }
    }
    
    // 地雷をエンティティに設定
    for (row, col) in mine_positions {
        if let Some(entity_id) = board_state.get_cell_entity(row, col) {
            if let Some(mut cell_content) = entity_manager.get_component_mut::<CellContentComponent>(entity_id) {
                cell_content.value = CellValue::Mine;
            }
        }
    }
    
    // 周囲の地雷数を計算して空のセルに設定
    for row in 0..height {
        for col in 0..width {
            if let Some(entity_id) = board_state.get_cell_entity(row, col) {
                // この位置のセルが地雷でなければ、周囲の地雷数を数える
                if let Some(cell_content) = entity_manager.get_component::<CellContentComponent>(entity_id) {
                    if let CellValue::Empty(_) = cell_content.value {
                        // 周囲の地雷数を数える
                        let mut mine_count = 0;
                        
                        // 隣接するセルを調べる
                        let adjacent_positions = board_state.get_adjacent_positions(row, col, board_config);
                        for (adj_row, adj_col) in adjacent_positions {
                            if let Some(adj_entity) = board_state.get_cell_entity(adj_row, adj_col) {
                                if let Some(adj_content) = entity_manager.get_component::<CellContentComponent>(adj_entity) {
                                    if let CellValue::Mine = adj_content.value {
                                        mine_count += 1;
                                    }
                                }
                            }
                        }
                        
                        // 周囲の地雷数を設定
                        if let Some(mut cell_content) = entity_manager.get_component_mut::<CellContentComponent>(entity_id) {
                            cell_content.value = CellValue::Empty(mine_count);
                        }
                    }
                }
            }
        }
    }
    
    Ok(())
} 