/**
 * UIシステム
 * 
 * ユーザーインターフェースの描画と更新を担当するシステム
 */
use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
use wasm_bindgen::JsValue;
use web_sys::CanvasRenderingContext2d;

use crate::entities::EntityManager;
use crate::systems::system_registry::DeltaTime;
use crate::resources::{RenderResource, GameStateResource, TimerResource, UiResource};

/// UIシステム - インターフェースの描画処理
pub fn ui_system(
    entity_manager: &mut EntityManager,
    resources: &mut HashMap<&'static str, Rc<RefCell<dyn std::any::Any>>>,
    _delta_time: DeltaTime,
) -> Result<(), JsValue> {
    // UIリソースを取得
    let ui_resource = resources.get("ui").and_then(|res| {
        res.clone().borrow_mut().downcast_mut::<UiResource>().map(|r| r.clone())
    });
    
    // 描画リソースを取得
    let render_resource = resources.get("render").and_then(|res| {
        res.clone().borrow_mut().downcast_mut::<RenderResource>().map(|r| r.clone())
    });
    
    // ゲーム状態を取得
    let game_state = resources.get("game_state").and_then(|res| {
        res.clone().borrow_mut().downcast_mut::<GameStateResource>().map(|r| r.clone())
    });
    
    if let (Some(ui), Some(render), Some(state)) = (ui_resource, render_resource, game_state) {
        let context = render.context.clone();
        
        // 現在の状態に応じたUI描画
        match state.current_state.as_str() {
            "loading" => {
                draw_loading_screen(&context, &ui, render.canvas_width, render.canvas_height)?;
            },
            "menu" => {
                draw_menu_screen(&context, &ui, render.canvas_width, render.canvas_height)?;
            },
            "game" => {
                draw_game_ui(entity_manager, resources, &context, &ui, render.canvas_width, render.canvas_height)?;
            },
            "game_over" => {
                draw_game_over_screen(resources, &context, &ui, render.canvas_width, render.canvas_height)?;
            },
            _ => {}
        }
    }
    
    Ok(())
}

/// ロード画面の描画
fn draw_loading_screen(
    context: &CanvasRenderingContext2d,
    ui: &UiResource,
    canvas_width: f64,
    canvas_height: f64,
) -> Result<(), JsValue> {
    // ローディングバーの描画
    let bar_width = canvas_width * 0.8;
    let bar_height = 20.0;
    let bar_x = (canvas_width - bar_width) / 2.0;
    let bar_y = canvas_height / 2.0 - bar_height / 2.0;
    
    // ロード進捗（0.0〜1.0）
    let progress = ui.loading_progress;
    
    // 背景バー
    context.set_fill_style(&JsValue::from_str("#333333"));
    context.fill_rect(bar_x, bar_y, bar_width, bar_height);
    
    // 進捗バー
    context.set_fill_style(&JsValue::from_str("#00FF00"));
    context.fill_rect(bar_x, bar_y, bar_width * progress, bar_height);
    
    // テキスト
    context.set_fill_style(&JsValue::from_str("#FFFFFF"));
    context.set_font("20px Arial");
    context.set_text_align("center");
    context.set_text_baseline("top");
    
    context.fill_text(
        "Loading...",
        canvas_width / 2.0,
        bar_y + bar_height + 10.0,
    )?;
    
    Ok(())
}

/// メニュー画面の描画
fn draw_menu_screen(
    context: &CanvasRenderingContext2d,
    ui: &UiResource,
    canvas_width: f64,
    canvas_height: f64,
) -> Result<(), JsValue> {
    // タイトル
    context.set_fill_style(&JsValue::from_str("#FFFFFF"));
    context.set_font("36px Arial");
    context.set_text_align("center");
    context.set_text_baseline("top");
    
    context.fill_text(
        "MultiPlayer Minesweeper",
        canvas_width / 2.0,
        50.0,
    )?;
    
    // 各ボタンの描画
    for (i, button) in ui.menu_buttons.iter().enumerate() {
        let button_width = 200.0;
        let button_height = 50.0;
        let button_x = (canvas_width - button_width) / 2.0;
        let button_y = 150.0 + (i as f64 * 70.0);
        
        // ボタン背景
        let button_color = if button.is_hovered {
            "#555555"
        } else {
            "#333333"
        };
        
        context.set_fill_style(&JsValue::from_str(button_color));
        context.fill_rect(button_x, button_y, button_width, button_height);
        
        // ボタンテキスト
        context.set_fill_style(&JsValue::from_str("#FFFFFF"));
        context.set_font("20px Arial");
        context.set_text_align("center");
        context.set_text_baseline("middle");
        
        context.fill_text(
            &button.text,
            canvas_width / 2.0,
            button_y + button_height / 2.0,
        )?;
    }
    
    // 難易度選択
    context.set_fill_style(&JsValue::from_str("#FFFFFF"));
    context.set_font("18px Arial");
    context.set_text_align("center");
    context.set_text_baseline("middle");
    
    context.fill_text(
        "Difficulty:",
        canvas_width / 2.0,
        350.0,
    )?;
    
    // 難易度ボタン
    let difficulties = ["Easy", "Medium", "Hard"];
    for (i, &diff) in difficulties.iter().enumerate() {
        let button_width = 100.0;
        let button_height = 40.0;
        let button_x = canvas_width / 2.0 - 150.0 + (i as f64 * 110.0);
        let button_y = 380.0;
        
        // 選択されているかで色を変える
        let is_selected = ui.selected_difficulty == i;
        let button_color = if is_selected {
            "#00AA00"
        } else {
            "#555555"
        };
        
        context.set_fill_style(&JsValue::from_str(button_color));
        context.fill_rect(button_x, button_y, button_width, button_height);
        
        // ボタンテキスト
        context.set_fill_style(&JsValue::from_str("#FFFFFF"));
        context.set_font("16px Arial");
        context.set_text_align("center");
        context.set_text_baseline("middle");
        
        context.fill_text(
            diff,
            button_x + button_width / 2.0,
            button_y + button_height / 2.0,
        )?;
    }
    
    Ok(())
}

