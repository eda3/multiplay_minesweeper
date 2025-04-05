/**
 * 互換GameState
 * 
 * 既存のGameStateと同じインターフェースを持ちつつ、内部ではECSアーキテクチャを使用する
 * 段階的な移行のための互換レイヤー
 */
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};
use std::collections::HashMap;
use std::cell::RefCell;
use std::rc::Rc;
use js_sys::{Math, Object, Reflect, Date};

use crate::js_bindings::{log, update_connection_status, update_player_count, update_game_status};
use crate::models::{CellValue, Screen, Player};
use crate::utils::get_cell_index_from_coordinates;
use crate::rendering::{Renderer, DrawOptions, GameRenderer};
use crate::network::{NetworkManager, MessageCallback};
use crate::board::Board;

// ECS関連
use crate::ecs_game::EcsGame;
use crate::resources::{
    BoardResource,
    PlayerStateResource,
    GameStateResource,
    GamePhase, 
    TimeResource, 
    InputResource,
    GameConfigResource,
    MouseState
};

/**
 * ゲーム全体の状態を管理する互換構造体
 * 既存のGameStateと同じインターフェースを持ち、内部ではECSを使用する
 */
#[derive(Clone)]
pub struct CompatGameState {
    // ECSゲームエンジン
    pub ecs_game: EcsGame,
    
    // 描画関連（ECSに移行中なので一時的に保持）
    pub canvas: HtmlCanvasElement,
    pub context: CanvasRenderingContext2d,
    pub renderer: GameRenderer,
    
    // ネットワーク関連（ECSに移行中なので一時的に保持）
    pub network: NetworkManager,
    
    // 現在の状態（ECSに移行中なので一時的に保持）
    pub current_screen: Screen,
    pub board: Option<Board>,
    
    // マウス座標（一時的に保持）
    pub mouse_x: f64,
    pub mouse_y: f64,
    pub mouse_down: bool,
    pub last_position_update: f64,
    
    // プレイヤー情報（互換性のため）
    pub players: HashMap<String, Player>,
    
    // ローカルプレイヤーID
    pub local_player_id: Option<String>,
}

/// 位置情報（互換性のため）
#[derive(Debug, Clone)]
pub struct Position {
    /// X座標
    pub x: f64,
    /// Y座標
    pub y: f64,
}

