use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, WebSocket, MessageEvent};
use std::collections::HashMap;
use js_sys::Math;
use serde::{Serialize, Deserialize};
use std::rc::Rc;
use std::cell::RefCell;

// JavaScriptの関数を呼び出すためのユーティリティ
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
    
    // JavaScriptのグローバル関数
    #[wasm_bindgen(js_name = updateConnectionStatus)]
    fn update_connection_status(connected: bool);
    
    #[wasm_bindgen(js_name = updatePlayerCount)]
    fn update_player_count(count: usize);
    
    #[wasm_bindgen(js_name = updateGameStatus)]
    fn update_game_status(status: &str);
    
    #[wasm_bindgen(js_name = getWebSocketUrl)]
    fn get_websocket_url() -> String;
}

// セルの状態
#[derive(Clone, Copy, PartialEq, Eq)]
enum CellValue {
    Mine,           // 地雷
    Empty(u8),      // 空白（周囲の地雷数）
}

// ------ プレイヤー構造体 ------ //
#[derive(Clone, Serialize, Deserialize)]
struct Player {
    id: String,
    x: f64,
    y: f64,
    color: String,
}

// ------ ゲーム状態 ------ //
struct GameState {
    local_player_id: Option<String>,
    players: HashMap<String, Player>,
    websocket: Option<WebSocket>,
    canvas: HtmlCanvasElement,
    context: CanvasRenderingContext2d,
    mouse_x: f64,
    mouse_y: f64,
    is_mouse_down: bool,
    last_position_update: f64,
    
    // マインスイーパー特有の状態
    board_width: usize,
    board_height: usize,
    mine_count: usize,
    cell_size: f64,
    cells: Vec<CellValue>,
    revealed: Vec<bool>,
    flagged: Vec<bool>,
    game_started: bool,
    game_over: bool,
    win: bool,
}

impl GameState {
    fn new(canvas: HtmlCanvasElement) -> Result<Self, JsValue> {
        // キャンバスから2Dコンテキストを取得
        let context = canvas
            .get_context("2d")?
            .unwrap()
            .dyn_into::<CanvasRenderingContext2d>()?;
        
        // デフォルトのボードサイズ
        let board_width = 16;
        let board_height = 16;
        let mine_count = 40;
        
        // セルのサイズを計算
        let cell_size = ((canvas.width() as f64).min(canvas.height() as f64) - 40.0) / board_width as f64;

        Ok(Self {
            local_player_id: None,
            players: HashMap::new(),
            websocket: None,
            canvas,
            context,
            mouse_x: 0.0,
            mouse_y: 0.0,
            is_mouse_down: false,
            last_position_update: 0.0,
            
            board_width,
            board_height,
            mine_count,
            cell_size,
            cells: vec![CellValue::Empty(0); board_width * board_height],
            revealed: vec![false; board_width * board_height],
            flagged: vec![false; board_width * board_height],
            game_started: false,
            game_over: false,
            win: false,
        })
    }