/// ゲーム中のUI描画
fn draw_game_ui(
    _entity_manager: &mut EntityManager,
    resources: &mut HashMap<&'static str, Rc<RefCell<dyn std::any::Any>>>,
    context: &CanvasRenderingContext2d,
    _ui: &UiResource,
    canvas_width: f64,
    _canvas_height: f64,
) -> Result<(), JsValue> {
    // ゲームタイマーを取得
    let game_timer = resources.get("game_timer").and_then(|res| {
        res.clone().borrow_mut().downcast_mut::<TimerResource>().map(|r| r.clone())
    });
    
    // ボードリソースを取得
    let board_resource = resources.get("board").and_then(|res| {
        res.clone().borrow_mut().downcast_mut::<crate::resources::BoardResource>().map(|r| r.clone())
    });
    
    if let Some(timer) = game_timer {
        // タイマー表示
        context.set_fill_style(&JsValue::from_str("#FFFFFF"));
        context.set_font("20px Arial");
        context.set_text_align("left");
        context.set_text_baseline("top");
        
        let seconds = timer.current_time.floor() as i32;
        let minutes = seconds / 60;
        let seconds_remainder = seconds % 60;
        
        context.fill_text(
            &format!("Time: {:02}:{:02}", minutes, seconds_remainder),
            10.0,
            10.0,
        )?;
    }
    
    if let Some(board) = board_resource {
        // 地雷カウンター
        context.set_fill_style(&JsValue::from_str("#FFFFFF"));
        context.set_font("20px Arial");
        context.set_text_align("right");
        context.set_text_baseline("top");
        
        // フラグが立てられた数を数える
        let mut flag_count = 0;
        for &flagged in &board.flagged {
            if flagged {
                flag_count += 1;
            }
        }
        
        let mines_remaining = board.mine_count - flag_count;
        
        context.fill_text(
            &format!("Mines: {}", mines_remaining),
            canvas_width - 10.0,
            10.0,
        )?;
    }
    
    Ok(())
}

/// ゲームオーバー画面の描画
fn draw_game_over_screen(
    resources: &mut HashMap<&'static str, Rc<RefCell<dyn std::any::Any>>>,
    context: &CanvasRenderingContext2d,
    _ui: &UiResource,
    canvas_width: f64,
    canvas_height: f64,
) -> Result<(), JsValue> {
    // ゲーム状態を取得
    let game_state = resources.get("game_state").and_then(|res| {
        res.clone().borrow_mut().downcast_mut::<GameStateResource>().map(|r| r.clone())
    });
    
    // リトライタイマーを取得
    let retry_timer = resources.get("retry_timer").and_then(|res| {
        res.clone().borrow_mut().downcast_mut::<TimerResource>().map(|r| r.clone())
    });
    
    if let Some(state) = game_state {
        // 半透明の背景
        context.set_fill_style(&JsValue::from_str("rgba(0, 0, 0, 0.7)"));
        context.fill_rect(0.0, 0.0, canvas_width, canvas_height);
        
        // 結果テキスト
        let result_text = if state.is_victory {
            "You Win!"
        } else {
            "Game Over"
        };
        
        let text_color = if state.is_victory {
            "#00FF00"
        } else {
            "#FF0000"
        };
        
        context.set_fill_style(&JsValue::from_str(text_color));
        context.set_font("48px Arial");
        context.set_text_align("center");
        context.set_text_baseline("middle");
        
        context.fill_text(
            result_text,
            canvas_width / 2.0,
            canvas_height / 2.0 - 50.0,
        )?;
        
        // リトライメッセージ
        if let Some(timer) = retry_timer {
            context.set_fill_style(&JsValue::from_str("#FFFFFF"));
            context.set_font("24px Arial");
            
            let seconds_remaining = (timer.duration - timer.current_time).ceil() as i32;
            
            context.fill_text(
                &format!("Returning to menu in {} seconds...", seconds_remaining),
                canvas_width / 2.0,
                canvas_height / 2.0 + 50.0,
            )?;
        }
    }
    
    Ok(())
} 