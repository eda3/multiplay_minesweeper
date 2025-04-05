/**
 * UIシステム
 * 
 * ユーザーインターフェースを更新・描画するシステム
 */
use std::rc::Rc;
use std::cell::RefCell;
use std::any::Any;
use wasm_bindgen::JsValue;

use crate::resources::RenderResource;
use crate::resources::{GameStateResource, GamePhase};
use crate::resources::TimeResource;
use crate::resources::InputResource;

/// ボタンの定義
#[derive(Debug, Clone)]
struct Button {
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    text: String,
    is_hovered: bool,
    is_pressed: bool,
    action: ButtonAction,
}

/// ボタンのアクション
#[derive(Debug, Clone)]
enum ButtonAction {
    StartGame,
    RestartGame,
    QuitGame,
    ChangeDifficulty(usize),
    ToggleSound,
    GoToMenu,
}

/// UIの状態
#[derive(Debug, Clone)]
struct UiState {
    buttons: Vec<Button>,
    active_screen: String,
    hover_index: Option<usize>,
    last_mouse_pos: (i32, i32),
}

/// UIシステム関数
pub fn ui_system(resources: &[Rc<RefCell<dyn Any>>]) {
    // RenderResourceのRcを探す
    let render_rc_option = resources.iter()
        .find(|r| r.borrow().is::<RenderResource>());
    
    // GameStateResourceのRcを探す
    let game_rc_option = resources.iter()
        .find(|r| r.borrow().is::<GameStateResource>());
    
    // TimeResourceのRcを探す
    let time_rc_option = resources.iter()
        .find(|r| r.borrow().is::<TimeResource>());
    
    // InputResourceのRcを探す
    let input_rc_option = resources.iter()
        .find(|r| r.borrow().is::<InputResource>());
    
    // 必要なリソースが見つからない場合は何もしない
    if render_rc_option.is_none() || game_rc_option.is_none() || 
       time_rc_option.is_none() || input_rc_option.is_none() {
        return;
    }
    
    // 借用して処理
    {
        let render = render_rc_option.unwrap().borrow();
        let render = render.downcast_ref::<RenderResource>().unwrap();
        
        let game = game_rc_option.unwrap().borrow();
        let game = game.downcast_ref::<GameStateResource>().unwrap();
        
        let time = time_rc_option.unwrap().borrow();
        let time = time.downcast_ref::<TimeResource>().unwrap();
        
        let input = input_rc_option.unwrap().borrow();
        let input = input.downcast_ref::<InputResource>().unwrap();
        
        // UIの状態を保持する静的変数
        thread_local! {
            static UI_STATE: RefCell<UiState> = RefCell::new(UiState {
                buttons: Vec::new(),
                active_screen: "main".to_string(),
                hover_index: None,
                last_mouse_pos: (0, 0),
            });
        }
        
        // UIの状態を更新
        UI_STATE.with(|ui_state| {
            let mut ui_state = ui_state.borrow_mut();
            
            // マウス位置の更新
            let (mouse_x, mouse_y) = input.get_mouse_position();
            ui_state.last_mouse_pos = (mouse_x, mouse_y);
            
            // UIを更新
            update_ui(&mut ui_state, game, input);
            
            // UIを描画
            if let Some(context) = render.get_context() {
                draw_ui(context, &ui_state, game, time);
            }
        });
    }
}

/// UIの状態を更新
fn update_ui(ui_state: &mut UiState, game: &GameStateResource, input: &InputResource) {
    // ゲームフェーズに応じてUIの状態を更新
    match game.phase {
        GamePhase::StartScreen => {
            if ui_state.active_screen != "start" {
                ui_state.active_screen = "start".to_string();
                ui_state.buttons.clear();
                generate_start_menu_buttons(ui_state);
            }
        },
        GamePhase::Playing => {
            ui_state.active_screen = "playing".to_string();
            ui_state.buttons.clear();
        },
        GamePhase::Paused => {
            if ui_state.active_screen != "pause" {
                ui_state.active_screen = "pause".to_string();
                ui_state.buttons.clear();
                generate_pause_menu_buttons(ui_state);
            }
        },
        GamePhase::GameOver { .. } => {
            if ui_state.active_screen != "gameover" {
                ui_state.active_screen = "gameover".to_string();
                ui_state.buttons.clear();
                generate_game_over_buttons(ui_state);
            }
            },
            _ => {}
    }
    
    // ボタンのホバー状態を更新
    update_button_hover_states(ui_state, ui_state.last_mouse_pos.0, ui_state.last_mouse_pos.1);
    
    // クリック処理
    if input.is_mouse_pressed(0) {
        if let Some(hover_index) = ui_state.hover_index {
            ui_state.buttons[hover_index].is_pressed = true;
            
            // クリックアクションの処理はここでは行わず、ゲームロジックに委ねる
            // 実際のアクションはinput_systemで処理される
        }
    }
}

