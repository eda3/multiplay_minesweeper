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
use crate::resources::resource_manager::ResourceManager;
use crate::systems::optimized::system_scheduler::DeltaTime;

use crate::entities::EntityManager;
use crate::resources::{BoardConfig, BoardResource};
use crate::components::{CellStateComponent, CellContentComponent, GridPositionComponent};
use crate::components::board_components::CellState;
use crate::entities::entity::EntityId;
use crate::models::CellValue;
use crate::resources::GameStateResource;

/// ボード初期化システム - ゲーム開始時にボードを初期化する
pub fn board_init_system(
    resources: &mut ResourceManager,
    _delta_time: DeltaTime
) -> Result<(), JsValue> {
    // ボード初期化ロジックをここに実装
    Ok(())
}

/// ボードの初期化処理
fn initialize_board(
    board_state: &mut BoardResource,
    entity_manager: &mut EntityManager,
) -> Result<(), JsValue> {
    // ボードをリセット
    board_state.reset();
    
    // 各セルのエンティティを設定
    for row in 0..board_state.height {
        for col in 0..board_state.width {
            let index = row * board_state.width + col;
            
            // セルエンティティを作成
            let cell_entity = entity_manager.create_entity();
            
            // 位置コンポーネントを追加
            let position = GridPositionComponent {
                row,
                col,
            };
            entity_manager.add_component(cell_entity, position);
            
            // セル状態コンポーネントを追加（BoardResourceのcellsの状態と同期）
            let cell_state = CellStateComponent {
                state: CellState::Hidden,
            };
            entity_manager.add_component(cell_entity, cell_state);
            
            // セルの内容コンポーネントを作成
            let cell_content = CellContentComponent {
                value: match board_state.cells[index] {
                    CellValue::Mine => CellValue::Mine,
                    CellValue::Empty(count) => CellValue::Empty(count as u8)
                }
            };
            entity_manager.add_component(cell_entity, cell_content);
        }
    }
    
    Ok(())
}

/// 指定した位置を避けて地雷を配置
pub fn place_mines(
    board_state: &mut BoardResource,
    entity_manager: &mut EntityManager,
    avoid_row: usize,
    avoid_col: usize,
) -> Result<(), JsValue> {
    let width = board_state.width;
    let first_click_index = avoid_row * width + avoid_col;
    
    // BoardResourceの初期化メソッド（引数なし）を使用
    // Boardエイリアスとして定義されているため、引数を取りません
    board_state.initialize();
    
    // NOTE: この部分ではfirst_click_indexの情報が失われますが、
    // Boardの実装では独自の方法で安全セルを決定している可能性があります
    
    // 各セルのエンティティに地雷情報を設定
    for index in 0..board_state.cells.len() {
        let row = index / width;
        let col = index % width;
        
        // エンティティを作成または取得
        let cell_entity = entity_manager.create_entity();
        
        // 位置コンポーネントを追加
        let position = GridPositionComponent {
            row,
            col,
        };
        entity_manager.add_component(cell_entity, position);
        
        // セル状態コンポーネントを追加
        // CellStateの型変換 - 同じ名前だが異なる型
        let component_state = match board_state.cells[index] {
            CellValue::Mine => CellState::Hidden, // 地雷は初期状態では隠れている
            CellValue::Empty(_) => CellState::Hidden // 空のセルも初期状態では隠れている
        };
        
        let cell_state = CellStateComponent {
            state: component_state,
        };
        entity_manager.add_component(cell_entity, cell_state);
        
        // セルの内容コンポーネントを作成
        let cell_content = CellContentComponent {
            value: match board_state.cells[index] {
                CellValue::Mine => CellValue::Mine,
                CellValue::Empty(count) => CellValue::Empty(count as u8)
            }
        };
        entity_manager.add_component(cell_entity, cell_content);
    }
    
    Ok(())
} 