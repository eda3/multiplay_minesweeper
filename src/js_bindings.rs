use wasm_bindgen::prelude::*;

/**
 * JavaScriptの関数を呼び出すためのユーティリティ
 */
#[wasm_bindgen]
extern "C" {
    // JavaScriptのconsole.log関数を呼び出すためのバインディング
    #[wasm_bindgen(js_namespace = console)]
    pub fn log(s: &str);
    
    // JavaScriptのグローバル関数
    // 接続状態をUIに表示するための関数
    #[wasm_bindgen(js_name = updateConnectionStatus)]
    pub fn update_connection_status(connected: bool);
    
    // プレイヤー数をUIに表示するための関数
    #[wasm_bindgen(js_name = updatePlayerCount)]
    pub fn update_player_count(count: usize);
    
    // ゲーム状態をUIに表示するための関数
    #[wasm_bindgen(js_name = updateGameStatus)]
    pub fn update_game_status(status: &str);
    
    // WebSocketのURLを取得するための関数
    #[wasm_bindgen(js_name = getWebSocketUrl)]
    pub fn get_websocket_url() -> String;
}

/**
 * アニメーションフレームをリクエストする
 * 
 * ブラウザの次の描画タイミングでコールバックを実行するようにリクエストします。
 * ゲームのアニメーションループを実現するために使用されます。
 * 
 * @param f 次のフレームで実行するクロージャ
 */
pub fn request_animation_frame(f: &Closure<dyn FnMut()>) {
    web_sys::window()
        .unwrap()
        .request_animation_frame(f.as_ref().unchecked_ref())
        .unwrap();
} 