/**
 * 入力システム
 * 
 * ユーザー入力の処理を担当するシステム
 */
use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
use wasm_bindgen::JsValue;
use web_sys::{MouseEvent, KeyboardEvent};

use crate::entities::EntityManager;
use crate::systems::system_registry::DeltaTime;
use crate::components::{Position, MouseState};
use crate::resources::{InputResource, BoardResource};

/// 入力システム - マウスとキーボードの入力を処理
pub fn input_system(
    entity_manager: &mut EntityManager,
    resources: &mut HashMap<&'static str, Rc<RefCell<dyn std::any::Any>>>,
    _delta_time: DeltaTime,
) -> Result<(), JsValue> {
    // InputResourceを取得
    let input_resource = resources.get("input").and_then(|res| {
        res.clone().borrow_mut().downcast_mut::<InputResource>().map(|r| r.clone())
    });
    
    if let Some(input) = input_resource {
        // マウス状態の処理
        if let Some(mouse_event) = input.last_mouse_event.clone() {
            handle_mouse_event(entity_manager, resources, mouse_event)?;
        }
        
        // キーボード状態の処理
        if let Some(key_event) = input.last_key_event.clone() {
            handle_keyboard_event(entity_manager, resources, key_event)?;
        }
    }
    
    Ok(())
}

/// マウスイベントの処理
fn handle_mouse_event(
    entity_manager: &mut EntityManager,
    resources: &mut HashMap<&'static str, Rc<RefCell<dyn std::any::Any>>>,
    event: MouseEvent,
) -> Result<(), JsValue> {
    // マウス座標を取得
    let x = event.offset_x() as f64;
    let y = event.offset_y() as f64;
    
    // マウス状態エンティティを見つける
    let mouse_entities = entity_manager.find_entities_with_component::<MouseState>();
    if let Some(mouse_entity) = mouse_entities.first() {
        // 座標コンポーネントを更新
        if let Some(mut position) = entity_manager.get_component_mut::<Position>(*mouse_entity) {
            position.x = x;
            position.y = y;
        }
        
        // マウス状態コンポーネントを更新
        if let Some(mut mouse_state) = entity_manager.get_component_mut::<MouseState>(*mouse_entity) {
            match event.type_().as_str() {
                "mousedown" => {
                    mouse_state.is_pressed = true;
                    mouse_state.button = event.button();
                    
                    // ボードリソースの取得とセル処理
                    if let Some(board_rc) = resources.get("board") {
                        if let Some(mut board) = board_rc.borrow_mut().downcast_mut::<BoardResource>() {
                            let cell_x = (x / board.cell_size) as usize;
                            let cell_y = (y / board.cell_size) as usize;
                            
                            // 左クリックでセルを開く、右クリックでフラグを立てる
                            if event.button() == 0 {  // 左ボタン
                                // セルを開く
                                board.reveal_cell(cell_x, cell_y);
                            } else if event.button() == 2 {  // 右ボタン
                                // フラグを立てる/下げる
                                board.toggle_flag(cell_x, cell_y);
                            }
                        }
                    }
                },
                "mouseup" => {
                    mouse_state.is_pressed = false;
                },
                "mousemove" => {
                    // 移動中に特別な処理が必要なら追加
                },
                _ => {}
            }
        }
    }
    
    Ok(())
}

/// キーボードイベントの処理
fn handle_keyboard_event(
    _entity_manager: &mut EntityManager,
    resources: &mut HashMap<&'static str, Rc<RefCell<dyn std::any::Any>>>,
    event: KeyboardEvent,
) -> Result<(), JsValue> {
    // キー入力に基づいた処理を実装
    match event.key().as_str() {
        "r" => {
            // Rキーでゲームリセット
            if let Some(board_rc) = resources.get("board") {
                if let Some(mut board) = board_rc.borrow_mut().downcast_mut::<BoardResource>() {
                    board.initialize();
                }
            }
        },
        // 他のキー入力処理を追加
        _ => {}
    }
    
    Ok(())
} 