impl CompatGameState {
    /**
     * CompatGameStateの新しいインスタンスを作成する
     * 
     * @param canvas キャンバス要素
     * @return CompatGameStateインスタンス
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
        let board = Some(Board::new(board_width, board_height, mine_count, cell_size));
        
        // ECSゲームエンジンの作成
        let mut ecs_game = EcsGame::new();
        ecs_game.initialize();
        
        // ゲームコンフィグの設定
        if let Some(game_config) = ecs_game.get_resource_mut::<GameStateResource>() {
            game_config.set_custom_board(board_width, board_height, mine_count);
            game_config.update_cell_size(canvas.width() as f64, canvas.height() as f64);
        }
        
        Ok(Self {
            ecs_game,
            canvas,
            context,
            renderer,
            network,
            current_screen: Screen::Title,
            board,
            mouse_x: 0.0,
            mouse_y: 0.0,
            mouse_down: false,
            last_position_update: 0.0,
            players: HashMap::new(),
            local_player_id: None,
        })
    }
    
    /**
     * WebSocketサーバーに接続する
     * 
     * @return 成功した場合はOk(()), エラーの場合はErr(JsValue)
     */
    pub fn connect_websocket(&mut self) -> Result<(), JsValue> {
        // WebSocketメッセージを処理するコールバック関数を作成
        let this = self as *mut CompatGameState;
        let message_callback: MessageCallback = Box::new(move |json: &serde_json::Value| -> Result<(), JsValue> {
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
                        
                        Ok(())
                    },
                    "player_joined" => {
                        // 新しいプレイヤーが参加
                        if let Some(id) = json["id"].as_str() {
                            // PlayerStateResourceを更新
                            if let Some(player_state) = game_state.ecs_game.get_resource_mut::<PlayerStateResource>() {
                                let color = json["color"].as_str().unwrap_or("#FF0000").to_string();
                                if !player_state.has_player(&id.to_string()) {
                                    log(&format!("Player joined: {}", id));
                                    player_state.add_player(id.to_string(), 0.0, 0.0, color.clone());
                                    
                                    // 既存のハッシュマップも更新（移行期間中）
                                    game_state.add_remote_player(id, 0.0, 0.0, color);
                                }
                            }
                        }
                        
                        Ok(())
                    },
                    "player_left" => {
                        // プレイヤーが退出
                        if let Some(id) = json["id"].as_str() {
                            log(&format!("Player left: {}", id));
                            
                            // PlayerStateResourceを更新
                            if let Some(player_state) = game_state.ecs_game.get_resource_mut::<PlayerStateResource>() {
                                player_state.remove_player();
                            }
                            
                            // 既存のハッシュマップも更新（移行期間中）
                            game_state.remove_player(id);
                        }
                        
                        Ok(())
                    },
                    "player_moved" => {
                        // プレイヤーの移動
                        if let (Some(id), Some(x), Some(y)) = (
                            json["id"].as_str(),
                            json["x"].as_f64(),
                            json["y"].as_f64()
                        ) {
                            // PlayerStateResourceを更新
                            if let Some(player_state) = game_state.ecs_game.get_resource_mut::<PlayerStateResource>() {
                                player_state.update_player_position(x, y);
                            }
                            
                            // 既存のハッシュマップも更新（移行期間中）
                            game_state.update_player_position(x, y);
                        }
                        
                        Ok(())
                    },
                    "cells_revealed" => {
                        // セルが開かれた
                        if let Some(cells) = json["cells"].as_array() {
                            if let Some(values) = json["values"].as_object() {
                                // 各セルを開く
                                for cell in cells {
                                    if let Some(index) = cell.as_i64() {
                                        let index = index as usize;
                                        game_state.board.as_mut().unwrap().revealed[index] = true;
                                        
                                        // セルの値を設定
                                        if let Some(value) = values.get(&index.to_string()) {
                                            if let Some(v) = value.as_i64() {
                                                if v == -1 {
                                                    game_state.board.as_mut().unwrap().cells[index] = CellValue::Mine;
                                                } else {
                                                    game_state.board.as_mut().unwrap().cells[index] = CellValue::Empty(v as u8);
                                                }
                                            }
                                        }
                                    }
                                }
                                
                                // ゲームオーバーかどうか
                                if let Some(game_over) = json["gameOver"].as_bool() {
                                    game_state.board.as_mut().unwrap().game_over = game_over;
                                    
                                    // CoreGameResourceも更新
                                    if game_over {
                                        if let Some(win) = json["win"].as_bool() {
                                            game_state.ecs_game.end_game(win);
                                        }
                                    }
                                }
                                
                                // 勝利かどうか
                                if let Some(win) = json["win"].as_bool() {
                                    game_state.board.as_mut().unwrap().win = win;
                                    
                                    // CoreGameResourceも更新
                                    game_state.ecs_game.end_game(win);
                                }
                                
                                // ゲーム状態を更新
                                game_state.update_game_status();
                            }
                        }
                        
                        Ok(())
                    },
                    "game_over" => {
                        // ゲームオーバー
                        game_state.board.as_mut().unwrap().game_over = true;
                        
                        // 勝利かどうか
                        if let Some(win) = json["win"].as_bool() {
                            game_state.board.as_mut().unwrap().win = win;
                            
                            // CoreGameResourceも更新
                            game_state.ecs_game.end_game(win);
                        }
                        
                        // 全てのセル情報を受け取って表示
                        if let Some(all_cell_values) = json["allCellValues"].as_object() {
                            log(&format!("ゲームオーバー：全てのセル情報を受信 ({} 個)", all_cell_values.len()));
                            
                            // 全てのセルの値を設定
                            for (index_str, value) in all_cell_values {
                                if let Ok(index) = index_str.parse::<usize>() {
                                    if index < game_state.board.as_ref().unwrap().cells.len() {
                                        if let Some(v) = value.as_i64() {
                                            if v == -1 {
                                                game_state.board.as_mut().unwrap().cells[index] = CellValue::Mine;
                                            } else {
                                                game_state.board.as_mut().unwrap().cells[index] = CellValue::Empty(v as u8);
                                            }
                                            // セルを表示状態に
                                            game_state.board.as_mut().unwrap().revealed[index] = true;
                                        }
                                    }
                                }
                            }
                        }
                        
                        // ゲーム状態を更新
                        game_state.update_game_status();
                        
                        Ok(())
                    },
                    _ => {
                        // その他のメッセージは無視
                        log(&format!("Unknown message type: {}", msg_type));
                        Ok(())
                    }
                }
            } else {
                Ok(())
            }
        });
        
        // WebSocketに接続
        self.network.connect(message_callback)
    }

    /**
     * 新しいプレイヤーを追加する
     * 
     * @param id プレイヤーID
     * @param other_players 他のプレイヤー情報
     */
    pub fn add_player(&mut self, id: String, other_players: serde_json::Value) {
        // 自分のプレイヤーを追加
        let color = format!("#{:06x}", (id.as_bytes()[0] as u32 * 0xFFFFFF) / 256);
        
        // PlayerStateResourceを更新
        if let Some(player_state) = self.ecs_game.get_resource_mut::<PlayerStateResource>() {
            player_state.set_local_player_id(id.clone());
            player_state.add_player(id.clone(), 0.0, 0.0, color.clone());
        }
        
        // 移行期間中は既存のプレイヤーマップも更新
        // 現在は互換レイヤーなのでPlayerの不足フィールドは一時的に省略
        
        // 他のプレイヤーを追加
        if let Some(players) = other_players.as_object() {
            for (player_id, player_data) in players {
                if player_id != &id {  // 自分以外のプレイヤー
                    if let (Some(x), Some(y), Some(color)) = (
                        player_data["x"].as_f64(),
                        player_data["y"].as_f64(), 
                        player_data["color"].as_str()
                    ) {
                        // PlayerStateResourceを更新
                        if let Some(player_state) = self.ecs_game.get_resource_mut::<PlayerStateResource>() {
                            player_state.add_player(player_id.to_string(), x, y, color.to_string());
                        }
                        
                        // 移行期間中は既存のプレイヤーマップも更新
                        self.add_remote_player(player_id, x, y, color.to_string());
                    }
                }
            }
        }
        
        // 接続状態とプレイヤー数を更新
        update_connection_status(true);
        update_player_count(self.get_player_count());
    }
    
    /**
     * リモートプレイヤーを追加する
     * 
     * @param id プレイヤーID
     * @param x X座標
     * @param y Y座標
     * @param color 色
     */
    pub fn add_remote_player(&mut self, id: &str, x: f64, y: f64, color: String) {
        // PlayerStateResourceを更新（すでに上位の関数で更新されている場合もある）
        if let Some(player_state) = self.ecs_game.get_resource_mut::<PlayerStateResource>() {
            if !player_state.has_player(&id.to_string()) {
                player_state.add_player(id.to_string(), x, y, color.clone());
            }
        }
        
        // 移行期間中は既存のプレイヤーマップも更新
        // 現在は互換レイヤーなのでPlayerの不足フィールドは一時的に省略
        
        // プレイヤー数を更新
        update_player_count(self.get_player_count() + 1);
    }

    /**
     * プレイヤーを削除する
     * 
     * @param id プレイヤーID
     */
    pub fn remove_player(&mut self, id: &str) {
        // PlayerStateResourceを更新（すでに上位の関数で更新されている場合もある）
        if let Some(player_state) = self.ecs_game.get_resource_mut::<PlayerStateResource>() {
            player_state.remove_player();
        }
        
        // プレイヤー数を更新
        update_player_count(self.get_player_count() - 1);
    }

    /**
     * プレイヤーのポジションを更新する
     * 
     * @param x X座標
     * @param y Y座標
     */
    pub fn update_player_position(&mut self, x: f64, y: f64) {
        // ECSゲームリソースを更新
        if let Some(mut player_state) = self.ecs_game.get_resource_mut::<PlayerStateResource>() {
            player_state.update_player_position(x, y);
        }
        
        // WebSocketでの配信は別のシステムで処理
    }

    /**
     * ゲーム状態を更新する
     * 
     * @param game_data ゲームデータ
     */
    pub fn update_game_state(&mut self, game_data: &serde_json::Map<String, serde_json::Value>) {
        // ボードデータの更新
        if let Some(board_data) = game_data.get("board") {
            if let (Some(width), Some(height), Some(mine_count)) = (
                board_data["width"].as_i64(),
                board_data["height"].as_i64(),
                board_data["mineCount"].as_i64()
            ) {
                // セルサイズを計算
                let cell_size = ((self.canvas.width() as f64).min(self.canvas.height() as f64) - 40.0) / width as f64;
                
                // ボードを再作成
                self.board = Some(Board::new(width as usize, height as usize, mine_count as usize, cell_size));
                
                // GameConfigResourceを更新
                if let Some(game_config) = self.ecs_game.get_resource_mut::<GameStateResource>() {
                    game_config.set_custom_board(width as usize, height as usize, mine_count as usize);
                    game_config.update_cell_size(self.canvas.width() as f64, self.canvas.height() as f64);
                }
            }
        }
        
        // ゲームフェーズの更新
        if let Some(phase) = game_data.get("phase") {
            if let Some(phase_str) = phase.as_str() {
                match phase_str {
                    "playing" => self.ecs_game.start_game(),
                    "paused" => self.ecs_game.pause_game(),
                    "gameover" => {
                        if let Some(win) = game_data.get("win").and_then(|w| w.as_bool()) {
                            self.ecs_game.end_game(win);
                        }
                    },
                    _ => {}
                }
            }
        }
    }

    /**
     * ゲームステータスを更新
     */
    pub fn update_game_status(&self) {
        let status = if self.board.as_ref().unwrap().game_over {
            if self.board.as_ref().unwrap().win {
                "勝利！"
            } else {
                "ゲームオーバー！"
            }
        } else if self.board.as_ref().unwrap().game_started {
            "ゲーム中..."
        } else {
            "ゲーム開始待ち..."
        };
        
        update_game_status(status);
    }

    /**
     * 指定座標のセルインデックスを取得
     * 
     * @param x X座標
     * @param y Y座標
     * @return セルインデックス（ボード範囲外の場合はNone）
     */
    pub fn get_cell_index(&self, x: f64, y: f64) -> Option<usize> {
        get_cell_index_from_coordinates(
            x, y,
            self.board.as_ref()?.cell_size,
            self.board.as_ref()?.width, 
            self.board.as_ref()?.height
        )
    }

    /**
     * ゲームの更新処理
     * 
     * @return 成功した場合はOk(()), エラーの場合はErr(JsValue)
     */
    pub fn update(&mut self) -> Result<(), JsValue> {
        // マウス座標をPlayerStateResourceに反映
        if let Some(mut player_state) = self.ecs_game.get_resource_mut::<PlayerStateResource>() {
            player_state.update_player_position(self.mouse_x, self.mouse_y);
            
            // マウスボタンの状態も更新
            if self.mouse_down {
                player_state.set_mouse_state(MouseState::LEFT_DOWN);
            } else {
                player_state.set_mouse_state(MouseState::UP);
            }
        }
        
        // ECSゲームエンジンの更新
        self.ecs_game.update();
        
        // 描画処理
        self.draw()?;
        
        // 定期的にプレイヤー位置を送信
        self.send_position_update()?;
        
        Ok(())
    }

    /**
     * 描画処理
     * 
     * @return 成功した場合はOk(()), エラーの場合はErr(JsValue)
     */
    pub fn draw(&mut self) -> Result<(), JsValue> {
        // クリア
        self.renderer.clear()?;
        
        // 描画オプションを作成
        let canvas_width = self.canvas.width() as f64;
        let canvas_height = self.canvas.height() as f64;
        let draw_options = DrawOptions {
            width: canvas_width,
            height: canvas_height,
            cell_size: 30.0, // デフォルト値
            scale: 1.0,
        };
        
        // 現在の画面を描画
        match self.current_screen {
            Screen::Title => {
                self.renderer.draw_title_screen(
                    self.canvas.width() as f64,
                    self.canvas.height() as f64,
                    self.network.is_connected
                )?;
            },
            Screen::Game => {
                if let Some(board) = &self.board {
                    // 描画オプションを更新
                    let draw_options = DrawOptions {
                        width: canvas_width,
                        height: canvas_height,
                        cell_size: board.cell_size,
                        scale: 1.0,
                    };
                    
                    // ボードを描画
                    self.renderer.draw_board(board, &draw_options)?;
                }
            }
        }
        
        Ok(())
    }

    /**
     * マウスクリックハンドラ
     * 
     * @param x X座標
     * @param y Y座標
     * @param right_click 右クリックかどうか
     * @return 成功した場合はOk(()), エラーの場合はErr(JsValue)
     */
    pub fn handle_mouse_click(&mut self, x: f64, y: f64, right_click: bool) -> Result<(), JsValue> {
        // 画面によって異なる処理
        match self.current_screen {
            Screen::Title => {
                // タイトル画面の場合、クリックでゲーム画面に遷移
                self.current_screen = Screen::Game;
                
                // 接続を試みる
                self.connect_websocket()?;
            },
            Screen::Game => {
                // ゲーム画面の場合、ボードのセルをクリック
                
                // ゲームオーバー時は何もしない
                if self.board.as_ref().map_or(true, |b| b.game_over) {
                    return Ok(());
                }
                
                // セルのインデックスを取得
                if let Some(index) = self.get_cell_index(x, y) {
                    if right_click {
                        // 右クリック：フラグのトグル
                        self.toggle_flag(index)?;
                    } else {
                        // 左クリック：セルを開く
                        self.reveal_cell(index)?;
                    }
                }
            }
        }
        
        Ok(())
    }

    /**
     * 自分の位置情報を送信
     * 
     * @return 成功した場合はOk(()), エラーの場合はErr(JsValue)
     */
    pub fn send_position_update(&mut self) -> Result<(), JsValue> {
        // 最後の位置情報送信から100ms以上経過している場合のみ送信
        let now = js_sys::Date::now();
        
        // タイトル画面やゲームオーバー時は送信しない
        if self.current_screen != Screen::Game || self.board.as_ref().map_or(true, |b| b.game_over) {
            return Ok(());
        }
        
        // TimeResourceから時間情報を取得
        let current_time = if let Some(time) = self.ecs_game.get_resource::<TimeResource>() {
            time.elapsed().as_secs_f64()
        } else {
            now
        };
        
        if current_time - self.last_position_update >= 100.0 {
            // マウス位置を取得
            let mouse_x = self.mouse_x;
            let mouse_y = self.mouse_y;
            
            // 位置情報を送信
            self.network.send_position_update(mouse_x, mouse_y)?;
            
            // 最終送信時間を更新
            self.last_position_update = current_time;
        }
        
        Ok(())
    }

    /**
     * セルを開く
     * 
     * @param index セルのインデックス
     * @return 成功した場合はOk(()), エラーの場合はErr(JsValue)
     */
    pub fn reveal_cell(&mut self, index: usize) -> Result<(), JsValue> {
        // 既に開いているセルや旗が立っているセルは開けない
        if self.board.as_ref().map_or(true, |b| b.revealed[index] || b.flagged[index]) {
            return Ok(());
        }
        
        // サーバーにセルを開く要求を送信
        self.network.send_reveal_cell(index)?;
        
        Ok(())
    }

    /**
     * セルにフラグを立てる/外す
     * 
     * @param index セルのインデックス
     * @return 成功した場合はOk(()), エラーの場合はErr(JsValue)
     */
    pub fn toggle_flag(&mut self, index: usize) -> Result<(), JsValue> {
        // 既に開いているセルはフラグ不可
        if self.board.as_ref().map_or(true, |b| b.revealed[index]) {
            return Ok(());
        }
        
        // フラグのトグル
        self.board.as_mut().unwrap().flagged[index] = !self.board.as_mut().unwrap().flagged[index];
        
        // ゲームステータスを更新
        self.update_game_status();
        
        Ok(())
    }

    /**
     * ゲームをリセット
     * 
     * @return 成功した場合はOk(()), エラーの場合はErr(JsValue)
     */
    pub fn reset_game(&mut self) -> Result<(), JsValue> {
        // サーバーにリセット要求を送信
        self.network.send_reset_game()?;
        
        Ok(())
    }
    
    /**
     * マウス位置を設定する（イベントハンドラから呼ばれる）
     */
    pub fn set_mouse_position(&mut self, x: f64, y: f64) {
        self.mouse_x = x;
        self.mouse_y = y;
        
        // PlayerStateResourceも更新
        if let Some(player_state) = self.ecs_game.get_resource_mut::<PlayerStateResource>() {
            player_state.update_player_position(x, y);
        }
    }
    
    /**
     * マウスボタンの状態を設定する（イベントハンドラから呼ばれる）
     */
    pub fn set_mouse_down(&mut self, down: bool) {
        self.mouse_down = down;
        
        // PlayerStateResourceも更新
        if let Some(player_state) = self.ecs_game.get_resource_mut::<PlayerStateResource>() {
            player_state.set_mouse_state(if down { MouseState::LEFT_DOWN } else { MouseState::UP });
        }
    }
    
    // ヘルパーメソッド（GameStateの移行中のため、既存のメソッドの代用）
    
    // プレイヤー数を取得
    fn get_player_count(&self) -> usize {
        if let Some(player_state) = self.ecs_game.get_resource::<PlayerStateResource>() {
            player_state.all_players().len()
        } else {
            0
        }
    }

    pub fn init_board(&mut self, width: u32, height: u32, mine_count: u32) {
        // ボードを新規作成
        self.board = Some(Board::new(width as usize, height as usize, mine_count as usize, 30.0));
        
        // GameStateResourceも更新
        // 備考：GameStateResourceのメソッドがないため、コメントアウト
        // if let Some(game_state) = self.ecs_game.get_resource_mut::<GameStateResource>() {
        //     game_state.initialize();
        // }
    }

    fn on_cell_revealed(&mut self, index: usize, revealed_by: &str) -> Result<(), JsValue> {
        // 自分のボードを直接参照
        if let Some(board) = &mut self.board {
            if !board.revealed[index] {
                // 初めて開かれたセル
                board.revealed[index] = true;

                // 得点処理
                if let Some(local_id) = &self.local_player_id {
                    if revealed_by == local_id {
                        if let Some(value) = get_cell_value(&board.cells[index]) {
                            if value > 0 {
                                // 数字セルを開いた場合、より高い数字ほど多く得点
                                // スコア処理は実装必要
                                // self.increase_score(value as u32 * 10);
                            } else {
                                // 空白セルは最小得点
                                // self.increase_score(5);
                            }
                        }
                    }
                }

                // 地雷セルを開いた場合、ゲームオーバー
                if let CellValue::Mine = board.cells[index] {
                    board.game_over = true;
                    board.win = false;
                    
                    // GameStateResourceの状態も更新 - 注：end_gameメソッドがないためコメントアウト
                    // if let Some(game_state) = self.ecs_game.get_resource_mut::<GameStateResource>() {
                    //     game_state.end_game(false);
                    // }
                }

                // 全ての非地雷セルが開かれたらゲームクリア
                let non_mine_cells = board.cells.iter().filter(|&c| !is_mine(c)).count();
                let revealed_count = board.revealed.iter().filter(|&r| *r).count();
                
                if revealed_count >= non_mine_cells {
                    board.game_over = true;
                    board.win = true;
                    
                    // GameStateResourceの状態も更新 - 注：end_gameメソッドがないためコメントアウト
                    // if let Some(game_state) = self.ecs_game.get_resource_mut::<GameStateResource>() {
                    //     game_state.end_game(true);
                    // }
                }
            }
        }

        Ok(())
    }

    /// ゲームステートを取得するメソッド
    pub fn get_game_state(&self) -> &GameStateResource {
        self.ecs_game.get_resource::<GameStateResource>().expect("GameStateResource not found")
    }

    /// ゲームステートをミュータブルで取得するメソッド
    pub fn get_game_state_mut(&mut self) -> &mut GameStateResource {
        self.ecs_game.get_resource_mut::<GameStateResource>().expect("GameStateResource not found")
    }
}

/// セルの値を取得する関数
fn get_cell_value(cell: &CellValue) -> Option<u8> {
    match cell {
        CellValue::Empty(value) => Some(*value),
        _ => None
    }
}

/// セルが地雷かどうかを判定する関数
fn is_mine(cell: &CellValue) -> bool {
    matches!(cell, CellValue::Mine)
} 