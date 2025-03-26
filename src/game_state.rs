use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};
use std::collections::HashMap;

use crate::js_bindings::{log, update_connection_status, update_player_count, update_game_status};
use crate::models::{CellValue, Screen, Player};
use crate::utils::get_cell_index_from_coordinates;
use crate::rendering::GameRenderer;
use crate::network::{NetworkManager, MessageCallback};

/**
 * ゲーム全体の状態を管理する構造体
 */
pub struct GameState {
    // プレイヤー関連
    pub local_player_id: Option<String>,  // ローカルプレイヤーのID
    pub players: HashMap<String, Player>, // 全プレイヤーの情報

    // 通信関連
    pub network: NetworkManager,          // ネットワーク管理

    // 描画関連
    pub canvas: HtmlCanvasElement,        // キャンバス要素
    pub context: CanvasRenderingContext2d,// 描画コンテキスト
    pub renderer: GameRenderer,           // 描画管理
    
    // マウス操作関連
    pub mouse_x: f64,                     // マウスX座標
    pub mouse_y: f64,                     // マウスY座標
    pub is_mouse_down: bool,              // マウスボタン押下状態
    pub last_position_update: f64,        // 最後に位置情報を送信した時間
    
    // 画面状態
    pub current_screen: Screen,           // 現在の画面
    
    // マインスイーパー特有の状態
    pub board_width: usize,               // ボードの幅
    pub board_height: usize,              // ボードの高さ
    pub mine_count: usize,                // 地雷の数
    pub cell_size: f64,                   // セルのサイズ
    pub cells: Vec<CellValue>,            // セルの値
    pub revealed: Vec<bool>,              // セルが開かれたかどうか
    pub flagged: Vec<bool>,               // フラグが立てられたかどうか
    pub game_started: bool,               // ゲームが開始されたかどうか
    pub game_over: bool,                  // ゲームオーバーかどうか
    pub win: bool,                        // 勝利したかどうか
}

impl GameState {
    /**
     * GameStateの新しいインスタンスを作成する
     * 
     * @param canvas キャンバス要素
     * @return GameStateインスタンス
     */
    pub fn new(canvas: HtmlCanvasElement) -> Result<Self, JsValue> {
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

        // レンダラーの作成
        let renderer = GameRenderer::new(context.clone());

        // ネットワークマネージャーの作成
        let network = NetworkManager::new();

        Ok(Self {
            local_player_id: None,
            players: HashMap::new(),
            network,
            canvas,
            context,
            renderer,
            mouse_x: 0.0,
            mouse_y: 0.0,
            is_mouse_down: false,
            last_position_update: 0.0,
            current_screen: Screen::Title,  // 初期画面はタイトル画面
            
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

    /**
     * WebSocketサーバーに接続する
     * 
     * @return 成功した場合はOk(()), エラーの場合はErr(JsValue)
     */
    pub fn connect_websocket(&mut self) -> Result<(), JsValue> {
        // WebSocketメッセージを処理するコールバック関数を作成
        let this = self as *mut GameState;
        let message_callback: MessageCallback = Box::new(move |json: &serde_json::Value| {
            let game_state = unsafe { &mut *this };
            
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
                                if let CellValue::Mine = game_state.cells[i] {
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
                            let index = index as usize;
                            if index < game_state.flagged.len() {
                                if let Some(flagged) = json["flagged"].as_bool() {
                                    game_state.flagged[index] = flagged;
                                }
                            }
                        }
                    },
                    _ => {
                        log(&format!("Unknown message type: {}", msg_type));
                    }
                }
            }
            
            Ok(())
        });
        
        // WebSocketを接続
        self.network.connect(message_callback)
    }

