/**
 * フラグトグルシステム
 * 
 * セルの右クリックでフラグの表示・非表示を切り替える
 */
use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
use std::any::Any;
use wasm_bindgen::JsValue;
use wasm_bindgen::prelude::*;
use std::fmt::{self, Debug};

use crate::entities::EntityManager;
use crate::systems::optimized::system_scheduler::DeltaTime;
use crate::resources::{
    BoardConfig,
    BoardResource,
    PlayerStateResource,
    MouseState,
    TimeResource,
    Resource
};
use crate::components::board_components::{CellStateComponent, CellState, CellContentComponent};
use crate::ecs::system::{System, SystemResult};
use crate::resources::ResourceManager;
use crate::resources::board_state::{BoardResource as BoardResourceState, CellState as CellStateState};
use crate::resources::mouse_state::MouseState as MouseStateState;
use crate::resources::player_state::PlayerStateResource as PlayerStateResourceState;

/// フラグトグルシステム - セルのフラグを切り替える処理を担当
pub fn flag_toggle_system(
    resources: &mut ResourceManager,
    _delta_time: DeltaTime
) -> Result<(), JsValue> {
    // フラグトグルロジックをここに実装
    Ok(())
}

/// 指定されたセルのフラグ状態を切り替える
/// 返値: フラグ状態が変更されたかどうか
pub fn toggle_flag(
    row: usize,
    col: usize,
    entity_manager: &mut EntityManager,
    board_state: &mut BoardResourceState,
) -> Result<bool, JsValue> {
    // セルが範囲内かチェック
    if row >= board_state.config.height || col >= board_state.config.width {
        return Ok(false);
    }
    
    // セルのインデックスを計算
    let index = row * board_state.config.width + col;
    if index >= board_state.cells.len() {
        return Ok(false);
    }
    
    // すでに公開済みの場合は何もしない
    if board_state.cells[index].state == CellStateState::Revealed {
        return Ok(false);
    }
    
    // フラグ状態を切り替え
    let cell = &mut board_state.cells[index];
    cell.toggle_flag();
    
    Ok(true)
}

/**
 * フラグトグルシステム
 * 
 * セルの右クリックでフラグの表示・非表示を切り替える
 */
#[derive(Debug)]
pub struct FlagToggleSystem {
    // 必要なステート
    active: bool,
}

impl FlagToggleSystem {
    pub fn new() -> Self {
        Self {
            // 初期化
            active: true,
        }
    }
}

impl System for FlagToggleSystem {
    fn update(&mut self, entity_manager: &mut EntityManager, resources: &mut ResourceManager) -> SystemResult {
        // プレイヤーの状態とボードの状態を取得
        let player_state_rc = match resources.get::<PlayerStateResourceState>() {
            Ok(state) => state,
            Err(_) => return SystemResult::Ok, // プレイヤー状態がなければ何もしない
        };
        
        // PlayerStateResourceをダウンキャスト
        let player_binding = player_state_rc.borrow();
        let player_state = match player_binding.downcast_ref::<PlayerStateResourceState>() {
            Some(state) => state,
            None => return SystemResult::Ok, // 型変換に失敗したら何もしない
        };
        
        // マウスの右ボタンが押されていない場合は何もしない
        if player_state.mouse_state.get_state() != MouseState::RIGHT_DOWN {
            return SystemResult::Ok;
        }
        
        // ボードの状態を取得
        let board_state_rc = match resources.get::<BoardResourceState>() {
            Ok(state) => state,
            Err(_) => return SystemResult::Ok, // ボード状態がなければ何もしない
        };
        
        // BoardResourceをダウンキャスト
        let board_binding = board_state_rc.borrow();
        let board_state = match board_binding.downcast_ref::<BoardResourceState>() {
            Some(state) => state,
            None => return SystemResult::Ok, // 型変換に失敗したら何もしない
        };
        
        // マウス座標からセルの位置を計算
        let cell_size = board_state.config.cell_size as f64; 
        let col = (player_state.mouse_state.x as f64 / cell_size) as usize;
        let row = (player_state.mouse_state.y as f64 / cell_size) as usize;
        
        // 範囲外のクリックは無視
        if row >= board_state.config.height || col >= board_state.config.width {
            return SystemResult::Ok;
        }
        
        // 可変の参照を取得するために再度リソースをmutで取得
        let board_state_rc_mut = match resources.get_mut::<BoardResourceState>() {
            Ok(state) => state,
            Err(_) => return SystemResult::Ok,
        };
        
        // ダウンキャスト（可変参照）
        let mut board_binding_mut = board_state_rc_mut.borrow_mut();
        if let Some(mut board_state_mut) = board_binding_mut.downcast_mut::<BoardResourceState>() {
            // セルのインデックスを計算
            let index = row * board_state_mut.config.width + col;
            
            // セルが範囲内かチェック
            if index < board_state_mut.cells.len() {
                // すでに開かれたセルは変更しない
                if board_state_mut.cells[index].state != CellStateState::Revealed {
                    // フラグを切り替え
                    board_state_mut.cells[index].toggle_flag();
                }
            }
        }
        
        SystemResult::Ok
    }
}

// system_trait::Systemの実装を追加
impl crate::systems::optimized::system_trait::System for FlagToggleSystem {
    fn name(&self) -> &str {
        "FlagToggleSystem"
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