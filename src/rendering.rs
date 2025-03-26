/**
 * ゲームの描画処理を担当するモジュール
 */
use wasm_bindgen::JsValue;
use web_sys::CanvasRenderingContext2d;
use std::collections::HashMap;

use crate::models::{CellValue, Player, Screen};
use crate::js_bindings::log;

/**
 * ゲームの描画を担当する構造体
 */
pub struct GameRenderer {
    pub context: CanvasRenderingContext2d,
}

impl GameRenderer {
    pub fn new(context: CanvasRenderingContext2d) -> Self {
        Self { context }
    }
    
    /**
     * キャンバスをクリアする
     */
    pub fn clear_canvas(&self, canvas_width: f64, canvas_height: f64) -> Result<(), JsValue> {
        self.context.clear_rect(0.0, 0.0, canvas_width, canvas_height);
        Ok(())
    }
    
    /**
     * ボードを描画する
     */
    pub fn draw_board(
        &self, 
        cells: &[CellValue], 
        revealed: &[bool], 
        flagged: &[bool],
        board_width: usize, 
        board_height: usize,
        cell_size: f64,
        canvas_width: f64,
        canvas_height: f64
    ) -> Result<(), JsValue> {
        let ctx = &self.context;
        
        // ボードの左上の座標
        let board_left = (canvas_width - cell_size * board_width as f64) / 2.0;
        let board_top = (canvas_height - cell_size * board_height as f64) / 2.0;
        
        // 背景を描画
        ctx.set_fill_style(&JsValue::from_str("#333333"));
        ctx.fill_rect(0.0, 0.0, canvas_width, canvas_height);
        
        // ボードを描画
        for y in 0..board_height {
            for x in 0..board_width {
                let index = y * board_width + x;
                let cell_x = board_left + x as f64 * cell_size;
                let cell_y = board_top + y as f64 * cell_size;
                
                // セルの背景
                if revealed[index] {
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
                    cell_size, 
                    cell_size
                );
                
                // 枠線
                ctx.set_stroke_style(&JsValue::from_str("#666666"));
                ctx.set_line_width(1.0);
                ctx.stroke_rect(
                    cell_x, 
                    cell_y, 
                    cell_size, 
                    cell_size
                );
                
                // セルの内容を描画
                if revealed[index] {
                    match cells[index] {
                        CellValue::Mine => {
                            // 地雷
                            ctx.set_fill_style(&JsValue::from_str("#FF0000"));
                            ctx.begin_path();
                            ctx.arc(
                                cell_x + cell_size / 2.0,
                                cell_y + cell_size / 2.0,
                                cell_size / 3.0,
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
                                    cell_x + cell_size / 2.0,
                                    cell_y + cell_size / 2.0,
                                )?;
                            }
                        }
                    }
                } else if flagged[index] {
                    // フラグ
                    ctx.set_fill_style(&JsValue::from_str("#FF0000"));
                    
                    // 旗竿
                    ctx.begin_path();
                    ctx.move_to(cell_x + cell_size * 0.3, cell_y + cell_size * 0.2);
                    ctx.line_to(cell_x + cell_size * 0.3, cell_y + cell_size * 0.8);
                    ctx.set_line_width(2.0);
                    ctx.stroke();
                    
                    // 旗
                    ctx.begin_path();
                    ctx.move_to(cell_x + cell_size * 0.3, cell_y + cell_size * 0.2);
                    ctx.line_to(cell_x + cell_size * 0.7, cell_y + cell_size * 0.35);
                    ctx.line_to(cell_x + cell_size * 0.3, cell_y + cell_size * 0.5);
                    ctx.close_path();
                    ctx.fill();
                }
            }
        }
        
