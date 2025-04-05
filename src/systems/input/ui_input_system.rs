/**
 * UI入力システム
 * 
 * UI要素とのインタラクションを処理する
 */
use std::rc::Rc;
use std::cell::RefCell;
use std::any::Any;

use crate::system::System;
use crate::system::system_registry::{SystemPhase, SystemId};
use crate::resources::{ResourceManager, PlayerStateResource, EventQueueResource, GameEvent};

/// UI入力システム
pub struct UIInputSystem {
    /// システム名
    name: String,
    /// 依存するシステムのID
    input_processing_id: Option<SystemId>,
}

impl UIInputSystem {
    /// 新しいUI入力システムを作成
    pub fn new() -> Self {
        Self {
            name: "UIInputSystem".to_string(),
            input_processing_id: None,
        }
    }
    
    /// 依存するシステムのIDを設定
    pub fn set_dependency(&mut self, input_processing_id: SystemId) {
        self.input_processing_id = Some(input_processing_id);
    }
    
    /// UI要素がクリック座標に含まれるかをテスト
    fn hit_test<'a>(&self, x: f64, y: f64, ui_elements: &'a [UIElement]) -> Option<&'a UIElement> {
        for element in ui_elements {
            if element.contains_point(x, y) {
                return Some(element);
            }
        }
        None
    }
}

/// シンプルなUI要素の定義
struct UIElement {
    id: String,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    clickable: bool,
}

impl UIElement {
    /// 指定された点（x, y）がUI要素の範囲内にあるかどうかをチェック
    fn contains_point(&self, x: f64, y: f64) -> bool {
        x >= self.x && x <= self.x + self.width && y >= self.y && y <= self.y + self.height
    }
}

impl System for UIInputSystem {
    fn name(&self) -> &str {
        &self.name
    }
    
    fn phase(&self) -> SystemPhase {
        SystemPhase::Input
    }
    
    fn dependencies(&self) -> Vec<SystemId> {
        match self.input_processing_id {
            Some(id) => vec![id],
            None => Vec::new(),
        }
    }
    
    fn run(&mut self, resources: &mut ResourceManager) {
        // プレイヤーステートリソースの取得
        let player_state_rc = match resources.get::<PlayerStateResource>() {
            Ok(rc) => rc,
            Err(_) => return,
        };
        
        // イベントキューリソースの取得
        let event_queue_rc = match resources.get_mut::<EventQueueResource>() {
            Ok(rc) => rc,
            Err(_) => return,
        };
        
        // UIイベントの処理
        {
            let player_state_ref = player_state_rc.borrow();
            let player_state = player_state_ref.downcast_ref::<PlayerStateResource>().unwrap();
            
            let mut event_queue_ref = event_queue_rc.borrow_mut();
            let event_queue = event_queue_ref.downcast_mut::<EventQueueResource>().unwrap();
            
            // プレイヤーの位置（マウス位置）を取得
            let mouse_x = player_state.mouse_state.x;
            let mouse_y = player_state.mouse_state.y;
            
            // UI要素のリスト（実際のゲームではこれはリソースとして管理される）
            let ui_elements = vec![
                UIElement {
                    id: "reset_button".to_string(),
                    x: 10.0,
                    y: 10.0,
                    width: 100.0,
                    height: 30.0,
                    clickable: true,
                },
                UIElement {
                    id: "settings_button".to_string(),
                    x: 120.0,
                    y: 10.0,
                    width: 100.0,
                    height: 30.0,
                    clickable: true,
                },
            ];
            
            // マウスボタンが押されているかチェック
            let is_left_clicked = player_state.mouse_state.clicked;
            
            if is_left_clicked {
                // ヒットテスト
                if let Some(element) = self.hit_test(mouse_x, mouse_y, &ui_elements) {
                    if element.clickable {
                        // クリック可能な要素がクリックされた場合、UIClickイベントを発行
                        event_queue.push_event(GameEvent::UIClick(element.id.clone()));
                        
                        // 要素に応じた特殊イベントも発行
                        match element.id.as_str() {
                            "reset_button" => {
                                event_queue.push_event(GameEvent::GameReset);
                            },
                            "settings_button" => {
                                event_queue.push_event(GameEvent::SettingChange("open_settings".to_string(), "true".to_string()));
                            },
                            _ => {},
                        }
                    }
                }
            }
        }
    }
} 