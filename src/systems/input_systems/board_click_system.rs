/**
 * ボードクリックシステム
 * 
 * マウスクリックを検知してセルのクリックイベントを発行するシステム
 */
use crate::resources::{ResourceManager, EventBusResource};
use crate::resources::mouse_state::MouseState;
use crate::resources::board_resources::BoardConfigResource;
use crate::events::input_events::MouseClickEvent;
use crate::events::board_events::CellRevealedEvent;
use crate::events::board_events::FlagPlacedEvent;
use crate::models::coordinate::Coordinate;
use crate::models::cell::CellValue;
use crate::systems::{EventSystemTrait, EventSystem};
use crate::system::system_registry::SystemPhase;
use crate::ecs::system::System;

/// ボードクリックシステム
pub struct BoardClickSystem {
    /// システム名
    name: String,
    /// イベントシステム
    event_system: EventSystem,
    /// セルサイズ（ピクセル単位）
    cell_size: u32,
}

impl BoardClickSystem {
    /// 新しいボードクリックシステムを作成
    pub fn new() -> Self {
        Self {
            name: "BoardClickSystem".to_string(),
            event_system: EventSystem::new(),
            cell_size: 32, // デフォルト値
        }
    }
    
    /// セルのインデックスを計算
    fn get_cell_index(&self, x: f64, y: f64, board_config: &BoardConfigResource) -> Option<(usize, usize)> {
        let col = (x / self.cell_size as f64) as usize;
        let row = (y / self.cell_size as f64) as usize;
        
        if row < board_config.height && col < board_config.width {
            Some((row, col))
        } else {
            None
        }
    }
}

impl Default for BoardClickSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl System for BoardClickSystem {
    fn name(&self) -> &str {
        &self.name
    }
    
    fn update(&mut self, _entity_manager: &mut crate::entities::EntityManager, resources: &mut ResourceManager) -> crate::ecs::system::SystemResult {
        // マウス状態リソースの取得
        let mouse_state_rc = match resources.get::<MouseState>() {
            Ok(rc) => rc,
            Err(_) => return crate::ecs::system::SystemResult::Error,
        };
        
        let mouse_state = mouse_state_rc.borrow();
        let mouse_state = match mouse_state.downcast_ref::<MouseState>() {
            Some(state) => state,
            None => return crate::ecs::system::SystemResult::Error,
        };
        
        // ボード設定リソースの取得
        let board_config_rc = match resources.get::<BoardConfigResource>() {
            Ok(rc) => rc,
            Err(_) => return crate::ecs::system::SystemResult::Error,
        };
        
        let board_config = board_config_rc.borrow();
        let board_config = match board_config.downcast_ref::<BoardConfigResource>() {
            Some(config) => config,
            None => return crate::ecs::system::SystemResult::Error,
        };
        
        // クリックがあったかどうかをチェック
        if mouse_state.clicked {
            // マウス座標を取得
            let (x, y) = (mouse_state.x, mouse_state.y);
            
            // クリック位置からセルのインデックスを計算
            if let Some((row, col)) = self.get_cell_index(x, y, board_config) {
                // MouseClickイベントを発行
                let click_event = MouseClickEvent {
                    x,
                    y,
                    is_right_click: mouse_state.right_button,
                    is_middle_click: mouse_state.middle_button,
                    is_double_click: false, // double clickは未実装
                };
                
                self.publish_event(click_event, resources);
                
                // 右クリックかどうかに応じて、セル公開またはフラグ設置イベントを発行
                let coord = Coordinate::new(row as u32, col as u32);
                
                if mouse_state.right_button {
                    // 右クリック時はフラグ設置イベントを発行
                    let flag_event = FlagPlacedEvent {
                        coord,
                        is_placed: true, // トグルは別のシステムで処理
                        remaining_flags: board_config.mine_count as u32, // 正確な値は別のシステムで計算
                    };
                    
                    self.publish_event(flag_event, resources);
                } else {
                    // 左クリック時はセル公開イベントを発行
                    let reveal_event = CellRevealedEvent {
                        coord,
                        value: CellValue::Unknown, // 実際の値は別のシステムで設定
                        is_chain: false,
                    };
                    
                    self.publish_event(reveal_event, resources);
                }
            }
        }
        
        crate::ecs::system::SystemResult::Ok
    }
}

impl EventSystemTrait for BoardClickSystem {
    fn get_handler_ids(&self) -> &std::sync::Arc<std::sync::Mutex<std::collections::HashMap<String, u64>>> {
        self.event_system.get_handler_ids()
    }
    
    fn get_handler_ids_mut(&mut self) -> &mut std::sync::Arc<std::sync::Mutex<std::collections::HashMap<String, u64>>> {
        self.event_system.get_handler_ids_mut()
    }
} 