/// ボタンのホバー状態を更新
fn update_button_hover_states(ui_state: &mut UiState, mouse_x: i32, mouse_y: i32) {
    ui_state.hover_index = None;
    
    for (index, button) in ui_state.buttons.iter_mut().enumerate() {
        let is_hovered = mouse_x >= button.x as i32 && 
                         mouse_x <= (button.x + button.width) as i32 &&
                         mouse_y >= button.y as i32 && 
                         mouse_y <= (button.y + button.height) as i32;
        
        button.is_hovered = is_hovered;
        
        if is_hovered {
            ui_state.hover_index = Some(index);
        }
    }
}

/// スタートメニューのボタンを生成
fn generate_start_menu_buttons(ui_state: &mut UiState) {
    let canvas_width = 800.0;  // 仮の値
    let canvas_height = 600.0; // 仮の値
    
    // 新しいゲームボタン
    ui_state.buttons.push(Button {
        x: canvas_width / 2.0 - 100.0,
        y: canvas_height / 2.0 - 50.0,
        width: 200.0,
        height: 40.0,
        text: "新しいゲーム".to_string(),
        is_hovered: false,
        is_pressed: false,
        action: ButtonAction::StartGame,
    });
    
    // 難易度選択ボタン
    ui_state.buttons.push(Button {
        x: canvas_width / 2.0 - 100.0,
        y: canvas_height / 2.0 + 10.0,
        width: 200.0,
        height: 40.0,
        text: "難易度: 初級".to_string(),
        is_hovered: false,
        is_pressed: false,
        action: ButtonAction::ChangeDifficulty(0),
    });
}

/// ポーズメニューのボタンを生成
fn generate_pause_menu_buttons(ui_state: &mut UiState) {
    let canvas_width = 800.0;  // 仮の値
    let canvas_height = 600.0; // 仮の値
    
    // 再開ボタン
    ui_state.buttons.push(Button {
        x: canvas_width / 2.0 - 100.0,
        y: canvas_height / 2.0 - 60.0,
        width: 200.0,
        height: 40.0,
        text: "ゲームを再開".to_string(),
        is_hovered: false,
        is_pressed: false,
        action: ButtonAction::StartGame,
    });
    
    // リスタートボタン
    ui_state.buttons.push(Button {
        x: canvas_width / 2.0 - 100.0,
        y: canvas_height / 2.0 - 10.0,
        width: 200.0,
        height: 40.0,
        text: "リスタート".to_string(),
        is_hovered: false,
        is_pressed: false,
        action: ButtonAction::RestartGame,
    });
    
    // メニューに戻るボタン
    ui_state.buttons.push(Button {
        x: canvas_width / 2.0 - 100.0,
        y: canvas_height / 2.0 + 40.0,
        width: 200.0,
        height: 40.0,
        text: "メインメニューに戻る".to_string(),
        is_hovered: false,
        is_pressed: false,
        action: ButtonAction::GoToMenu,
    });
}

/// ゲームオーバー時のボタンを生成
fn generate_game_over_buttons(ui_state: &mut UiState) {
    let canvas_width = 800.0;  // 仮の値
    let canvas_height = 600.0; // 仮の値
    
    // リスタートボタン
    ui_state.buttons.push(Button {
        x: canvas_width / 2.0 - 100.0,
        y: canvas_height / 2.0 + 60.0,
        width: 200.0,
        height: 40.0,
        text: "もう一度プレイ".to_string(),
        is_hovered: false,
        is_pressed: false,
        action: ButtonAction::RestartGame,
    });
    
    // メニューに戻るボタン
    ui_state.buttons.push(Button {
        x: canvas_width / 2.0 - 100.0,
        y: canvas_height / 2.0 + 110.0,
        width: 200.0,
        height: 40.0,
        text: "メインメニューに戻る".to_string(),
        is_hovered: false,
        is_pressed: false,
        action: ButtonAction::GoToMenu,
    });
}

/// UIを描画
fn draw_ui(
    context: &web_sys::CanvasRenderingContext2d,
    ui_state: &UiState,
    game: &GameStateResource,
    time: &TimeResource
) {
    match game.phase {
        GamePhase::StartScreen => {
            draw_start_menu(context);
        },
        GamePhase::Playing => {
            draw_playing_ui(context, game, time);
        },
        GamePhase::Paused => {
            draw_pause_menu(context);
        },
        GamePhase::GameOver { .. } => {
            draw_game_over_ui(context, game);
        },
        _ => {}
    }
    
    // ボタンを描画
    for button in &ui_state.buttons {
        draw_button(context, button);
    }
}

