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
use wasm_bindgen::prelude::*;
use log;
use crate::resources::resource_manager::ResourceManager;
use crate::systems::optimized::system_scheduler::DeltaTime;

use crate::entities::EntityManager;
use crate::resources::{BoardConfig, BoardResource};
use crate::components::{CellStateComponent, CellContentComponent, GridPositionComponent};
use crate::components::board_components::CellState;
use crate::entities::entity::EntityId;
use crate::models::CellValue;
use crate::resources::GameStateResource;
use crate::resources::board_resources::{BoardConfigResource, BoardStateResource};
use crate::resources::event_queue_resource::EventQueueResource;

/// ボード初期化システム - ゲーム開始時にボードを初期化する
pub fn board_init_system(
    resources: &mut ResourceManager,
    _delta_time: DeltaTime
) -> Result<(), JsValue> {
    // 必要なリソースの取得
    let board_config_rc = match resources.get::<BoardConfigResource>() {
        Ok(config_rc) => config_rc,
        Err(_) => {
            // デフォルト設定を作成して登録
            let default_config = BoardConfigResource::default();
            resources.add_or_update(default_config.clone());
            // 再度取得を試みる
            resources.get::<BoardConfigResource>().unwrap()
        }
    };
    
    let board_state_rc = match resources.get::<BoardStateResource>() {
        Ok(state_rc) => state_rc,
        Err(_) => {
            // デフォルト状態を作成して登録
            let default_state = BoardStateResource::default();
            resources.add_or_update(default_state.clone());
            // 再度取得を試みる
            resources.get::<BoardStateResource>().unwrap()
        }
    };
    
    // 可変で借用してダウンキャスト
    let mut board_state = board_state_rc.borrow_mut();
    let mut board_state = board_state.downcast_mut::<BoardStateResource>().unwrap();
    
    // 初期化済みかチェック（すでに初期化されていたら何もしない）
    if board_state.is_initialized {
        return Ok(());
    }
    
    // エンティティマネージャーの取得
    let entity_manager_rc = match resources.get::<EntityManager>() {
        Ok(manager) => manager,
        Err(_) => {
            log::warn!("EntityManager not found, cannot initialize board");
            return Ok(());
        }
    };
    
    // 可変で借用してダウンキャスト
    let mut entity_manager_ref = entity_manager_rc.borrow_mut();
    if let Some(entity_manager) = entity_manager_ref.downcast_mut::<EntityManager>() {
        // ボードの初期化
        initialize_board_grid(&mut board_state, &board_config_rc.borrow().downcast_ref::<BoardConfigResource>().unwrap(), entity_manager)?;
        
        // 初期化フラグを設定
        board_state.is_initialized = true;
    } else {
        log::warn!("Failed to downcast EntityManager");
        return Ok(());
    }
    
    // borrow_mutによって取得した可変参照はここで終了
    // 新しいBoardStateResourceを作成して更新する
    let updated_board_state = BoardStateResource {
        is_initialized: true,
        ..(*board_state).clone()
    };
    
    // リソースの更新
    resources.add_or_update(updated_board_state);
    
    // 初期化完了イベントを発行
    if let Ok(event_queue_rc) = resources.get::<EventQueueResource>() {
        let mut event_queue_ref = event_queue_rc.borrow_mut();
        if let Some(event_queue) = event_queue_ref.downcast_mut::<EventQueueResource>() {
            event_queue.push_event(crate::resources::event_queue_resource::GameEvent::UIClick("board_initialized".to_string()));
        }
    }
    
    Ok(())
}

/// ボードグリッドの初期化
fn initialize_board_grid(
    board_state: &mut BoardStateResource,
    board_config: &BoardConfigResource,
    entity_manager: &mut EntityManager,
) -> Result<(), JsValue> {
    // グリッドの初期化
    board_state.initialize_grid(board_config.width, board_config.height);
    
    // 残りの安全セル数を初期化
    board_state.remaining_safe_cells = board_config.total_cells() - board_config.mine_count;
    
    // 各セルのエンティティを作成
    for row in 0..board_config.height {
        for col in 0..board_config.width {
            // セルエンティティを作成
            let cell_entity = entity_manager.create_entity();
            
            // 位置コンポーネントを追加
            let position = GridPositionComponent {
                row,
                col,
            };
            entity_manager.add_component(cell_entity, position);
            
            // セル状態コンポーネントを追加（初期状態は隠れている）
            let cell_state = CellStateComponent {
                state: CellState::Hidden,
            };
            entity_manager.add_component(cell_entity, cell_state);
            
            // セルの内容コンポーネントを追加（初期状態は空）
            let cell_content = CellContentComponent {
                value: CellValue::Empty(0),
            };
            entity_manager.add_component(cell_entity, cell_content);
            
            // エンティティIDをグリッドに登録
            board_state.cell_grid.insert((row, col), cell_entity);
        }
    }
    
    // 地雷は最初のクリックまで配置しない（safe_first_clickフラグを考慮）
    // 初回クリックは別のシステムで処理される
    
    Ok(())
}