    /**
     * 自分をプレイヤーとして追加する
     * 
     * @param id プレイヤーID
     * @param other_players 他のプレイヤー情報
     */
    pub fn add_player(&mut self, id: String, other_players: serde_json::Value) {
        // 自分をローカルプレイヤーとして設定
        self.local_player_id = Some(id.clone());
        self.network.set_local_player_id(id.clone());
        
        // 自分をプレイヤーとして追加
        let player = Player {
            id: id.clone(),
            x: self.mouse_x,
            y: self.mouse_y,
            color: "#00FF00".to_string(), // 自分は緑色
        };
        self.players.insert(id, player);
        
        // 他のプレイヤーも追加
        if let Some(players) = other_players.as_object() {
            for (player_id, player_data) in players {
                if let Some(player_obj) = player_data.as_object() {
                    if let (Some(x), Some(y), Some(color)) = (
                        player_obj.get("x").and_then(|v| v.as_f64()),
                        player_obj.get("y").and_then(|v| v.as_f64()),
                        player_obj.get("color").and_then(|v| v.as_str())
                    ) {
                        let player = Player {
                            id: player_id.clone(),
                            x,
                            y,
                            color: color.to_string(),
                        };
                        self.players.insert(player_id.clone(), player);
                    }
                }
            }
        }
        
        // プレイヤー数の表示を更新
        update_player_count(self.players.len());
        
        // ゲーム画面に切り替え
        self.current_screen = Screen::Game;
    }

