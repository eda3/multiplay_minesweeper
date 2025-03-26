/**
 * ボードシステム
 * 
 * マインスイーパーのボード操作を管理するシステム
 */
use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
use wasm_bindgen::JsValue;

use crate::entities::EntityManager;
use crate::systems::system_registry::DeltaTime;
use crate::resources::BoardResource;
use crate::components::Board;

/// ボードシステム - ボードの状態を更新
pub fn board_system(
    entity_manager: &mut EntityManager,
    resources: &mut HashMap<&'static str, Rc<RefCell<dyn std::any::Any>>>,
    _delta_time: DeltaTime,
) -> Result<(), JsValue> {
    // BoardResourceを取得
    let board_resource = resources.get("board").and_then(|res| {
        res.clone().borrow_mut().downcast_mut::<BoardResource>().map(|r| r.clone())
    });
    
    if let Some(board) = board_resource {
        // ボードが更新された場合、各エンティティの状態を更新
        if board.is_updated {
            update_board_entities(entity_manager, &board)?;
            
            // 更新フラグをリセット
            if let Some(board_rc) = resources.get("board") {
                if let Some(mut board_res) = board_rc.borrow_mut().downcast_mut::<BoardResource>() {
                    board_res.is_updated = false;
                }
            }
        }
        
        // 勝利条件のチェック
        check_win_condition(entity_manager, resources, &board)?;
    }
    
    Ok(())
}

/// ボードエンティティの更新
fn update_board_entities(
    entity_manager: &mut EntityManager,
    board: &BoardResource,
) -> Result<(), JsValue> {
    // ボードコンポーネントを持つエンティティを検索
    let board_entities = entity_manager.find_entities_with_component::<Board>();
    
    for entity_id in board_entities {
        // ボードコンポーネントを更新
        if let Some(mut board_comp) = entity_manager.get_component_mut::<Board>(entity_id) {
            // ボードデータをリソースから更新
            board_comp.width = board.width;
            board_comp.height = board.height;
            board_comp.mine_count = board.mine_count;
            board_comp.cells = board.cells.clone();
            board_comp.revealed = board.revealed.clone();
            board_comp.flagged = board.flagged.clone();
            board_comp.game_over = board.game_over;
            board_comp.game_won = board.game_won;
        }
    }
    
    Ok(())
}

/// 勝利条件のチェック
fn check_win_condition(
    _entity_manager: &mut EntityManager,
    resources: &mut HashMap<&'static str, Rc<RefCell<dyn std::any::Any>>>,
    board: &BoardResource,
) -> Result<(), JsValue> {
    // まだゲーム中で、勝利条件が満たされていなければチェック
    if !board.game_over && !board.game_won {
        // ゲームがスタートしていれば処理を続行
        if board.game_started {
            // 全ての安全なセルが公開されたかチェック
            let mut all_safe_revealed = true;
            
            for y in 0..board.height {
                for x in 0..board.width {
                    let idx = y * board.width + x;
                    
                    // 地雷でなく、まだ公開されていないセルがあれば勝利条件未達成
                    if board.cells[idx] != -1 && !board.revealed[idx] {
                        all_safe_revealed = false;
                        break;
                    }
                }
                
                if !all_safe_revealed {
                    break;
                }
            }
            
            // 勝利条件が満たされた場合
            if all_safe_revealed {
                // ボードリソースを勝利状態に更新
                if let Some(board_rc) = resources.get("board") {
                    if let Some(mut board_res) = board_rc.borrow_mut().downcast_mut::<BoardResource>() {
                        board_res.game_won = true;
                        board_res.is_updated = true;
                        
                        // 全ての地雷にフラグを立てる
                        for y in 0..board_res.height {
                            for x in 0..board_res.width {
                                let idx = y * board_res.width + x;
                                
                                if board_res.cells[idx] == -1 {
                                    board_res.flagged[idx] = true;
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    
    Ok(())
} 