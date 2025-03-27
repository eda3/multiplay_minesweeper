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
use std::thread::LocalKey;

// サブモジュールを登録
mod js_bindings;
mod models;
mod game_state;
mod utils;
mod rendering;
mod network;
mod board;  // ボードモジュールを追加
mod components; // ECSコンポーネント
pub mod entities;   // ECSエンティティ

// ECS関連モジュール
pub mod resources;  // ECSリソース
pub mod system;     // システムの基本インターフェース
pub mod systems;    // 具体的なシステム実装
pub mod ecs;        // ECSコア機能
pub mod ecs_game;   // ECSベースのゲームエンジン

// サブモジュールからの要素をインポート
use js_bindings::{log, request_animation_frame};
use game_state::GameState;

// ECS関連のコンポーネントを再エクスポート
pub use ecs_game::EcsGame;
pub use ecs::World;
pub use system::{System, SystemRegistry};
pub use resources::ResourceManager;

pub mod compat_game_state;  // 互換レイヤーを追加

// グローバルなゲーム状態（互換レイヤー）
thread_local! {
    static GAME_STATE: RefCell<Option<compat_game_state::CompatGameState>> = RefCell::new(None);
}

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

/**
 * ECSベースの新ゲームエンジンを使用するゲームの初期化
 * 
 * @param canvas_id キャンバスのID
 * @return 成功した場合はOk(()), エラーの場合はErr(JsValue)
 */
#[wasm_bindgen]
pub fn init_game(canvas_id: &str) -> Result<(), JsValue> {
    // パニックハンドラを設定
    console_error_panic_hook::set_once();
    
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let canvas = document.get_element_by_id(canvas_id).unwrap();
    let canvas: web_sys::HtmlCanvasElement = canvas.dyn_into::<web_sys::HtmlCanvasElement>()?;
    
    // 互換GameStateクラスのインスタンスを作成
    let game_state = compat_game_state::CompatGameState::new(canvas)?;
    
    // グローバル変数に保存
    GAME_STATE.with(|gs| {
        *gs.borrow_mut() = Some(game_state);
    });
    
    // イベントリスナーとアニメーションフレームの設定
    setup_event_listeners(canvas_id)?;
    
    Ok(())
}

/**
 * イベントリスナーとアニメーションフレームのセットアップ
 * 
 * @param canvas_id キャンバスのID
 * @return 成功した場合はOk(()), エラーの場合はErr(JsValue)
 */
fn setup_event_listeners(canvas_id: &str) -> Result<(), JsValue> {
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let canvas = document.get_element_by_id(canvas_id).unwrap();
    let canvas: web_sys::HtmlCanvasElement = canvas.dyn_into::<web_sys::HtmlCanvasElement>()?;
    
    // マウス移動イベント
    let mouse_move_closure = Closure::wrap(Box::new(move |event: web_sys::MouseEvent| {
        GAME_STATE.with(|gs| {
            if let Some(ref mut game) = *gs.borrow_mut() {
                let rect = game.canvas.get_bounding_client_rect();
                let x = event.client_x() as f64 - rect.left();
                let y = event.client_y() as f64 - rect.top();
                
                // マウス座標を更新
                game.set_mouse_position(x, y);
            }
        });
    }) as Box<dyn FnMut(web_sys::MouseEvent)>);
    
    canvas.add_event_listener_with_callback(
        "mousemove",
        mouse_move_closure.as_ref().unchecked_ref(),
    )?;
    mouse_move_closure.forget();
    
    // マウスクリックイベント
    let mouse_click_closure = Closure::wrap(Box::new(move |event: web_sys::MouseEvent| {
        event.prevent_default();
        
        GAME_STATE.with(|gs| {
            if let Some(ref mut game) = *gs.borrow_mut() {
                let rect = game.canvas.get_bounding_client_rect();
                let x = event.client_x() as f64 - rect.left();
                let y = event.client_y() as f64 - rect.top();
                
                // 右クリックかどうか
                let right_click = event.button() == 2;
                
                if let Err(e) = game.handle_mouse_click(x, y, right_click) {
                    log(&format!("Mouse click error: {:?}", e));
                }
            }
        });
    }) as Box<dyn FnMut(web_sys::MouseEvent)>);
    
    canvas.add_event_listener_with_callback(
        "mousedown",
        mouse_click_closure.as_ref().unchecked_ref(),
    )?;
    mouse_click_closure.forget();
    
    // マウスダウンイベント
    let mouse_down_closure = Closure::wrap(Box::new(move |event: web_sys::MouseEvent| {
        GAME_STATE.with(|gs| {
            if let Some(ref mut game) = *gs.borrow_mut() {
                // マウスダウン状態を設定
                game.set_mouse_down(true);
            }
        });
    }) as Box<dyn FnMut(web_sys::MouseEvent)>);
    
    canvas.add_event_listener_with_callback(
        "mousedown",
        mouse_down_closure.as_ref().unchecked_ref(),
    )?;
    mouse_down_closure.forget();
    
    // マウスアップイベント
    let mouse_up_closure = Closure::wrap(Box::new(move |event: web_sys::MouseEvent| {
        GAME_STATE.with(|gs| {
            if let Some(ref mut game) = *gs.borrow_mut() {
                // マウスアップ状態を設定
                game.set_mouse_down(false);
            }
        });
    }) as Box<dyn FnMut(web_sys::MouseEvent)>);
    
    canvas.add_event_listener_with_callback(
        "mouseup",
        mouse_up_closure.as_ref().unchecked_ref(),
    )?;
    mouse_up_closure.forget();
    
    // コンテキストメニューを無効化
    let context_menu_closure = Closure::wrap(Box::new(move |event: web_sys::MouseEvent| {
        event.prevent_default();
    }) as Box<dyn FnMut(web_sys::MouseEvent)>);
    
    canvas.add_event_listener_with_callback(
        "contextmenu",
        context_menu_closure.as_ref().unchecked_ref(),
    )?;
    context_menu_closure.forget();
    
    // アニメーションフレームのセットアップ
    let f = Rc::new(RefCell::new(None));
    let g = f.clone();
    
    *g.borrow_mut() = Some(Closure::wrap(Box::new(move || {
        // ゲームの更新
        GAME_STATE.with(|gs| {
            if let Some(ref mut game) = *gs.borrow_mut() {
                if let Err(e) = game.update() {
                    log(&format!("Game update error: {:?}", e));
                    return;
                }
            }
        });
        
        // 次のフレームをリクエスト
        request_animation_frame(f.borrow().as_ref().unwrap());
    }) as Box<dyn FnMut()>));
    
    request_animation_frame(g.borrow().as_ref().unwrap());
    
    Ok(())
} 