/// 指定した位置を避けて地雷を配置
pub fn place_mines(
    board_state: &mut BoardStateResource,
    entity_manager: &mut EntityManager,
    board_config: &BoardConfigResource,
    avoid_row: usize,
    avoid_col: usize,
) -> Result<(), JsValue> {
    let width = board_config.width;
    let height = board_config.height;
    let mine_count = board_config.mine_count;
    let total_cells = width * height;
    
    // 避けるセルのインデックス
    let avoid_index = avoid_row * width + avoid_col;
    
    // 利用可能なセルのインデックスのリスト（0からtotal_cells-1まで）
    let mut available_indices: Vec<usize> = (0..total_cells).collect();
    
    // 避けるセルを利用可能なインデックスから削除
    if let Some(pos) = available_indices.iter().position(|&idx| idx == avoid_index) {
        available_indices.remove(pos);
    }
    
    // ランダム生成器の初期化
    let mut rng = rand::thread_rng();
    
    // 地雷を配置
    let mine_indices: Vec<usize> = available_indices
        .into_iter()
        .take(mine_count)
        .collect();
    
    // 地雷を配置し、周囲の地雷カウントを更新
    for mine_idx in mine_indices {
        let mine_row = mine_idx / width;
        let mine_col = mine_idx % width;
        
        // 地雷のセルエンティティを取得
        if let Some(mine_entity) = board_state.get_cell_entity(mine_row, mine_col) {
            // 地雷コンポーネントを更新
            if let Some(mut content) = entity_manager.get_component_mut::<CellContentComponent>(mine_entity) {
                content.value = CellValue::Mine;
            }
            
            // 周囲8方向のセルの地雷カウントを更新
            let adjacent_positions = board_state.get_adjacent_positions(mine_row, mine_col, &board_config);
            
            for (adj_row, adj_col) in adjacent_positions {
                if let Some(adj_entity) = board_state.get_cell_entity(adj_row, adj_col) {
                    if let Some(mut content) = entity_manager.get_component_mut::<CellContentComponent>(adj_entity) {
                        // 隣接セルが地雷でなければ、カウントを増やす
                        if let CellValue::Empty(count) = content.value {
                            content.value = CellValue::Empty(count + 1);
                        }
                    }
                }
            }
        }
    }
    
    // 初回クリックフラグを更新
    board_state.first_click = false;
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resources::resource_manager::ResourceManager;
    use crate::components::component_factory::ComponentFactory;
    use crate::entities::entity_manager::EntityManager;
    
    /// ボード初期化システムのテスト
    #[test]
    fn test_board_init_system() {
        // リソースマネージャーの準備
        let mut resources = ResourceManager::new();
        
        // エンティティマネージャーの準備
        let entity_manager = EntityManager::new();
        resources.add_resource("entity_manager", entity_manager);
        
        // ボード設定の準備
        let config = BoardConfigResource::new(5, 5, 5, true);
        resources.add_resource("board_config", config);
        
        // ボード初期化システムの実行
        let result = board_init_system(&mut resources, DeltaTime(0.0));
        assert!(result.is_ok());
        
        // ボード状態の確認
        let board_state = resources.get_resource::<BoardStateResource>("board_state").unwrap();
        
        // 初期化フラグの確認
        assert!(board_state.is_initialized);
        
        // セルエンティティの数の確認（5x5=25）
        assert_eq!(board_state.cell_grid.len(), 25);
        
        // 残りの安全セル数の確認（25-5=20）
        assert_eq!(board_state.remaining_safe_cells, 20);
        
        // 初回クリックフラグの確認
        assert!(board_state.first_click);
    }
    
    /// 地雷配置のテスト
    #[test]
    fn test_place_mines() {
        // リソースマネージャーの準備
        let mut resources = ResourceManager::new();
        
        // エンティティマネージャーの準備
        let mut entity_manager = EntityManager::new();
        
        // ボード設定の準備
        let config = BoardConfigResource::new(5, 5, 5, true);
        entity_manager.add_resource("board_config", config);
        
        // ボード状態の準備
        let mut board_state = BoardStateResource::new();
        
        // グリッドの初期化
        board_state.initialize_grid(5, 5);
        
        // 各セルのエンティティを作成
        for row in 0..5 {
            for col in 0..5 {
                let cell_entity = entity_manager.create_entity();
                
                // 位置コンポーネント
                entity_manager.add_component(cell_entity, GridPositionComponent { row, col });
                
                // 状態コンポーネント
                entity_manager.add_component(cell_entity, CellStateComponent { state: CellState::Hidden });
                
                // 内容コンポーネント
                entity_manager.add_component(cell_entity, CellContentComponent { value: CellValue::Empty(0) });
                
                // エンティティIDをグリッドに登録
                board_state.cell_grid.insert((row, col), cell_entity);
            }
        }
        
        // 地雷を配置（中央のセルを避ける）
        let result = place_mines(&mut board_state, &mut entity_manager, &config, 2, 2);
        assert!(result.is_ok());
        
        // 地雷の数をカウント
        let mut mine_count = 0;
        for row in 0..5 {
            for col in 0..5 {
                if let Some(entity_id) = board_state.get_cell_entity(row, col) {
                    if let Some(content) = entity_manager.get_component::<CellContentComponent>(entity_id) {
                        if let CellValue::Mine = content.value {
                            mine_count += 1;
                        }
                    }
                }
            }
        }
        
        // 地雷数の確認
        assert_eq!(mine_count, 5);
        
        // 中央のセルが地雷でないことを確認
        if let Some(entity_id) = board_state.get_cell_entity(2, 2) {
            if let Some(content) = entity_manager.get_component::<CellContentComponent>(entity_id) {
                if let CellValue::Mine = content.value {
                    panic!("Center cell should not be a mine");
                }
            }
        }
        
        // 初回クリックフラグの確認
        assert!(!board_state.first_click);
    }
} 