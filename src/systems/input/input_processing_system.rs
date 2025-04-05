/**
 * 入力処理システム
 * 
 * 入力状態を更新し、ゲーム内での入力処理を行う
 */
use std::rc::Rc;
use std::cell::RefCell;
use std::any::Any;

use crate::system::System;
use crate::system::system_registry::{SystemPhase, SystemId};
use crate::resources::{ResourceManager, InputResource, PlayerStateResource, MouseState, EventQueueResource, GameEvent};

/// 入力処理システム
pub struct InputProcessingSystem {
    /// システム名
    name: String,
    /// InputCollectionSystemのID（依存関係用）
    input_collection_id: Option<SystemId>,
}

impl InputProcessingSystem {
    /// 新しい入力処理システムを作成
    pub fn new() -> Self {
        Self {
            name: "InputProcessingSystem".to_string(),
            input_collection_id: None,
        }
    }
    
    /// 依存するシステムのIDを設定
    pub fn set_dependency(&mut self, input_collection_id: SystemId) {
        self.input_collection_id = Some(input_collection_id);
    }
}

impl System for InputProcessingSystem {
    fn name(&self) -> &str {
        &self.name
    }
    
    fn phase(&self) -> SystemPhase {
        SystemPhase::Input
    }
    
    fn dependencies(&self) -> Vec<SystemId> {
        match self.input_collection_id {
            Some(id) => vec![id],
            None => Vec::new(),
        }
    }
    
    fn run(&mut self, resources: &mut ResourceManager) {
        // InputResourceとPlayerStateResourceを取得
        let input_rc = match resources.get::<InputResource>() {
            Ok(rc) => rc,
            Err(_) => return,
        };
        
        let player_state_rc = match resources.get_mut::<PlayerStateResource>() {
            Ok(rc) => rc,
            Err(_) => return,
        };
        
        let event_queue_rc = match resources.get_mut::<EventQueueResource>() {
            Ok(rc) => rc,
            Err(_) => return,
        };
        
        // マウス位置をPlayerStateResourceに反映
        {
            let input_ref = input_rc.borrow();
            let input = input_ref.downcast_ref::<InputResource>().unwrap();
            
            let mut player_state_ref = player_state_rc.borrow_mut();
            let player_state = player_state_ref.downcast_mut::<PlayerStateResource>().unwrap();
            
            let mut event_queue_ref = event_queue_rc.borrow_mut();
            let event_queue = event_queue_ref.downcast_mut::<EventQueueResource>().unwrap();
            
            // マウス座標の更新
            let (x, y) = input.get_mouse_position();
            player_state.update_player_position(x as f64, y as f64);
            
            // マウスボタン状態の更新と対応するイベント発行
            if input.is_mouse_down(0) {
                player_state.set_mouse_state(MouseState::LeftDown);
                
                // マウス位置からセル座標を計算（仮の実装）
                // 実際にはボードのサイズやセルサイズを考慮する必要がある
                let cell_x = (x as f64 / 30.0).floor() as usize; // 仮のセルサイズ30
                let cell_y = (y as f64 / 30.0).floor() as usize;
                
                // 左クリックの場合はセル公開イベントを発行
                if input.is_mouse_pressed(0) {
                    event_queue.push_event(GameEvent::CellReveal(cell_y, cell_x));
                }
            } else if input.is_mouse_down(2) {
                player_state.set_mouse_state(MouseState::RightDown);
                
                // マウス位置からセル座標を計算
                let cell_x = (x as f64 / 30.0).floor() as usize;
                let cell_y = (y as f64 / 30.0).floor() as usize;
                
                // 右クリックの場合はフラグトグルイベントを発行
                if input.is_mouse_pressed(2) {
                    event_queue.push_event(GameEvent::FlagToggle(cell_y, cell_x));
                }
            } else if input.is_mouse_down(1) {
                player_state.set_mouse_state(MouseState::MiddleDown);
            } else {
                player_state.set_mouse_state(MouseState::Up);
            }
            
            // マウス移動イベントの発行
            event_queue.push_event(GameEvent::MouseMove(x as f64, y as f64));
            
            // キーボードの特殊入力イベント処理
            if input.is_key_pressed("KeyR") {
                // Rキーでゲームリセット
                event_queue.push_event(GameEvent::GameReset);
            }
            
            if input.is_key_pressed("Escape") || input.is_key_pressed("KeyP") {
                // ESCキーまたはPキーで一時停止/再開
                event_queue.push_event(GameEvent::PauseToggle);
            }
        }
    }
} 