/**
 * 勝利条件確認システム
 * 
 * ゲームの勝利条件（すべての安全なセルが公開された状態）を
 * チェックするシステム
 */
use crate::components::board_components::{CellStateComponent, CellState, CellContentComponent};
use crate::entities::{EntityManager, EntityId};
use crate::resources::{
    BoardConfigResource,
    BoardStateResource,
    CoreGameResource,
    GamePhase,
    Resource
};
use crate::models::CellValue;
use crate::ecs::system::{System, SystemResult};
use crate::resources::ResourceManager;

/// 勝利条件チェックシステム
/// ゲームの勝利条件をチェックする
pub struct WinConditionSystem {
    // 必要なステート
}

impl WinConditionSystem {
    pub fn new() -> Self {
        Self {
            // 初期化
        }
    }
}

impl System for WinConditionSystem {
    fn update(&mut self, entity_manager: &mut EntityManager, resources: &mut ResourceManager) -> SystemResult {
        // ボードの状態を取得
        let board_state = match resources.get::<BoardStateResource>() {
            Some(state) => state,
            None => return SystemResult::Ok, // ボード状態がなければ何もしない
        };
        
        // ゲームが終了している場合は何もしない
        if board_state.is_game_over || board_state.is_win {
            return SystemResult::Ok;
        }
        
        // 最初のクリックが行われていない場合は何もしない
        if !board_state.first_click {
            return SystemResult::Ok;
        }
        
        // 残りの安全なセルがなければ勝利
        if board_state.remaining_safe_cells == 0 {
            // 可変参照を取得
            let mut board_state = match resources.get_mut::<BoardStateResource>() {
                Some(state) => state,
                None => return SystemResult::Ok,
            };
            
            // 勝利フラグをセット
            board_state.is_win = true;
            
            // ゲームフェーズを Victory に変更
            if let Some(mut core_game) = resources.get_mut::<CoreGameResource>() {
                core_game.set_phase(GamePhase::GameOver { win: true });
            }
            
            // すべての地雷にフラグを立てる
            Self::flag_all_mines(entity_manager, resources);
        }
        
        SystemResult::Ok
    }
}

impl WinConditionSystem {
    // すべての地雷にフラグを立てる補助メソッド
    fn flag_all_mines(entity_manager: &mut EntityManager, resources: &mut ResourceManager) {
        let board_state = match resources.get::<BoardStateResource>() {
            Some(state) => state,
            None => return,
        };
        
        let board_config = match resources.get::<BoardConfigResource>() {
            Some(config) => config,
            None => return,
        };
        
        // すべてのセルをチェック
        for row in 0..board_config.height {
            for col in 0..board_config.width {
                if let Some(entity_id) = board_state.get_cell_entity(row, col) {
                    // TODO: EntityManagerのget_componentメソッドを実装後に修正
                    // コンテンツが地雷かチェック
                    // let is_mine = match entity_manager.get_component::<CellContentComponent>(entity_id) {
                    //     Some(content) => matches!(content.value, CellValue::Mine),
                    //     None => false,
                    // };
                    
                    // // 地雷ならフラグを立てる（まだフラグが立っていない場合）
                    // if is_mine {
                    //     if let Some(mut state) = entity_manager.get_component_mut::<CellStateComponent>(entity_id) {
                    //         if state.state == CellState::Hidden {
                    //             state.state = CellState::Flagged;
                    //         }
                    //     }
                    // }
                }
            }
        }
    }
} 