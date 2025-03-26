/**
 * マルチプレイヤーマインスイーパーゲーム
 * 
 * このモジュールは、WebAssemblyを使用したマルチプレイヤーマインスイーパーゲームの
 * フロントエンド部分を実装しています。WebSocketを使用してサーバーと通信し、
 * マルチプレイヤーでマインスイーパーを楽しむことができます。
 * 
 * 機能:
 * - WebSocketを使用したリアルタイム通信
 * - マルチプレイヤー対応（他のプレイヤーのカーソル表示）
 * - マインスイーパーの基本ルール（地雷回避、数字表示など）
 */
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::HtmlCanvasElement;
use std::rc::Rc;
use std::cell::RefCell;

// サブモジュールを登録
mod js_bindings;
mod models;
mod game_state;
mod utils;
mod rendering;

// サブモジュールからの要素をインポート
use js_bindings::{log, request_animation_frame};
use game_state::GameState;

/**
 * ゲームのエントリーポイント
 * 
 * Webページから呼び出されるWASMのエントリーポイントです。
 * ゲームの初期化、イベントリスナーの設定、アニメーションループの開始を行います。
 * 
 * @param canvas_element ゲームを描画するキャンバス要素
 * @return 成功した場合はOk(()), エラーの場合はErr(JsValue)
 */
#[wasm_bindgen]
pub fn start_game(canvas_element: HtmlCanvasElement) -> Result<(), JsValue> {
    // パニック時にログ出力するようにする
    console_error_panic_hook::set_once();
    
    // ゲーム状態の初期化
    let game_state = Rc::new(RefCell::new(GameState::new(canvas_element.clone())?));
    
    // マウスイベントのセットアップ
    let game_state_clone = game_state.clone();
    let mouse_move_closure = Closure::wrap(Box::new(move |event: web_sys::MouseEvent| {
        let mut game = game_state_clone.borrow_mut();
        let rect = game.canvas.get_bounding_client_rect();
        game.mouse_x = event.client_x() as f64 - rect.left();
        game.mouse_y = event.client_y() as f64 - rect.top();
    }) as Box<dyn FnMut(web_sys::MouseEvent)>);
    
    canvas_element.add_event_listener_with_callback(
        "mousemove",
        mouse_move_closure.as_ref().unchecked_ref(),
    )?;
    mouse_move_closure.forget();
    
    // マウスクリックイベントのセットアップ
    let game_state_clone = game_state.clone();
    let mouse_click_closure = Closure::wrap(Box::new(move |event: web_sys::MouseEvent| {
        event.prevent_default();
        let mut game = game_state_clone.borrow_mut();
        let rect = game.canvas.get_bounding_client_rect();
        let x = event.client_x() as f64 - rect.left();
        let y = event.client_y() as f64 - rect.top();
        
        // 右クリックかどうか
        let right_click = event.button() == 2;
        
        if let Err(e) = game.handle_mouse_click(x, y, right_click) {
            log(&format!("Mouse click error: {:?}", e));
        }
    }) as Box<dyn FnMut(web_sys::MouseEvent)>);
    
    canvas_element.add_event_listener_with_callback(
        "mousedown",
        mouse_click_closure.as_ref().unchecked_ref(),
    )?;
    mouse_click_closure.forget();
    
    // コンテキストメニューを無効化
    let context_menu_closure = Closure::wrap(Box::new(move |event: web_sys::MouseEvent| {
        event.prevent_default();
    }) as Box<dyn FnMut(web_sys::MouseEvent)>);
    
    canvas_element.add_event_listener_with_callback(
        "contextmenu",
        context_menu_closure.as_ref().unchecked_ref(),
    )?;
    context_menu_closure.forget();
    
    // アニメーションフレームのセットアップ
    let f = Rc::new(RefCell::new(None));
    let g = f.clone();
    
    let game_state_clone = game_state.clone();
    *g.borrow_mut() = Some(Closure::wrap(Box::new(move || {
        // ゲームの更新
        if let Err(e) = game_state_clone.borrow_mut().update() {
            log(&format!("Game update error: {:?}", e));
            return;
        }
        
        // 次のフレームをリクエスト
        request_animation_frame(f.borrow().as_ref().unwrap());
    }) as Box<dyn FnMut()>));
    
    request_animation_frame(g.borrow().as_ref().unwrap());
    
    Ok(())
}

// パニックハンドラのセットアップ
extern crate console_error_panic_hook; 