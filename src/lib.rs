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
use std::borrow::BorrowMut;

// サブモジュールを登録
mod js_bindings;
pub mod models;
mod game_state;
mod utils;
mod rendering;
mod network;
pub mod board;  // ボードモジュールを追加
mod components; // ECSコンポーネント
pub mod entities;   // ECSエンティティ

// ECS関連モジュール
pub mod resources;  // ECSリソース
pub mod system;     // システムの基本インターフェース
pub mod systems;    // 具体的なシステム実装
pub mod ecs;        // ECSコア機能
pub mod ecs_game;   // ECSベースのゲームエンジン
pub mod events;     // イベント関連モジュール

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
    let mouse_move_closure = Closure::wrap(Box::new(move |event: web_sys::MouseEvent| {
        GAME_STATE.with(|gs| {
            if let Some(ref mut game) = *gs.borrow_mut() {
                let rect = game.canvas.get_bounding_client_rect();
                game.mouse_x = event.client_x() as f64 - rect.left();
                game.mouse_y = event.client_y() as f64 - rect.top();
            }
        });
    }) as Box<dyn FnMut(web_sys::MouseEvent)>);
    
    canvas_element.add_event_listener_with_callback(
        "mousemove",
        mouse_move_closure.as_ref().unchecked_ref(),
    )?;
    mouse_move_closure.forget();
    
    // マウスクリックイベントのセットアップ
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
    
    *g.as_ref().borrow_mut() = Some(Closure::wrap(Box::new(move || {
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
    
    // ロガーを初期化
    wasm_logger::init(wasm_logger::Config::default());
    log::info!("Wasm logger initialized");
    
    // TODO: ECSベースのゲーム初期化を実装
    // 一時的な空実装
    log("ECS based game initialization is not implemented yet.");
    
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
    let canvas_for_move = canvas.clone();
    let mouse_move_closure = Closure::wrap(Box::new(move |event: web_sys::MouseEvent| {
        let rect = canvas_for_move.get_bounding_client_rect();
        
        GAME_STATE.with(|gs| {
            if let Some(ref mut game) = *gs.borrow_mut() {
                // マウス座標更新
                game.mouse_x = event.client_x() as f64 - rect.left();
                game.mouse_y = event.client_y() as f64 - rect.top();
            }
        });
    }) as Box<dyn FnMut(web_sys::MouseEvent)>);
    
    // 別のキャンバス参照を作成
    let canvas_for_event = canvas.clone();
    canvas_for_event.add_event_listener_with_callback(
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
    
    *g.as_ref().borrow_mut() = Some(Closure::wrap(Box::new(move || {
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

// テストモジュールの実装（nightly以外のコンパイラではスキップ）
#[cfg(test)]
mod tests {
    // テストケースをここに書く
}

// マクロのエクスポート
// pub use crate::impl_typed_event;
// pub use crate::impl_timestamped_event; 