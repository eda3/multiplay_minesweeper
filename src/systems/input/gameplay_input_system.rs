/**
 * ゲームプレイ入力システム
 * 
 * ゲームプレイに関する入力を処理する
 */
use std::rc::Rc;
use std::cell::RefCell;
use std::any::Any;

use crate::system::System;
use crate::system::system_registry::{SystemPhase, SystemId};
use crate::resources::{
    ResourceManager, 
    PlayerStateResource, 
    BoardResource, 
    EventQueueResource, 
    GameEvent,
    BoardStateResource
};

/// ゲームプレイ入力システム
pub struct GameplayInputSystem {
    /// システム名
    name: String,
    /// 依存するシステムのID
    ui_input_id: Option<SystemId>,
}

impl GameplayInputSystem {
    /// 新しいゲームプレイ入力システムを作成
    pub fn new() -> Self {
        Self {
            name: "GameplayInputSystem".to_string(),
            ui_input_id: None,
        }
    }
    
    /// 依存するシステムのIDを設定
    pub fn set_dependency(&mut self, ui_input_id: SystemId) {
        self.ui_input_id = Some(ui_input_id);
    }
    
    /// マウス座標からボード上のセル位置を計算
    fn calculate_cell_position(&self, x: f64, y: f64, board: &BoardResource) -> Option<(usize, usize)> {
        // ボードの位置とサイズ情報を使ってセル位置を計算
        let board_x = 0.0; // ボードのX座標
        let board_y = 50.0; // ボードのY座標（UI部分の下に配置されていると仮定）
        let cell_size = 30.0; // セルのサイズ
        
        // ボード領域外ならNoneを返す
        if x < board_x || y < board_y {
            return None;
        }
        
        // セル座標を計算
        let cell_x = ((x - board_x) / cell_size) as usize;
        let cell_y = ((y - board_y) / cell_size) as usize;
        
        // ボードの範囲内かチェック
        if cell_x < board.width && cell_y < board.height {
            Some((cell_x, cell_y))
        } else {
            None
        }
    }
}

impl System for GameplayInputSystem {
    fn name(&self) -> &str {
        &self.name
    }
    
    fn phase(&self) -> SystemPhase {
        SystemPhase::Input
    }
    
    fn dependencies(&self) -> Vec<SystemId> {
        match self.ui_input_id {
            Some(id) => vec![id],
            None => Vec::new(),
        }
    }
    
    fn run(&mut self, resources: &mut ResourceManager) {
        // 必要なリソースの取得
        let player_state_rc = match resources.get::<PlayerStateResource>() {
            Ok(rc) => rc,
            Err(_) => return,
        };
        
        let board_rc = match resources.get::<BoardResource>() {
            Ok(rc) => rc,
            Err(_) => return,
        };
        
        let board_state_rc = match resources.get::<BoardStateResource>() {
            Ok(rc) => rc,
            Err(_) => return,
        };
        
        let event_queue_rc = match resources.get_mut::<EventQueueResource>() {
            Ok(rc) => rc,
            Err(_) => return,
        };
        
        // ゲームプレイ入力の処理
        {
            let player_state_ref = player_state_rc.borrow();
            let player_state = player_state_ref.downcast_ref::<PlayerStateResource>().unwrap();
            
            let board_ref = board_rc.borrow();
            let board = board_ref.downcast_ref::<BoardResource>().unwrap();
            
            let board_state_ref = board_state_rc.borrow();
            let board_state = board_state_ref.downcast_ref::<BoardStateResource>().unwrap();
            
            let mut event_queue_ref = event_queue_rc.borrow_mut();
            let event_queue = event_queue_ref.downcast_mut::<EventQueueResource>().unwrap();
            
            // ゲームが進行中かどうかをチェック
            let is_game_active = !board_state.is_game_over && !board_state.is_win;
            
            if !is_game_active {
                return;
            }
            
            // マウス位置の取得
            let mouse_x = player_state.mouse_state.x;
            let mouse_y = player_state.mouse_state.y;
            
            // マウスボタンの状態を取得
            let is_left_clicked = player_state.mouse_state.clicked;
            let is_right_clicked = player_state.mouse_state.right_clicked;
            
            // セル位置の計算
            if let Some((cell_x, cell_y)) = self.calculate_cell_position(mouse_x, mouse_y, board) {
                // 左クリックでセルを開く
                if is_left_clicked {
                    event_queue.push_event(GameEvent::CellReveal(cell_x, cell_y));
                }
                
                // 右クリックでフラグをトグル
                if is_right_clicked {
                    event_queue.push_event(GameEvent::FlagToggle(cell_x, cell_y));
                }
                
                // マウス移動イベントも発行（ホバー効果などに利用可能）
                event_queue.push_event(GameEvent::MouseMove(mouse_x, mouse_y));
            }
        }
    }
} 