    /**
     * リモートプレイヤーを追加する
     * 
     * @param id プレイヤーID
     * @param x X座標
     * @param y Y座標
     * @param color カーソルの色
     */
    pub fn add_remote_player(&mut self, id: &str, x: f64, y: f64, color: String) {
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

    /**
     * プレイヤーを削除する
     * 
     * @param id 削除するプレイヤーのID
     */
    pub fn remove_player(&mut self, id: &str) {
        self.players.remove(id);
        
        // プレイヤー数の表示を更新
        update_player_count(self.players.len());
    }

    /**
     * プレイヤーの位置を更新する
     * 
     * @param id 更新するプレイヤーのID
     * @param x 新しいX座標
     * @param y 新しいY座標
     */
    pub fn update_player_position(&mut self, id: &str, x: f64, y: f64) {
        if let Some(player) = self.players.get_mut(id) {
            player.x = x;
            player.y = y;
        }
    }

    /**
     * ゲーム状態を更新する
     * 
     * サーバーから受信したデータを元にゲーム状態を更新します。
     * 
     * @param game_data サーバーから受信したゲーム状態データ
     */
    pub fn update_game_state(&mut self, game_data: &serde_json::Map<String, serde_json::Value>) {
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

    /**
     * ゲーム状態の表示を更新する
     * 
     * 現在のゲーム状態に基づいてUIに表示するステータスを更新します。
     */
    pub fn update_game_status(&self) {
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

    /**
     * マウス座標からセルのインデックスを取得する
     * 
     * 画面上のマウス座標から対応するセルのインデックスを計算します。
     * 座標がボード外の場合はNoneを返します。
     * 
     * @param x マウスのX座標
     * @param y マウスのY座標
     * @return セルのインデックス（Option<usize>）
     */
    pub fn get_cell_index(&self, x: f64, y: f64) -> Option<usize> {
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

    /**
     * ゲームの状態を更新する
     * 
     * プレイヤーの位置などを更新し、画面を再描画します。
     * 
     * @return 成功した場合はOk(()), エラーの場合はErr(JsValue)
     */
    pub fn update(&mut self) -> Result<(), JsValue> {
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

    /**
     * ゲームを描画する
     * 
     * 現在の画面状態に応じて、タイトル画面かゲーム画面を描画します。
     * 
     * @return 成功した場合はOk(()), エラーの場合はErr(JsValue)
     */
    pub fn draw(&mut self) -> Result<(), JsValue> {
        let canvas_width = self.canvas.width() as f64;
        let canvas_height = self.canvas.height() as f64;
        
        match self.current_screen {
            Screen::Title => {
                // タイトル画面を描画
                self.renderer.draw_title_screen(canvas_width, canvas_height, self.network.is_connected)?;
            },
            Screen::Game => {
                // ボードを描画
                self.renderer.draw_board(
                    &self.cells,
                    &self.revealed,
                    &self.flagged,
                    self.board_width,
                    self.board_height,
                    self.cell_size,
                    canvas_width,
                    canvas_height
                )?;
                
                // プレイヤーを描画
                self.renderer.draw_players(&self.players, &self.local_player_id)?;
                
                // UIを描画
                self.renderer.draw_ui(canvas_width)?;
                
                // 接続状態を描画
                self.renderer.draw_connection_status(self.network.is_connected)?;
                
                // ゲームオーバー時の処理
                if self.game_over {
                    if self.win {
                        self.renderer.draw_win_screen(canvas_width, canvas_height)?;
                    } else {
                        self.renderer.draw_game_over_screen(canvas_width, canvas_height)?;
                    }
                }
            }
        }
        
        Ok(())
    }

    /**
     * マウスクリック処理を行う
     * 
     * 画面状態に応じて適切なクリック処理を実行します：
     * - タイトル画面：スタートボタンの処理
     * - ゲーム画面：セルのクリックやフラグ処理
     * 
     * @param x クリック位置のX座標
     * @param y クリック位置のY座標
     * @param right_click 右クリックかどうか
     * @return 成功した場合はOk(()), エラーの場合はErr(JsValue)
     */
    pub fn handle_mouse_click(&mut self, x: f64, y: f64, right_click: bool) -> Result<(), JsValue> {
        match self.current_screen {
            Screen::Title => {
                // スタートボタンの位置を計算
                let canvas_width = self.canvas.width() as f64;
                let canvas_height = self.canvas.height() as f64;
                let button_x = canvas_width / 2.0;
                let button_y = canvas_height / 2.0 + 50.0;
                let button_width = 200.0;
                let button_height = 60.0;
                
                // スタートボタンがクリックされたかチェック
                if x >= button_x - button_width / 2.0 &&
                   x <= button_x + button_width / 2.0 &&
                   y >= button_y - button_height / 2.0 &&
                   y <= button_y + button_height / 2.0 {
                    // ゲーム画面に遷移
                    self.current_screen = Screen::Game;
                    
                    // WebSocketに接続
                    self.connect_websocket()?;
                }
            },
            Screen::Game => {
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
            }
        }
        
        Ok(())
    }

    /**
     * ローカルプレイヤーの位置情報を送信する
     * 
     * 一定間隔で位置情報を送信します。
     * 
     * @return 成功した場合はOk(()), エラーの場合はErr(JsValue)
     */
    pub fn send_position_update(&mut self) -> Result<(), JsValue> {
        // 現在時刻を取得
        let now = js_sys::Date::now();
        
        // 前回の更新から一定時間（100ms）経過していれば送信
        if now - self.last_position_update > 100.0 {
            self.last_position_update = now;
            
            // 位置情報を送信
            self.network.send_position_update(self.mouse_x, self.mouse_y)?;
        }
        
        Ok(())
    }

    /**
     * セルを開く
     * 
     * @param index 開くセルのインデックス
     * @return 成功した場合はOk(()), エラーの場合はErr(JsValue)
     */
    pub fn reveal_cell(&mut self, index: usize) -> Result<(), JsValue> {
        // すでに開かれている、またはフラグが立っている場合は何もしない
        if self.revealed[index] || self.flagged[index] {
            return Ok(());
        }
        
        // ゲームオーバーの場合は何もしない
        if self.game_over {
            return Ok(());
        }
        
        // サーバーに送信
        self.network.send_reveal_cell(index)
    }

    /**
     * フラグを切り替える
     * 
     * @param index フラグを切り替えるセルのインデックス
     * @return 成功した場合はOk(()), エラーの場合はErr(JsValue)
     */
    pub fn toggle_flag(&mut self, index: usize) -> Result<(), JsValue> {
        // すでに開かれている場合は何もしない
        if self.revealed[index] {
            return Ok(());
        }
        
        // ゲームオーバーの場合は何もしない
        if self.game_over {
            return Ok(());
        }
        
        // サーバーに送信
        self.network.send_toggle_flag(index)
    }

    /**
     * ゲームをリセットする
     * 
     * @return 成功した場合はOk(()), エラーの場合はErr(JsValue)
     */
    pub fn reset_game(&mut self) -> Result<(), JsValue> {
        // サーバーに送信
        self.network.send_reset_game()
    }
} 