        Ok(())
    }
    
    /**
     * プレイヤーのカーソルを描画する
     */
    pub fn draw_players(
        &self, 
        players: &HashMap<String, Player>,
        local_player_id: &Option<String>
    ) -> Result<(), JsValue> {
        let ctx = &self.context;
        
        // 全プレイヤーを描画
        for (id, player) in players {
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
    
    /**
     * UIを描画する
     */
    pub fn draw_ui(&self, canvas_width: f64) -> Result<(), JsValue> {
        let ctx = &self.context;
        
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
    
    /**
     * 接続状態を描画する
     */
    pub fn draw_connection_status(&self, is_connected: bool) -> Result<(), JsValue> {
        let ctx = &self.context;
        
        // 接続状態の色を設定
        let (color, text) = if is_connected {
            ("#4CAF50", "接続中")
        } else {
            ("#FF0000", "未接続")
        };
        
        // 接続状態の背景
        ctx.set_fill_style(&JsValue::from_str(color));
        ctx.begin_path();
        ctx.arc(30.0, 30.0, 10.0, 0.0, std::f64::consts::PI * 2.0)?;
        ctx.fill();
        
        // 接続状態のテキスト
        ctx.set_fill_style(&JsValue::from_str("#FFFFFF"));
        ctx.set_font("16px Arial");
        ctx.set_text_align("left");
        ctx.set_text_baseline("middle");
        ctx.fill_text(text, 50.0, 30.0)?;
        
        Ok(())
    }
    
    /**
     * タイトル画面を描画する
     */
    pub fn draw_title_screen(&self, canvas_width: f64, canvas_height: f64, is_connected: bool) -> Result<(), JsValue> {
        let ctx = &self.context;
        
        // 背景を描画
        ctx.set_fill_style(&JsValue::from_str("#333333"));
        ctx.fill_rect(0.0, 0.0, canvas_width, canvas_height);
        
        // タイトルを描画
        ctx.set_fill_style(&JsValue::from_str("#FFFFFF"));
        ctx.set_font("bold 48px Arial");
        ctx.set_text_align("center");
        ctx.set_text_baseline("middle");
        ctx.fill_text(
            "マルチプレイヤー\nマインスイーパー",
            canvas_width / 2.0,
            canvas_height / 2.0 - 50.0,
        )?;
        
        // スタートボタンを描画
        let button_x = canvas_width / 2.0;
        let button_y = canvas_height / 2.0 + 50.0;
        let button_width = 200.0;
        let button_height = 60.0;
        
        // ボタンの背景
        ctx.set_fill_style(&JsValue::from_str("#4CAF50"));
        ctx.fill_rect(
            button_x - button_width / 2.0,
            button_y - button_height / 2.0,
            button_width,
            button_height,
        );
        
        // ボタンのテキスト
        ctx.set_fill_style(&JsValue::from_str("#FFFFFF"));
        ctx.set_font("bold 24px Arial");
        ctx.fill_text(
            "スタート",
            button_x,
            button_y,
        )?;
        
        // 接続状態を描画
        self.draw_connection_status(is_connected)?;
        
        Ok(())
    }
    
    /**
     * ゲームオーバー画面を描画する
     */
    pub fn draw_game_over_screen(&self, canvas_width: f64, canvas_height: f64) -> Result<(), JsValue> {
        let ctx = &self.context;
        
        // 半透明の背景
        ctx.set_fill_style(&JsValue::from_str("rgba(0, 0, 0, 0.7)"));
        ctx.fill_rect(0.0, 0.0, canvas_width, canvas_height);
        
        // ゲームオーバーテキスト
        ctx.set_fill_style(&JsValue::from_str("#FF0000"));
        ctx.set_font("bold 48px Arial");
        ctx.set_text_align("center");
        ctx.set_text_baseline("middle");
        ctx.fill_text(
            "ゲームオーバー",
            canvas_width / 2.0,
            canvas_height / 2.0,
        )?;
        
        Ok(())
    }
    
    /**
     * 勝利画面を描画する
     */
    pub fn draw_win_screen(&self, canvas_width: f64, canvas_height: f64) -> Result<(), JsValue> {
        let ctx = &self.context;
        
        // 半透明の背景
        ctx.set_fill_style(&JsValue::from_str("rgba(0, 0, 0, 0.7)"));
        ctx.fill_rect(0.0, 0.0, canvas_width, canvas_height);
        
        // 勝利テキスト
        ctx.set_fill_style(&JsValue::from_str("#00FF00"));
        ctx.set_font("bold 48px Arial");
        ctx.set_text_align("center");
        ctx.set_text_baseline("middle");
        ctx.fill_text(
            "勝利！",
            canvas_width / 2.0,
            canvas_height / 2.0,
        )?;
        
        Ok(())
    }
} 