/**
 * マインスイーパーのボードを管理するモジュール
 */
use wasm_bindgen::JsValue;
use crate::models::CellValue;
use crate::utils::{coordinates_to_index, index_to_coordinates, get_adjacent_offsets};
use crate::js_bindings::log;

/**
 * マインスイーパーのボードを表す構造体
 */
pub struct Board {
    /// ボードの幅
    pub width: usize,
    /// ボードの高さ
    pub height: usize,
    /// 地雷の数
    pub mine_count: usize,
    /// セルのサイズ（ピクセル）
    pub cell_size: f64,
    /// セルの値
    pub cells: Vec<CellValue>,
    /// セルが開かれたかどうか
    pub revealed: Vec<bool>,
    /// セルにフラグが立てられたかどうか
    pub flagged: Vec<bool>,
    /// ゲームが開始されたかどうか
    pub game_started: bool,
    /// ゲームオーバーかどうか
    pub game_over: bool,
    /// 勝利したかどうか
    pub win: bool,
}

impl Board {
    /**
     * 新しいボードを作成する
     */
    pub fn new(width: usize, height: usize, mine_count: usize, cell_size: f64) -> Self {
        Self {
            width,
            height,
            mine_count,
            cell_size,
            cells: vec![CellValue::Empty(0); width * height],
            revealed: vec![false; width * height],
            flagged: vec![false; width * height],
            game_started: false,
            game_over: false,
            win: false,
        }
    }
    
    /**
     * ボードを初期化する
     */
    pub fn initialize(&mut self) {
        self.cells = vec![CellValue::Empty(0); self.width * self.height];
        self.revealed = vec![false; self.width * self.height];
        self.flagged = vec![false; self.width * self.height];
        self.game_started = false;
        self.game_over = false;
        self.win = false;
    }
    
