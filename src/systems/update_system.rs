/**
 * 更新システム
 * 
 * ゲーム状態の更新を担当するシステム
 */
use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
use wasm_bindgen::JsValue;
use js_sys::Date;

use crate::entities::EntityManager;
use crate::systems::system_registry::DeltaTime;
use crate::resources::{GameStateResource, TimerResource};

/// 更新システム - ゲーム状態の更新を担当
pub fn update_system(
    entity_manager: &mut EntityManager,
    resources: &mut HashMap<&'static str, Rc<RefCell<dyn std::any::Any>>>,
    delta_time: DeltaTime,
) -> Result<(), JsValue> {
    // ゲーム状態リソースを取得
    let game_state = resources.get("game_state").and_then(|res| {
        res.clone().borrow_mut().downcast_mut::<GameStateResource>().map(|r| r.clone())
    });
    
    if let Some(state) = game_state {
        match state.current_state.as_str() {
            "loading" => {
                update_loading_state(entity_manager, resources)?;
            },
            "menu" => {
                update_menu_state(entity_manager, resources)?;
            },
            "game" => {
                update_game_state(entity_manager, resources, delta_time)?;
            },
            "game_over" => {
                update_game_over_state(entity_manager, resources)?;
            },
            _ => {}
        }
    }
    
    // タイマーの更新
    update_timers(resources, delta_time)?;
    
    Ok(())
}

/// ロード状態の更新
fn update_loading_state(
    _entity_manager: &mut EntityManager,
    resources: &mut HashMap<&'static str, Rc<RefCell<dyn std::any::Any>>>,
) -> Result<(), JsValue> {
    // アセットの読み込みが完了したかチェック
    let all_assets_loaded = true; // 実際のアセット読み込み状態に基づいて判定
    
    if all_assets_loaded {
        // 読み込み完了したらメニューに遷移
        if let Some(game_state_rc) = resources.get("game_state") {
            if let Some(mut game_state) = game_state_rc.borrow_mut().downcast_mut::<GameStateResource>() {
                game_state.current_state = "menu".to_string();
            }
        }
    }
    
    Ok(())
}

/// メニュー状態の更新
fn update_menu_state(
    _entity_manager: &mut EntityManager,
    _resources: &mut HashMap<&'static str, Rc<RefCell<dyn std::any::Any>>>,
) -> Result<(), JsValue> {
    // メニュー状態の更新処理
    // ボタンのハイライトなどの処理があればここで
    
    Ok(())
}

/// ゲーム状態の更新
fn update_game_state(
    _entity_manager: &mut EntityManager,
    resources: &mut HashMap<&'static str, Rc<RefCell<dyn std::any::Any>>>,
    _delta_time: DeltaTime,
) -> Result<(), JsValue> {
    // ボードリソースを取得してゲームの状態をチェック
    let board_resource = resources.get("board").and_then(|res| {
        res.clone().borrow_mut().downcast_mut::<crate::resources::BoardResource>().map(|r| r.clone())
    });
    
    if let Some(board) = board_resource {
        // ゲームオーバーまたは勝利状態になっていればゲーム状態を変更
        if board.game_over || board.game_won {
            if let Some(game_state_rc) = resources.get("game_state") {
                if let Some(mut game_state) = game_state_rc.borrow_mut().downcast_mut::<GameStateResource>() {
                    game_state.current_state = "game_over".to_string();
                    game_state.is_victory = board.game_won;
                }
            }
        }
    }
    
    Ok(())
}

/// ゲームオーバー状態の更新
fn update_game_over_state(
    _entity_manager: &mut EntityManager,
    resources: &mut HashMap<&'static str, Rc<RefCell<dyn std::any::Any>>>,
) -> Result<(), JsValue> {
    // ゲームオーバー時の状態更新
    // リトライタイマーなどを更新
    
    // リトライタイマーを取得
    let retry_timer = resources.get("retry_timer").and_then(|res| {
        res.clone().borrow_mut().downcast_mut::<TimerResource>().map(|r| r.clone())
    });
    
    if let Some(timer) = retry_timer {
        if timer.is_completed {
            // タイマー完了したらメニューに戻る
            if let Some(game_state_rc) = resources.get("game_state") {
                if let Some(mut game_state) = game_state_rc.borrow_mut().downcast_mut::<GameStateResource>() {
                    game_state.current_state = "menu".to_string();
                }
            }
            
            // タイマーをリセット
            if let Some(timer_rc) = resources.get("retry_timer") {
                if let Some(mut timer) = timer_rc.borrow_mut().downcast_mut::<TimerResource>() {
                    timer.is_completed = false;
                    timer.current_time = 0.0;
                }
            }
        }
    }
    
    Ok(())
}

/// タイマーの更新
fn update_timers(
    resources: &mut HashMap<&'static str, Rc<RefCell<dyn std::any::Any>>>,
    delta_time: DeltaTime,
) -> Result<(), JsValue> {
    // 登録されているすべてのタイマーを更新
    
    // ゲームタイマー
    if let Some(timer_rc) = resources.get("game_timer") {
        if let Some(mut timer) = timer_rc.borrow_mut().downcast_mut::<TimerResource>() {
            if timer.is_running && !timer.is_completed {
                timer.current_time += delta_time.0;
                
                if timer.duration > 0.0 && timer.current_time >= timer.duration {
                    timer.is_completed = true;
                    timer.is_running = false;
                }
            }
        }
    }
    
    // リトライタイマー
    if let Some(timer_rc) = resources.get("retry_timer") {
        if let Some(mut timer) = timer_rc.borrow_mut().downcast_mut::<TimerResource>() {
            if timer.is_running && !timer.is_completed {
                timer.current_time += delta_time.0;
                
                if timer.duration > 0.0 && timer.current_time >= timer.duration {
                    timer.is_completed = true;
                    timer.is_running = false;
                }
            }
        }
    }
    
    // その他のタイマーがあれば同様に更新
    
    Ok(())
}

/// 現在時刻を取得（ミリ秒）
pub fn get_current_time() -> f64 {
    Date::now()
} 