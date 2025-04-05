use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};
use std::collections::HashMap;

use crate::js_bindings::{log, update_connection_status, update_player_count, update_game_status};
use crate::models::{CellValue, Screen, Player};
use crate::utils::get_cell_index_from_coordinates;
use crate::rendering::{Renderer, GameRenderer, DrawOptions};
use crate::network::{NetworkManager, MessageCallback};
use crate::board::Board;
use crate::components::position::Position;
use crate::resources::MouseState;

// 位置情報の更新間隔（ミリ秒）
const POSITION_UPDATE_INTERVAL: f64 = 100.0;

/**
 * ゲーム状態
 * 
 * ゲーム全体の状態を管理する
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
    pub mouse_state: MouseState,          // マウス状態
    pub last_position_update: f64,        // 最後に位置情報を送信した時間
    
    // 画面状態
    pub current_screen: Screen,           // 現在の画面
    
    // ボード関連
    pub board: Board,                     // ゲームボード
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
        
        // ボードの作成
        let board = Board::new(board_width, board_height, mine_count, cell_size);

        Ok(Self {
            local_player_id: None,
            players: HashMap::new(),
            network,
            canvas,
            context,
            renderer,
            mouse_state: MouseState {
                x: 0.0,
                y: 0.0,
                left_button: false,
                right_button: false,
                middle_button: false,
                prev_x: 0.0,
                prev_y: 0.0,
                clicked: false,
                right_clicked: false,
            },
            last_position_update: 0.0,
            current_screen: Screen::Title,  // 初期画面はタイトル画面
            board,
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
                                        game_state.board.revealed[index] = true;
                                        
                                        // セルの値を設定
                                        if let Some(value) = values.get(&index.to_string()) {
                                            if let Some(v) = value.as_i64() {
                                                if v == -1 {
                                                    game_state.board.cells[index] = CellValue::Mine;
                                                } else {
                                                    game_state.board.cells[index] = CellValue::Empty(v as u8);
                                                }
                                            }
                                        }
                                    }
                                }
                                
                                // ゲームオーバーかどうか
                                if let Some(game_over) = json["gameOver"].as_bool() {
                                    game_state.board.game_over = game_over;
                                }
                                
                                // 勝利かどうか
                                if let Some(win) = json["win"].as_bool() {
                                    game_state.board.win = win;
                                }
                                
                                // ゲーム状態を更新
                                game_state.update_game_status();
                            }
                        }
                    },
                    "game_over" => {
                        // ゲームオーバー
                        game_state.board.game_over = true;
                        
                        // 勝利かどうか
                        if let Some(win) = json["win"].as_bool() {
                            game_state.board.win = win;
                        }
                        
                        // 全てのセル情報を受け取って表示
                        if let Some(all_cell_values) = json["allCellValues"].as_object() {
                            log(&format!("ゲームオーバー：全てのセル情報を受信 ({} 個)", all_cell_values.len()));
                            
                            // 全てのセルの値を設定
                            for (index_str, value) in all_cell_values {
                                if let Ok(index) = index_str.parse::<usize>() {
                                    if index < game_state.board.cells.len() {
                                        if let Some(v) = value.as_i64() {
                                            if v == -1 {
                                                game_state.board.cells[index] = CellValue::Mine;
                                            } else {
                                                game_state.board.cells[index] = CellValue::Empty(v as u8);
                                            }
                                        }
                                    }
                                }
                            }
                            
                            // 地雷セルは表示、他は元のまま
                            for i in 0..game_state.board.cells.len() {
                                if let CellValue::Mine = game_state.board.cells[i] {
                                    game_state.board.revealed[i] = true;
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
                            if index < game_state.board.flagged.len() {
                                if let Some(flagged) = json["flagged"].as_bool() {
                                    game_state.board.flagged[index] = flagged;
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
            name: format!("プレイヤー_{}", id),
            x: self.mouse_state.x as f64,
            y: self.mouse_state.y as f64,
            color: "#00FF00".to_string(), // 自分は緑色
            score: 0,
            is_local: true,
            is_host: true,
            is_alive: true,
            cells_revealed: 0,
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
                            name: format!("プレイヤー_{}", player_id),
                            x,
                            y,
                            color: color.to_string(),
                            score: 0,
                            is_local: false,
                            is_host: false,
                            is_alive: true,
                            cells_revealed: 0,
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
            name: format!("プレイヤー_{}", id),
            x,
            y,
            color,
            score: 0,
            is_local: false,
            is_host: false,
            is_alive: true,
            cells_revealed: 0,
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
        // ボードの更新を委譲
        self.board.update_from_server(game_data);
        
        // セルサイズの更新（キャンバスサイズが必要なため、ここで行う）
        self.board.cell_size = ((self.canvas.width() as f64).min(self.canvas.height() as f64) - 40.0) / self.board.width as f64;
        
        // ゲーム状態の表示を更新
        self.update_game_status();
    }

    /**
     * ゲーム状態の表示を更新する
     * 
     * 現在のゲーム状態に基づいてUIに表示するステータスを更新します。
     */
    pub fn update_game_status(&self) {
        let status = if self.board.game_over {
            if self.board.win {
                "勝利！"
            } else {
                "ゲームオーバー！"
            }
        } else if self.board.game_started {
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
        let canvas_width = self.canvas.width() as f64;
        let canvas_height = self.canvas.height() as f64;
        
        self.board.get_cell_index(x, y, canvas_width, canvas_height)
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
                player.x = self.mouse_state.x as f64;
                player.y = self.mouse_state.y as f64;
                
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
     * 現在の画面状態に応じて適切な描画を行います：
     * - タイトル画面：タイトルと接続状態
     * - ゲーム画面：ボード、セル、プレイヤー
     * 
     * @return 成功した場合はOk(()), エラーの場合はErr(JsValue)
     */
    pub fn draw(&mut self) -> Result<(), JsValue> {
        // キャンバスのサイズを取得
        let canvas_width = self.canvas.width() as f64;
        let canvas_height = self.canvas.height() as f64;
        
        // キャンバスをクリア
        self.renderer.clear_canvas(canvas_width, canvas_height)?;
        
        // 描画オプションを作成
        let draw_options = DrawOptions {
            width: canvas_width,
            height: canvas_height,
            cell_size: self.board.cell_size as f64,
            scale: 1.0,
        };
        
        match self.current_screen {
            Screen::Title => {
                // タイトル画面を描画
                self.renderer.draw_title_screen(canvas_width, canvas_height, self.network.is_connected)?;
            },
            Screen::Game => {
                // ゲーム画面を描画
                let board = &self.board;
                
                // 描画オプションを更新
                let draw_options = DrawOptions {
                    width: canvas_width,
                    height: canvas_height,
                    cell_size: board.cell_size as f64,
                    scale: 1.0,
                };
                
                // ボードを描画
                self.renderer.draw_board(board, &draw_options)?;
                
                // プレイヤーを描画
                let positions: HashMap<String, Position> = self.players
                    .iter()
                    .map(|(id, player)| {
                        (id.clone(), Position {
                            x: player.x as f64,
                            y: player.y as f64,
                        })
                    })
                    .collect();
                
                // UIを描画
                self.renderer.draw_ui(&draw_options)?;
                
                // ゲーム終了状態なら対応する画面を描画
                if board.game_over {
                    if board.win {
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
                        self.toggle_flag(x, y)?;
                    } else {
                        // 左クリック: セルを開く
                        self.reveal_cell(x, y)?;
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
            self.network.send_position_update(self.mouse_state.x as f64, self.mouse_state.y as f64)?;
        }
        
        Ok(())
    }

    /**
     * セルを開く
     * 
     * @param x クリックされたセルのX座標
     * @param y クリックされたセルのY座標
     * @return 成功した場合はOk(()), エラーの場合はErr(JsValue)
     */
    pub fn reveal_cell(&mut self, x: f64, y: f64) -> Result<(), JsValue> {
        let cell_index = match self.get_cell_index(x, y) {
            Some(index) => index,
            None => return Ok(()),
        };
        
        if self.current_screen == Screen::Game && !self.is_game_over() {
            if self.is_multiplayer() && self.network.is_connected {
                self.network.send_message(&serde_json::json!({
                    "type": "reveal_cell",
                    "index": cell_index as u32,
                    "player_id": self.local_player_id.clone().unwrap_or_default()
                }))?;
            } else {
                self.handle_cell_reveal(cell_index);
            }
        }
        
        Ok(())
    }

    /**
     * フラグを切り替える
     * 
     * @param x クリックされたセルのX座標
     * @param y クリックされたセルのY座標
     * @return 成功した場合はOk(()), エラーの場合はErr(JsValue)
     */
    pub fn toggle_flag(&mut self, x: f64, y: f64) -> Result<(), JsValue> {
        let cell_index = match self.get_cell_index(x, y) {
            Some(index) => index,
            None => return Ok(()),
        };
        
        if self.current_screen == Screen::Game && !self.is_game_over() {
            if self.is_multiplayer() && self.network.is_connected {
                self.network.send_message(&serde_json::json!({
                    "type": "toggle_flag",
                    "index": cell_index as u32,
                    "player_id": self.local_player_id.clone().unwrap_or_default()
                }))?;
            } else {
                self.handle_flag_toggle(cell_index);
            }
        }
        
        Ok(())
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

    /**
     * マウス座標を設定する
     * 
     * @param x マウスのX座標
     * @param y マウスのY座標
     * @return 成功した場合はOk(()), エラーの場合はErr(JsValue)
     */
    pub fn set_mouse_position(&mut self, x: f64, y: f64) {
        self.mouse_state.x = x;
        self.mouse_state.y = y;
        
        if self.is_multiplayer() && self.network.is_connected {
            self.update_local_player_position(x, y);
            
            // マウス座標更新間隔の制限
            let current_time = js_sys::Date::now();
            if current_time - self.last_position_update > POSITION_UPDATE_INTERVAL {
                self.last_position_update = current_time;
                self.send_position_update().unwrap_or_else(|e| {
                    crate::js_bindings::log(&format!("Error sending position update: {:?}", e));
                });
            }
        }
    }
    
    /// マウスボタンの状態を設定
    pub fn set_mouse_down(&mut self, button: i32, is_down: bool) -> Result<(), JsValue> {
        // ボタンによって状態を更新
        match button {
            0 => self.mouse_state.left_button = is_down,
            2 => self.mouse_state.right_button = is_down,
            _ => {}
        }
        
        Ok(())
    }

    pub fn position(&self) -> Position {
        Position {
            x: self.mouse_state.x,
            y: self.mouse_state.y,
        }
    }

    fn update_local_player_position(&mut self, x: f64, y: f64) {
        if let Some(player) = self.players.get_mut(&self.local_player_id.clone().unwrap_or_default()) {
            player.x = x;
            player.y = y;
        }
    }

    fn is_game_over(&self) -> bool {
        self.board.game_over
    }

    fn is_multiplayer(&self) -> bool {
        self.local_player_id.is_some()
    }

    fn handle_cell_reveal(&mut self, index: usize) {
        // Implementation of handle_cell_reveal method
    }

    fn handle_flag_toggle(&mut self, index: usize) {
        // Implementation of handle_flag_toggle method
    }
} 