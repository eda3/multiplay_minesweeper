/**
 * 勝利条件確認システム
 * 
 * ゲームの勝利条件（すべての安全なセルが公開された状態）を
 * チェックするシステム
 */
use crate::components::board_components::{CellStateComponent, CellState, CellContentComponent};
use crate::entities::{EntityManager, EntityId};
use crate::resources::{
    BoardConfig,
    BoardResource,
    CoreGameResource,
    GamePhase,
    Resource
};
use crate::models::CellValue;
use crate::ecs::system::{System, SystemResult};
use crate::resources::ResourceManager;
use wasm_bindgen::prelude::*;
use crate::systems::optimized::system_scheduler::DeltaTime;
use std::fmt::{self, Debug};
use std::rc::Rc;
use std::cell::RefCell;
use std::any::Any;

/// 勝利条件チェックシステム
/// ゲームの勝利条件をチェックする
#[derive(Debug)]
pub struct WinConditionSystem {
    // 必要なステート
    active: bool,
}

impl WinConditionSystem {
    pub fn new() -> Self {
        Self {
            // 初期化
            active: true,
        }
    }
}

impl System for WinConditionSystem {
    fn update(&mut self, entity_manager: &mut EntityManager, resources: &mut ResourceManager) -> SystemResult {
        // ボードの状態を取得
        let board_state = match resources.get::<BoardResource>() {
            Ok(state) => state,
            Err(_) => return SystemResult::Ok, // ボード状態がなければ何もしない
        };
        
        // RcのRefCellからBoardResourceを取得
        let binding = board_state.borrow();
        let board_state_ref = match binding.downcast_ref::<BoardResource>() {
            Some(state) => state,
            None => return SystemResult::Ok, // 型変換に失敗したら何もしない
        };
        
        // 勝利条件をチェック - BoardResourceのメソッドを使用
        let is_win = board_state_ref.check_win_condition();
        
        // 勝利条件を満たしたら
        if is_win {
            // CoreGameResourceを取得して勝利状態に設定
            if let Ok(core_game) = resources.get_mut::<CoreGameResource>() {
                // 可変参照を取得してダウンキャスト
                if let Some(mut core_game) = core_game.borrow_mut().downcast_mut::<CoreGameResource>() {
                    // 勝利フラグを設定
                    core_game.set_game_over(true);
                }
            }
            
            // すべての地雷にフラグを立てる - BoardResourceのメソッドを使用
            if let Ok(board_state_mut) = resources.get_mut::<BoardResource>() {
                let mut binding_mut = board_state_mut.borrow_mut();
                if let Some(mut board) = binding_mut.downcast_mut::<BoardResource>() {
                    // BoardResourceのreveal_all_minesメソッドを使用
                    board.reveal_all_mines();
                }
            }
        }
        
        SystemResult::Ok
    }
}

/// 勝利条件システムの関数バージョン
/// システムレジストリから直接呼び出すための関数
pub fn win_condition_system(
    resources: &mut ResourceManager,
    _delta_time: DeltaTime,
    score: u32,
    elapsed_time: f64
) -> Result<(), JsValue> {
    // 勝利条件チェックロジックをここに実装
    // BoardResourceの状態を取得
    if let Ok(board_state) = resources.get::<BoardResource>() {
        if let Some(board) = board_state.borrow().downcast_ref::<BoardResource>() {
            // 勝利条件をチェック
            let is_win = board.check_win_condition();
            
            // 勝利なら、コアゲームを更新
            if is_win {
                if let Ok(core_game) = resources.get_mut::<CoreGameResource>() {
                    if let Some(mut game) = core_game.borrow_mut().downcast_mut::<CoreGameResource>() {
                        game.set_game_over(true);
                        // スコアと時間を設定
                        game.add_score(score);
                        // TODO: 経過時間の設定
                    }
                }
                
                // すべての地雷にフラグを立てる
                if let Ok(board_state_mut) = resources.get_mut::<BoardResource>() {
                    if let Some(mut board_mut) = board_state_mut.borrow_mut().downcast_mut::<BoardResource>() {
                        board_mut.reveal_all_mines();
                    }
                }
            }
        }
    }
    
    Ok(())
}

// system_trait::Systemの実装を追加
impl crate::systems::optimized::system_trait::System for WinConditionSystem {
    fn name(&self) -> &str {
        "WinConditionSystem"
    }
    
    fn update(&mut self, entity_manager: &mut EntityManager, delta_time: f32) {
        // ecs::system::Systemのupdateを再利用
        let dummy_resources = &mut ResourceManager::new();
        let _ = System::update(self, entity_manager, dummy_resources);
    }
    
    fn is_active(&self) -> bool {
        self.active
    }
    
    fn set_active(&mut self, active: bool) {
        self.active = active;
    }
    
    fn priority(&self) -> i32 {
        0 // デフォルトの優先度
    }
} 