/// ボタンを描画
fn draw_button(context: &web_sys::CanvasRenderingContext2d, button: &Button) {
    // ボタンの背景
    let background_color = if button.is_pressed {
        "#005577"
    } else if button.is_hovered {
        "#0088aa"
        } else {
        "#0099cc"
    };
    
    context.set_fill_style(&JsValue::from_str(background_color));
    context.fill_rect(button.x, button.y, button.width, button.height);
    
    // ボタンの枠線
    context.set_stroke_style(&JsValue::from_str("#004466"));
    context.set_line_width(2.0);
    context.stroke_rect(button.x, button.y, button.width, button.height);
    
    // ボタンのテキスト
    context.set_fill_style(&JsValue::from_str("#ffffff"));
    context.set_font("16px Arial");
        context.set_text_align("center");
        context.set_text_baseline("middle");
        context.fill_text(
            &button.text,
        button.x + button.width / 2.0, 
        button.y + button.height / 2.0
    ).unwrap();
}

/// スタートメニューを描画
fn draw_start_menu(context: &web_sys::CanvasRenderingContext2d) {
    context.set_fill_style(&JsValue::from_str("#333333"));
    context.set_font("32px Arial");
    context.set_text_align("center");
    context.set_text_baseline("middle");
    context.fill_text("マインスイーパー", 400.0, 150.0).unwrap();
}

/// プレイ中のUIを描画
fn draw_playing_ui(
    context: &web_sys::CanvasRenderingContext2d,
    game: &GameStateResource,
    time: &TimeResource
) {
    // 経過時間を表示
    let elapsed_seconds = (game.elapsed_time / 1000.0) as u32;
    let minutes = elapsed_seconds / 60;
    let seconds = elapsed_seconds % 60;
    
    context.set_fill_style(&JsValue::from_str("#333333"));
    context.set_font("18px Arial");
    context.set_text_align("left");
    context.set_text_baseline("top");
    context.fill_text(
        &format!("時間: {:02}:{:02}", minutes, seconds),
        10.0,
        10.0
    ).unwrap();
    
    // スコアを表示
    context.set_text_align("right");
        context.fill_text(
        &format!("スコア: {}", game.score),
        790.0,
        10.0
    ).unwrap();
    
    // FPSを表示（デバッグモードの場合）
    if game.debug_mode {
        context.set_text_align("left");
        context.set_font("12px Arial");
        context.fill_text(
            &format!("FPS: {:.1}", time.get_fps()),
            10.0,
            40.0
        ).unwrap();
    }
}

/// ポーズメニューを描画
fn draw_pause_menu(context: &web_sys::CanvasRenderingContext2d) {
    // 半透明の背景
    context.set_fill_style(&JsValue::from_str("rgba(0, 0, 0, 0.5)"));
    context.fill_rect(0.0, 0.0, 800.0, 600.0);
    
    // ポーズタイトル
    context.set_fill_style(&JsValue::from_str("#ffffff"));
    context.set_font("32px Arial");
    context.set_text_align("center");
    context.set_text_baseline("middle");
    context.fill_text("一時停止", 400.0, 150.0).unwrap();
}

/// ゲームオーバーUIを描画
fn draw_game_over_ui(context: &web_sys::CanvasRenderingContext2d, game: &GameStateResource) {
        // 半透明の背景
        context.set_fill_style(&JsValue::from_str("rgba(0, 0, 0, 0.7)"));
    context.fill_rect(0.0, 0.0, 800.0, 600.0);
    
    // 結果タイトル
    let title = match game.phase {
        GamePhase::GameOver { win, .. } => {
            if win {
                "ゲームクリア！"
        } else {
                "ゲームオーバー"
            }
        },
        _ => "ゲーム終了"
    };
    
    context.set_fill_style(&JsValue::from_str(
        if title == "ゲームクリア！" { "#44ff44" } else { "#ff4444" }
    ));
    context.set_font("32px Arial");
        context.set_text_align("center");
        context.set_text_baseline("middle");
    context.fill_text(title, 400.0, 150.0).unwrap();
    
    // スコア表示
    context.set_fill_style(&JsValue::from_str("#ffffff"));
    context.set_font("24px Arial");
    
    if let GamePhase::GameOver { score, time, .. } = game.phase {
        let minutes = time.as_secs() / 60;
        let seconds = time.as_secs() % 60;
        
        context.fill_text(
            &format!("スコア: {}", score),
            400.0,
            200.0
        ).unwrap();
            
            context.fill_text(
            &format!("時間: {:02}:{:02}", minutes, seconds),
            400.0,
            240.0
        ).unwrap();
    }
} 