    /**
     * 指定されたセルを開く
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
        
        // セルを開く
        self.revealed[index] = true;
        
        // 地雷だった場合はゲームオーバー
        if let CellValue::Mine = self.cells[index] {
            self.game_over = true;
            self.win = false;
            
            // 全ての地雷を表示
            for i in 0..self.cells.len() {
                if let CellValue::Mine = self.cells[i] {
                    self.revealed[i] = true;
                }
            }
            
            return Ok(());
        }
        
        // 0（周囲に地雷がない）の場合は、周囲のセルも開く
        if let CellValue::Empty(0) = self.cells[index] {
            self.reveal_adjacent_cells(index)?;
        }
        
        // 勝利条件をチェック
        self.check_win();
        
        Ok(())
    }
    
    /**
     * 周囲のセルを再帰的に開く（0の場合）
     * 
     * @param index 中心となるセルのインデックス
     * @return 成功した場合はOk(()), エラーの場合はErr(JsValue)
     */
    fn reveal_adjacent_cells(&mut self, index: usize) -> Result<(), JsValue> {
        let (row, col) = index_to_coordinates(index, self.width);
        
        // 周囲8方向のオフセット
        for (dr, dc) in get_adjacent_offsets().iter() {
            let new_row = row as isize + dr;
            let new_col = col as isize + dc;
            
            // ボードの範囲内かチェック
            if new_row >= 0 && new_row < self.height as isize && 
               new_col >= 0 && new_col < self.width as isize {
                let new_index = coordinates_to_index(new_row as usize, new_col as usize, self.width);
                
                // 未開放かつフラグがない場合のみ開く
                if !self.revealed[new_index] && !self.flagged[new_index] {
                    self.revealed[new_index] = true;
                    
                    // 0の場合は再帰的に周囲も開く
                    if let CellValue::Empty(0) = self.cells[new_index] {
                        self.reveal_adjacent_cells(new_index)?;
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /**
     * フラグを切り替える
     * 
     * @param index フラグを切り替えるセルのインデックス
     */
    pub fn toggle_flag(&mut self, index: usize) {
        // すでに開かれている場合は何もしない
        if self.revealed[index] {
            return;
        }
        
        // ゲームオーバーの場合は何もしない
        if self.game_over {
            return;
        }
        
        // フラグを切り替え
        self.flagged[index] = !self.flagged[index];
    }
    
    /**
     * 勝利条件をチェックする
     * 
     * 地雷以外の全てのセルが開かれていれば勝利
     */
    pub fn check_win(&mut self) {
        let total_cells = self.width * self.height;
        let mines = self.mine_count;
        let non_mine_cells = total_cells - mines;
        
        // 開かれたセルの数をカウント
        let revealed_count = self.revealed.iter().filter(|&&r| r).count();
        
        // 地雷以外の全てのセルが開かれていれば勝利
        if revealed_count == non_mine_cells {
            self.game_over = true;
            self.win = true;
            
            // 全ての地雷にフラグを立てる
            for i in 0..self.cells.len() {
                if let CellValue::Mine = self.cells[i] {
                    self.flagged[i] = true;
                }
            }
            
            log("勝利条件を満たしました！");
        }
    }
    
    /**
     * マウス座標からセルのインデックスを取得する
     * 
     * 画面上のマウス座標から対応するセルのインデックスを計算します。
     * 座標がボード外の場合はNoneを返します。
     * 
     * @param x マウスのX座標
     * @param y マウスのY座標
     * @param canvas_width キャンバスの幅
     * @param canvas_height キャンバスの高さ
     * @return セルのインデックス（Option<usize>）
     */
    pub fn get_cell_index(&self, x: f64, y: f64, canvas_width: f64, canvas_height: f64) -> Option<usize> {
        // ボードの左上の座標
        let board_left = (canvas_width - self.cell_size * self.width as f64) / 2.0;
        let board_top = (canvas_height - self.cell_size * self.height as f64) / 2.0;
        
        // ボード外の場合はNone
        if x < board_left || x >= board_left + self.cell_size * self.width as f64 ||
           y < board_top || y >= board_top + self.cell_size * self.height as f64 {
            return None;
        }
        
        // セルの座標を計算
        let cell_x = ((x - board_left) / self.cell_size) as usize;
        let cell_y = ((y - board_top) / self.cell_size) as usize;
        
        // インデックスを返す
        Some(cell_y * self.width + cell_x)
    }
    
    /**
     * サーバーから受信したデータでボードを更新する
     * 
     * @param game_data サーバーから受信したゲーム状態データ
     */
    pub fn update_from_server(&mut self, game_data: &serde_json::Map<String, serde_json::Value>) {
        // ボードサイズの更新
        if let Some(width) = game_data.get("boardWidth").and_then(|v| v.as_i64()) {
            self.width = width as usize;
        }
        
        if let Some(height) = game_data.get("boardHeight").and_then(|v| v.as_i64()) {
            self.height = height as usize;
        }
        
        if let Some(mines) = game_data.get("mineCount").and_then(|v| v.as_i64()) {
            self.mine_count = mines as usize;
        }
        
        // セルサイズの更新は親のGameStateで行う（キャンバスサイズが必要なため）
        
        // ボードを初期化
        self.cells = vec![CellValue::Empty(0); self.width * self.height];
        
        // revealed配列を更新
        if let Some(revealed) = game_data.get("revealed").and_then(|v| v.as_array()) {
            self.revealed = revealed.iter()
                .map(|v| v.as_bool().unwrap_or(false))
                .collect();
        } else {
            self.revealed = vec![false; self.width * self.height];
        }
        
        // flagged配列を更新
        if let Some(flagged) = game_data.get("flagged").and_then(|v| v.as_array()) {
            self.flagged = flagged.iter()
                .map(|v| v.as_bool().unwrap_or(false))
                .collect();
        } else {
            self.flagged = vec![false; self.width * self.height];
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
    }
} 