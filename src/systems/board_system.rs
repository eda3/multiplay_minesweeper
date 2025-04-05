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
use crate::resources::board_state::BoardResource;
use crate::resources::board_state::{CellState, Cell};
use crate::board::Board;
use crate::models::cell::CellValue;

// 新しいフラグを追加（実際にはBoardResourceに追加するべき）
thread_local! {
    static BOARD_UPDATED: RefCell<bool> = RefCell::new(false);
    static GAME_OVER: RefCell<bool> = RefCell::new(false);
    static GAME_WON: RefCell<bool> = RefCell::new(false);
}

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
        // ボードが更新されたか確認
        let is_updated = BOARD_UPDATED.with(|updated| *updated.borrow());
        
        // ボードが更新された場合、各エンティティの状態を更新
        if is_updated {
            update_board_entities(entity_manager, &board)?;
            
            // 更新フラグをリセット
            BOARD_UPDATED.with(|updated| *updated.borrow_mut() = false);
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
            board_comp.width = board.config.width;
            board_comp.height = board.config.height;
            board_comp.mine_count = board.config.mine_count;
            
            // CellValueの配列を作成
            let mut cells = Vec::with_capacity(board.cells.len());
            
            // BoardResourceの各セル情報をBoardのCellValue型に変換
            for cell in &board.cells {
                // Cell構造体からCellValue enumへ変換
                let cell_value = if cell.is_mine {
                    CellValue::Mine
                } else {
                    CellValue::Empty(cell.adjacent_mines)
                };
                cells.push(cell_value);
            }
            board_comp.cells = cells;
            
            // セルの状態をbool配列に変換
            let mut revealed = vec![false; board.cells.len()];
            let mut flagged = vec![false; board.cells.len()];
            
            // 各セルの状態からrevealed/flagged配列を作成
            for (i, cell) in board.cells.iter().enumerate() {
                // Cell型の状態チェックメソッドを使用
                revealed[i] = cell.is_revealed(); // is_revealed()メソッドはCell型にある
                flagged[i] = cell.is_flagged();   // is_flagged()メソッドもCell型にある
            }
            
            board_comp.revealed = revealed;
            board_comp.flagged = flagged;
            
            // ゲーム状態を取得
            let game_over = GAME_OVER.with(|over| *over.borrow());
            let game_won = GAME_WON.with(|won| *won.borrow());
            
            board_comp.game_over = game_over;
            board_comp.win = game_won;
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
    // 現在のゲーム状態を取得
    let game_over = GAME_OVER.with(|over| *over.borrow());
    let game_won = GAME_WON.with(|won| *won.borrow());
    
    // まだゲーム中で、勝利条件が満たされていなければチェック
    if !game_over && !game_won {
        // ゲームがスタートしていれば処理を続行（最初のクリックが済んでいるかどうか）
        if !board.first_click {
            // 全ての安全なセルが公開されたかチェック
            let all_safe_revealed = board.check_win_condition();
            
            // 勝利条件が満たされた場合
            if all_safe_revealed {
                // ボードリソースを勝利状態に更新
                if let Some(board_rc) = resources.get("board") {
                    if let Some(mut board_res) = board_rc.borrow_mut().downcast_mut::<BoardResource>() {
                        // ゲーム勝利フラグを設定
                        GAME_WON.with(|won| *won.borrow_mut() = true);
                        BOARD_UPDATED.with(|updated| *updated.borrow_mut() = true);
                        
                        // 全ての地雷にフラグを立てる
                        board_res.reveal_all_mines();
                    }
                }
            }
        }
    }
    
    Ok(())
} 