    // WebSocketに接続
    fn connect_websocket(&mut self) -> Result<(), JsValue> {
        // オブジェクトを所有させてWebSocketに保存
        let server_url = get_websocket_url();
        log(&format!("Connecting to WebSocket server at: {}", server_url));
        
        let ws = WebSocket::new(&server_url)?;

        // onopen: 接続成功時のコールバック
        let onopen_callback = Closure::wrap(Box::new(move || {
            log("WebSocket connected!");
            update_connection_status(true);
        }) as Box<dyn FnMut()>);
        ws.set_onopen(Some(onopen_callback.as_ref().unchecked_ref()));
        onopen_callback.forget();

        // WebSocketをゲーム状態に保存
        self.websocket = Some(ws.clone());

        // onmessage: メッセージ受信時のコールバック
        let this = self as *mut GameState; // 安全ではないが、クロージャー内でselfを使うためのワークアラウンド

        let onmessage_callback = Closure::wrap(Box::new(move |e: MessageEvent| {
            if let Ok(txt) = e.data().dyn_into::<js_sys::JsString>() {
                let message = String::from(txt);
                log(&format!("Message received: {}", message));
                
                // 安全ではない生ポインタを安全な参照に変換
                // クラッシュを避けるために、これはコールバック内でのみ使用されることに注意
                let game_state = unsafe { &mut *this };
                
                // JSONをパース
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&message) {
                    // メッセージタイプによって処理を分ける
                    if let Some(msg_type) = json["type"].as_str() {
                        match msg_type {
                            "init" => {
                                // 初期化メッセージ
                                log("Init message received!");
                                if let Some(player_id) = json["playerId"].as_str() {
                                    // 自分をプレイヤーとして追加
                                    log(&format!("Adding local player: {}", player_id));
                                    game_state.add_player(player_id.to_string(), json["players"].clone());
                                }
                                
                                // ゲーム状態を更新
                                if let Some(game_data) = json["gameState"].as_object() {
                                    game_state.update_game_state(game_data);
                                }
                            },
                            "player_joined" => {
                                // 新しいプレイヤーが参加
                                if let Some(id) = json["id"].as_str() {
                                    if !game_state.players.contains_key(id) {
                                        log(&format!("Player joined: {}", id));
                                        let color = json["color"].as_str().unwrap_or("#FF0000").to_string();
                                        game_state.add_remote_player(id, 0.0, 0.0, color);
                                    }
                                }
                            },
                            "player_left" => {
                                // プレイヤーが退出
                                if let Some(id) = json["id"].as_str() {
                                    log(&format!("Player left: {}", id));
                                    game_state.remove_player(id);
                                }
                            },
                            "player_moved" => {
                                // プレイヤーの移動
                                if let (Some(id), Some(x), Some(y)) = (
                                    json["id"].as_str(),
                                    json["x"].as_f64(),
                                    json["y"].as_f64()
                                ) {
                                    game_state.update_player_position(id, x, y);
                                }
                            },
                            "cells_revealed" => {
                                // セルが開かれた
                                if let Some(cells) = json["cells"].as_array() {
                                    if let Some(values) = json["values"].as_object() {
                                        // 各セルを開く
                                        for cell in cells {
                                            if let Some(index) = cell.as_i64() {
                                                let index = index as usize;
                                                game_state.revealed[index] = true;
                                                
                                                // セルの値を設定
                                                if let Some(value) = values.get(&index.to_string()) {
                                                    if let Some(v) = value.as_i64() {
                                                        if v == -1 {
                                                            game_state.cells[index] = CellValue::Mine;
                                                        } else {
                                                            game_state.cells[index] = CellValue::Empty(v as u8);
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    
                                    // ゲームオーバーかどうか
                                    if let Some(game_over) = json["gameOver"].as_bool() {
                                        game_state.game_over = game_over;
                                    }
                                    
                                    // 勝利かどうか
                                    if let Some(win) = json["win"].as_bool() {
                                        game_state.win = win;
                                    }
                                    
                                    // ゲーム状態を更新
                                    game_state.update_game_status();
                                }
                            },
                            "game_over" => {
                                // ゲームオーバー
                                game_state.game_over = true;
                                
                                // 勝利かどうか
                                if let Some(win) = json["win"].as_bool() {
                                    game_state.win = win;
                                }
                                
                                // 全てのセル情報を受け取って表示
                                if let Some(all_cell_values) = json["allCellValues"].as_object() {
                                    log(&format!("ゲームオーバー：全てのセル情報を受信 ({} 個)", all_cell_values.len()));
                                    
                                    // 全てのセルの値を設定
                                    for (index_str, value) in all_cell_values {
                                        if let Ok(index) = index_str.parse::<usize>() {
                                            if index < game_state.cells.len() {
                                                if let Some(v) = value.as_i64() {
                                                    if v == -1 {
                                                        game_state.cells[index] = CellValue::Mine;
                                                    } else {
                                                        game_state.cells[index] = CellValue::Empty(v as u8);
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    
                                    // 地雷セルは表示、他は元のまま
                                    for i in 0..game_state.cells.len() {
                                        if matches!(game_state.cells[i], CellValue::Mine) {
                                            game_state.revealed[i] = true;
                                        }
                                    }
                                }
                                
                                // ゲーム状態を更新
                                game_state.update_game_status();
                            },
                            "flag_toggled" => {
                                // フラグが切り替えられた
                                if let Some(index) = json["index"].as_i64() {
                                    if let Some(flagged) = json["flagged"].as_bool() {
                                        game_state.flagged[index as usize] = flagged;
                                    }
                                }
                            },
                            "game_reset" => {
                                // ゲームがリセットされた
                                if let Some(game_data) = json["gameState"].as_object() {
                                    game_state.update_game_state(game_data);
                                }
                                
                                // 新しい形式のリセットメッセージにも対応（トップレベルのプロパティ）
                                let mut props = serde_json::Map::new();
                                
                                if let Some(width) = json["boardWidth"].as_i64() {
                                    props.insert("boardWidth".to_string(), serde_json::json!(width));
                                }
                                
                                if let Some(height) = json["boardHeight"].as_i64() {
                                    props.insert("boardHeight".to_string(), serde_json::json!(height));
                                }
                                
                                if let Some(mines) = json["mineCount"].as_i64() {
                                    props.insert("mineCount".to_string(), serde_json::json!(mines));
                                }
                                
                                // プロパティが取得できたら、ゲーム状態を更新
                                if !props.is_empty() {
                                    log("新しい形式のゲームリセットメッセージを処理しています");
                                    game_state.update_game_state(&props);
                                }
                            },
                            _ => {
                                log(&format!("Unknown message type: {}", msg_type));
                            }
                        }
                    }
                } else {
                    log(&format!("Failed to parse JSON: {}", message));
                }
            }
        }) as Box<dyn FnMut(MessageEvent)>);

        ws.set_onmessage(Some(onmessage_callback.as_ref().unchecked_ref()));
        onmessage_callback.forget();

        // onerror: エラー発生時のコールバック
        let onerror_callback = Closure::wrap(Box::new(move |_: web_sys::Event| {
            log("WebSocket error");
            update_connection_status(false);
        }) as Box<dyn FnMut(web_sys::Event)>);
        ws.set_onerror(Some(onerror_callback.as_ref().unchecked_ref()));
        onerror_callback.forget();

        // onclose: 接続終了時のコールバック
        let onclose_callback = Closure::wrap(Box::new(move |_: web_sys::Event| {
            log("WebSocket closed");
            update_connection_status(false);
        }) as Box<dyn FnMut(web_sys::Event)>);
        ws.set_onclose(Some(onclose_callback.as_ref().unchecked_ref()));
        onclose_callback.forget();

        Ok(())
    }

    // メッセージを送信
    fn send_message(&self, message: &str) -> Result<(), JsValue> {
        if let Some(ws) = &self.websocket {
            ws.send_with_str(message)?;
        }
        Ok(())
    }

    // 位置更新メッセージを送信
    fn send_position_update(&mut self) -> Result<(), JsValue> {
        if let Some(player_id) = &self.local_player_id {
            if let Some(player) = self.players.get(player_id) {
                let now = js_sys::Date::now();
                
                // 位置更新の頻度を制限（100msごと）
                if now - self.last_position_update >= 100.0 {
                    let message = serde_json::json!({
                        "type": "player_move",
                        "x": player.x,
                        "y": player.y
                    });
                    
                    self.send_message(&message.to_string())?;
                    self.last_position_update = now;
                }
            }
        }
        Ok(())
    }

    // セルを開く
    fn reveal_cell(&mut self, index: usize) -> Result<(), JsValue> {
        // 既に開かれている、フラグが立てられている、またはゲームオーバーの場合は何もしない
        if self.revealed[index] || self.flagged[index] || self.game_over {
            return Ok(());
        }
        
        // サーバーにセルを開くメッセージを送信
        let message = serde_json::json!({
            "type": "reveal_cell",
            "index": index
        });
        
        self.send_message(&message.to_string())
    }

    // フラグを切り替え
    fn toggle_flag(&mut self, index: usize) -> Result<(), JsValue> {
        // 既に開かれている、またはゲームオーバーの場合は何もしない
        if self.revealed[index] || self.game_over {
            return Ok(());
        }
        
        // サーバーにフラグを切り替えるメッセージを送信
        let message = serde_json::json!({
            "type": "toggle_flag",
            "index": index
        });
        
        self.send_message(&message.to_string())
    }

    // ゲームをリセット
    fn reset_game(&mut self) -> Result<(), JsValue> {
        // サーバーにゲームをリセットするメッセージを送信
        let message = serde_json::json!({
            "type": "reset_game"
        });
        
        self.send_message(&message.to_string())
    }

    // プレイヤーを追加（初期化時）
    fn add_player(&mut self, local_id: String, players_json: serde_json::Value) {
        // ローカルプレイヤーIDを設定
        self.local_player_id = Some(local_id.clone());
        
        // ローカルプレイヤーを追加
        let player = Player {
            id: local_id.clone(),
            x: self.canvas.width() as f64 / 2.0,
            y: self.canvas.height() as f64 / 2.0,
            color: "#FF0000".to_string(), // サーバーから色が届いていないので仮の色
        };
        
        self.players.insert(local_id, player);
        
        // 他のプレイヤーを追加
        if let Some(players_array) = players_json.as_array() {
            for player_json in players_array {
                if let (Some(id), Some(x), Some(y), Some(color)) = (
                    player_json["id"].as_str(),
                    player_json["x"].as_f64(),
                    player_json["y"].as_f64(),
                    player_json["color"].as_str()
                ) {
                    self.add_remote_player(id, x, y, color.to_string());
                }
            }
        }
        
        // プレイヤー数の表示を更新
        update_player_count(self.players.len());
    }

    // リモートプレイヤーを追加
    fn add_remote_player(&mut self, id: &str, x: f64, y: f64, color: String) {
        let player = Player {
            id: id.to_string(),
            x,
            y,
            color,
        };
        
        self.players.insert(id.to_string(), player);
        
        // プレイヤー数の表示を更新
        update_player_count(self.players.len());
    }

    // プレイヤーを削除
    fn remove_player(&mut self, id: &str) {
        self.players.remove(id);
        
        // プレイヤー数の表示を更新
        update_player_count(self.players.len());
    }

    // プレイヤーの位置を更新
    fn update_player_position(&mut self, id: &str, x: f64, y: f64) {
        if let Some(player) = self.players.get_mut(id) {
            player.x = x;
            player.y = y;
        }
    }

    // ゲーム状態を更新
    fn update_game_state(&mut self, game_data: &serde_json::Map<String, serde_json::Value>) {
        if let Some(width) = game_data.get("boardWidth").and_then(|v| v.as_i64()) {
            self.board_width = width as usize;
        }
        
        if let Some(height) = game_data.get("boardHeight").and_then(|v| v.as_i64()) {
            self.board_height = height as usize;
        }
        
        if let Some(mines) = game_data.get("mineCount").and_then(|v| v.as_i64()) {
            self.mine_count = mines as usize;
        }
        
        // セルサイズを更新
        self.cell_size = ((self.canvas.width() as f64).min(self.canvas.height() as f64) - 40.0) / self.board_width as f64;
        
        // ボードを初期化
        self.cells = vec![CellValue::Empty(0); self.board_width * self.board_height];
        
        // revealed配列を更新
        if let Some(revealed) = game_data.get("revealed").and_then(|v| v.as_array()) {
            self.revealed = revealed.iter()
                .map(|v| v.as_bool().unwrap_or(false))
                .collect();
        } else {
            self.revealed = vec![false; self.board_width * self.board_height];
        }
        
        // flagged配列を更新
        if let Some(flagged) = game_data.get("flagged").and_then(|v| v.as_array()) {
            self.flagged = flagged.iter()
                .map(|v| v.as_bool().unwrap_or(false))
                .collect();
        } else {
            self.flagged = vec![false; self.board_width * self.board_height];
        }
        
        // ゲーム状態を更新
        if let Some(started) = game_data.get("gameStarted").and_then(|v| v.as_bool()) {
            self.game_started = started;
        }
        
        if let Some(over) = game_data.get("gameOver").and_then(|v| v.as_bool()) {
            self.game_over = over;
        }
        
        if let Some(win) = game_data.get("win").and_then(|v| v.as_bool()) {
            self.win = win;
        }
        
        // 既に開かれたセルの値を設定
        if let Some(cell_values) = game_data.get("cellValues").and_then(|v| v.as_object()) {
            log(&format!("セル値を受信: {} 個", cell_values.len()));
            
            for (index_str, value) in cell_values {
                if let Ok(index) = index_str.parse::<usize>() {
                    if index < self.cells.len() {
                        if let Some(v) = value.as_i64() {
                            if v == -1 {
                                self.cells[index] = CellValue::Mine;
                            } else {
                                self.cells[index] = CellValue::Empty(v as u8);
                            }
                            log(&format!("セル[{}]の値を{}に設定", index, v));
                        }
                    }
                }
            }
        }
        
        // ゲーム状態を更新
        self.update_game_status();
    }

    // ゲーム状態の表示を更新
    fn update_game_status(&self) {
        let status = if self.game_over {
            if self.win {
                "勝利！"
            } else {
                "ゲームオーバー！"
            }
        } else if self.game_started {
            "ゲーム中..."
        } else {
            "ゲーム開始待ち..."
        };
        
        update_game_status(status);
    }

    // マウス座標からセルのインデックスを取得
    fn get_cell_index(&self, x: f64, y: f64) -> Option<usize> {
        // ボードの左上の座標
        let board_left = (self.canvas.width() as f64 - self.cell_size * self.board_width as f64) / 2.0;
        let board_top = (self.canvas.height() as f64 - self.cell_size * self.board_height as f64) / 2.0;
        
        // ボード外の場合はNone
        if x < board_left || x >= board_left + self.cell_size * self.board_width as f64 ||
           y < board_top || y >= board_top + self.cell_size * self.board_height as f64 {
            return None;
        }
        
        // セルの座標を計算
        let cell_x = ((x - board_left) / self.cell_size) as usize;
        let cell_y = ((y - board_top) / self.cell_size) as usize;
        
        // インデックスを返す
        Some(cell_y * self.board_width + cell_x)
    }

    // ボードを描画
    fn draw_board(&self) -> Result<(), JsValue> {
        let ctx = &self.context;
        let canvas_width = self.canvas.width() as f64;
        let canvas_height = self.canvas.height() as f64;
        
        // ボードの左上の座標
        let board_left = (canvas_width - self.cell_size * self.board_width as f64) / 2.0;
        let board_top = (canvas_height - self.cell_size * self.board_height as f64) / 2.0;
        
        // 背景を描画
        ctx.set_fill_style(&JsValue::from_str("#333333"));
        ctx.fill_rect(0.0, 0.0, canvas_width, canvas_height);
        
        // ボードを描画
        for y in 0..self.board_height {
            for x in 0..self.board_width {
                let index = y * self.board_width + x;
                let cell_x = board_left + x as f64 * self.cell_size;
                let cell_y = board_top + y as f64 * self.cell_size;
                
                // セルの背景
                if self.revealed[index] {
                    // 開かれたセル
                    ctx.set_fill_style(&JsValue::from_str("#CCCCCC"));
                } else {
                    // 閉じたセル
                    ctx.set_fill_style(&JsValue::from_str("#999999"));
                }
                
                // セルを描画
                ctx.fill_rect(
                    cell_x, 
                    cell_y, 
                    self.cell_size, 
                    self.cell_size
                );
                
                // 枠線
                ctx.set_stroke_style(&JsValue::from_str("#666666"));
                ctx.set_line_width(1.0);
                ctx.stroke_rect(
                    cell_x, 
                    cell_y, 
                    self.cell_size, 
                    self.cell_size
                );
                
                // セルの内容を描画
                if self.revealed[index] {
                    match self.cells[index] {
                        CellValue::Mine => {
                            // 地雷
                            ctx.set_fill_style(&JsValue::from_str("#FF0000"));
                            ctx.begin_path();
                            ctx.arc(
                                cell_x + self.cell_size / 2.0,
                                cell_y + self.cell_size / 2.0,
                                self.cell_size / 3.0,
                                0.0,
                                std::f64::consts::PI * 2.0,
                            )?;
                            ctx.fill();
                        },
                        CellValue::Empty(count) => {
                            if count > 0 {
                                // 周囲の地雷数
                                let color = match count {
                                    1 => "#0000FF", // 青
                                    2 => "#008000", // 緑
                                    3 => "#FF0000", // 赤
                                    4 => "#000080", // 紺
                                    5 => "#800000", // 茶
                                    6 => "#008080", // シアン
                                    7 => "#000000", // 黒
                                    8 => "#808080", // グレー
                                    _ => "#000000", // 黒
                                };
                                
                                ctx.set_fill_style(&JsValue::from_str(color));
                                ctx.set_font("bold 16px Arial");
                                ctx.set_text_align("center");
                                ctx.set_text_baseline("middle");
                                ctx.fill_text(
                                    &count.to_string(),
                                    cell_x + self.cell_size / 2.0,
                                    cell_y + self.cell_size / 2.0,
                                )?;
                            }
                        }
                    }
                } else if self.flagged[index] {
                    // フラグ
                    ctx.set_fill_style(&JsValue::from_str("#FF0000"));
                    
                    // 旗竿
                    ctx.begin_path();
                    ctx.move_to(cell_x + self.cell_size * 0.3, cell_y + self.cell_size * 0.2);
                    ctx.line_to(cell_x + self.cell_size * 0.3, cell_y + self.cell_size * 0.8);
                    ctx.set_line_width(2.0);
                    ctx.stroke();
                    
                    // 旗
                    ctx.begin_path();
                    ctx.move_to(cell_x + self.cell_size * 0.3, cell_y + self.cell_size * 0.2);
                    ctx.line_to(cell_x + self.cell_size * 0.7, cell_y + self.cell_size * 0.35);
                    ctx.line_to(cell_x + self.cell_size * 0.3, cell_y + self.cell_size * 0.5);
                    ctx.close_path();
                    ctx.fill();
                }
            }
        }
        
        Ok(())
    }

    // プレイヤーのカーソルを描画
    fn draw_players(&self) -> Result<(), JsValue> {
        let ctx = &self.context;
        
        // 全プレイヤーを描画
        for (id, player) in &self.players {
            // カーソルを描画
            ctx.set_fill_style(&JsValue::from_str(&player.color));
            ctx.begin_path();
            ctx.arc(
                player.x,
                player.y,
                8.0,
                0.0,
                std::f64::consts::PI * 2.0,
            )?;
            ctx.fill();
            
            // プレイヤーIDを表示
            ctx.set_font("12px Arial");
            ctx.set_text_align("center");
            ctx.set_text_baseline("top");
            ctx.fill_text(
                &id,
                player.x,
                player.y + 10.0,
            )?;
        }
        
        Ok(())
    }

    // UIを描画
    fn draw_ui(&self) -> Result<(), JsValue> {
        let ctx = &self.context;
        let canvas_width = self.canvas.width() as f64;
        
        // リセットボタン
        ctx.set_fill_style(&JsValue::from_str("#4CAF50"));
        ctx.begin_path();
        ctx.ellipse(
            canvas_width - 80.0,
            30.0,
            40.0,
            20.0,
            0.0,
            0.0,
            std::f64::consts::PI * 2.0,
        )?;
        ctx.fill();
        
        ctx.set_fill_style(&JsValue::from_str("#FFFFFF"));
        ctx.set_font("14px Arial");
        ctx.set_text_align("center");
        ctx.set_text_baseline("middle");
        ctx.fill_text(
            "リセット",
            canvas_width - 80.0,
            30.0,
        )?;
        
        Ok(())
    }

    // ゲームの更新
    fn update(&mut self) -> Result<(), JsValue> {
        // ローカルプレイヤーの移動
        if let Some(player_id) = &self.local_player_id {
            if let Some(player) = self.players.get_mut(player_id) {
                // プレイヤーの位置を更新
                player.x = self.mouse_x;
                player.y = self.mouse_y;
                
                // 位置情報を送信
                self.send_position_update()?;
            }
        }
        
        // 描画
        self.draw()?;
        
        Ok(())
    }

    // 描画
    fn draw(&mut self) -> Result<(), JsValue> {
        // ボードを描画
        self.draw_board()?;
        
        // プレイヤーを描画
        self.draw_players()?;
        
        // UIを描画
        self.draw_ui()?;
        
        Ok(())
    }

    // マウスクリック処理
    fn handle_mouse_click(&mut self, x: f64, y: f64, right_click: bool) -> Result<(), JsValue> {
        // リセットボタンがクリックされたかチェック
        let canvas_width = self.canvas.width() as f64;
        let reset_x = canvas_width - 80.0;
        let reset_y = 30.0;
        let dx = x - reset_x;
        let dy = y - reset_y;
        if dx * dx / (40.0 * 40.0) + dy * dy / (20.0 * 20.0) <= 1.0 {
            // リセットボタンがクリックされた
            return self.reset_game();
        }
        
        // クリックされたセルを取得
        if let Some(index) = self.get_cell_index(x, y) {
            if right_click {
                // 右クリック: フラグを切り替え
                self.toggle_flag(index)?;
            } else {
                // 左クリック: セルを開く
                self.reveal_cell(index)?;
            }
        }
        
        Ok(())
    }
}

// ------ WASMエントリーポイント ------ //
#[wasm_bindgen]
pub fn start_game(canvas_element: HtmlCanvasElement) -> Result<(), JsValue> {
    // パニック時にログ出力するようにする
    console_error_panic_hook::set_once();
    
    // ゲーム状態の初期化
    let game_state = Rc::new(RefCell::new(GameState::new(canvas_element.clone())?));
    
    // WebSocketに接続
    game_state.borrow_mut().connect_websocket()?;
    
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

// アニメーションフレームのリクエスト
fn request_animation_frame(f: &Closure<dyn FnMut()>) {
    web_sys::window()
        .unwrap()
        .request_animation_frame(f.as_ref().unchecked_ref())
        .unwrap();
}

// パニックハンドラのセットアップ
extern crate console_error_panic_hook;
