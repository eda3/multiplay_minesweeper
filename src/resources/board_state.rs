/**
 * ボードリソース
 * 
 * マインスイーパーのゲームボード状態を管理する
 */
use super::resource_trait::Resource;
use std::{fmt::Debug, any::Any};

/// セルの状態
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CellState {
    /// 隠れた状態
    Hidden,
    /// 開かれた状態
    Revealed,
    /// フラグが立てられた状態
    Flagged,
    /// 爆発した状態
    Exploded,
}

/// セルのデータ
#[derive(Debug, Clone)]
pub struct Cell {
    /// 周囲の地雷数
    pub adjacent_mines: u8,
    /// 地雷かどうか
    pub is_mine: bool,
    /// セルの状態
    pub state: CellState,
    /// セルの位置（インデックス）
    pub index: usize,
}

impl Cell {
    /// 新しいセルを作成
    pub fn new(index: usize) -> Self {
        Self {
            adjacent_mines: 0,
            is_mine: false,
            state: CellState::Hidden,
            index,
        }
    }
    
    /// セルを開く
    pub fn reveal(&mut self) -> bool {
        if self.state == CellState::Hidden {
            if self.is_mine {
                self.state = CellState::Exploded;
                return true; // 爆発
            } else {
                self.state = CellState::Revealed;
            }
        }
        false // 爆発していない
    }
    
    /// フラグを切り替える
    pub fn toggle_flag(&mut self) {
        if self.state == CellState::Hidden {
            self.state = CellState::Flagged;
        } else if self.state == CellState::Flagged {
            self.state = CellState::Hidden;
        }
    }
    
    /// セルが開かれているかどうか
    pub fn is_revealed(&self) -> bool {
        matches!(self.state, CellState::Revealed | CellState::Exploded)
    }
    
    /// セルにフラグが立っているかどうか
    pub fn is_flagged(&self) -> bool {
        self.state == CellState::Flagged
    }
}

/// ボード設定
#[derive(Debug, Clone)]
pub struct BoardConfig {
    /// ボードの幅
    pub width: usize,
    /// ボードの高さ
    pub height: usize,
    /// 地雷の数
    pub mine_count: usize,
    /// セルのサイズ（ピクセル）
    pub cell_size: usize,
}

impl Default for BoardConfig {
    fn default() -> Self {
        Self {
            width: 16,
            height: 16,
            mine_count: 40,
            cell_size: 30,
        }
    }
}

impl BoardConfig {
    /// プリセット：初級
    pub fn beginner() -> Self {
        Self {
            width: 9,
            height: 9,
            mine_count: 10,
            cell_size: 30,
        }
    }
    
    /// プリセット：中級
    pub fn intermediate() -> Self {
        Self {
            width: 16,
            height: 16,
            mine_count: 40,
            cell_size: 30,
        }
    }
    
    /// プリセット：上級
    pub fn expert() -> Self {
        Self {
            width: 30,
            height: 16,
            mine_count: 99,
            cell_size: 30,
        }
    }
    
    /// カスタム設定
    pub fn custom(width: usize, height: usize, mine_count: usize) -> Self {
        let max_mines = (width * height) / 2; // 最大地雷数は総セル数の半分まで
        let mine_count = mine_count.min(max_mines);
        
        Self {
            width,
            height,
            mine_count,
            cell_size: 30,
        }
    }
    
    /// ボードの総セル数
    pub fn total_cells(&self) -> usize {
        self.width * self.height
    }
    
    /// ボードの幅（ピクセル）
    pub fn board_width_px(&self) -> usize {
        self.width * self.cell_size
    }
    
    /// ボードの高さ（ピクセル）
    pub fn board_height_px(&self) -> usize {
        self.height * self.cell_size
    }
}

/// ボードリソース
#[derive(Debug, Clone)]
pub struct BoardResource {
    /// ボード設定
    pub config: BoardConfig,
    /// セルの配列
    pub cells: Vec<Cell>,
    /// 最初のクリックフラグ
    pub first_click: bool,
    /// 残りの地雷数
    pub remaining_mines: usize,
    /// 判明した地雷数
    pub discovered_mines: usize,
    /// 開かれたセル数
    pub revealed_cells: usize,
}

impl BoardResource {
    /// 新しいボードリソースを作成
    pub fn new(config: BoardConfig) -> Self {
        let cell_count = config.total_cells();
        let cells = (0..cell_count).map(Cell::new).collect();
        
        Self {
            remaining_mines: config.mine_count,
            discovered_mines: 0,
            revealed_cells: 0,
            first_click: true,
            config,
            cells,
        }
    }
    
    /// ボードを初期化（地雷を配置）
    pub fn initialize(&mut self, first_click_index: usize) {
        // 最初にクリックしたセルとその周囲には地雷を配置しない
        let mut safe_cells = Vec::new();
        safe_cells.push(first_click_index);
        
        // 周囲のセルも安全に
        for neighbor_idx in self.get_neighbor_indices(first_click_index) {
            safe_cells.push(neighbor_idx);
        }
        
        // 地雷をランダムに配置
        let mut remaining_mines = self.config.mine_count;
        let total_cells = self.config.total_cells();
        let mut rng = js_sys::Math::random;
        
        while remaining_mines > 0 {
            let idx = (rng() * (total_cells as f64)) as usize;
            
            // 安全セルには配置しない
            if safe_cells.contains(&idx) {
                continue;
            }
            
            let cell = &mut self.cells[idx];
            if !cell.is_mine {
                cell.is_mine = true;
                
                // 周囲のセルの隣接地雷数を増やす
                for neighbor_idx in self.get_neighbor_indices(idx) {
                    self.cells[neighbor_idx].adjacent_mines += 1;
                }
                
                remaining_mines -= 1;
            }
        }
        
        self.first_click = false;
    }
    
    /// 指定されたインデックスのセルを開く
    pub fn reveal_cell(&mut self, index: usize) -> bool {
        if index >= self.cells.len() {
            return false;
        }
        
        // 最初のクリックなら初期化
        if self.first_click {
            self.initialize(index);
        }
        
        // すでに開かれているかフラグがあれば何もしない
        let cell = &self.cells[index];
        if cell.is_revealed() || cell.is_flagged() {
            return false;
        }
        
        // セルを開く
        let exploded = self.cells[index].reveal();
        if exploded {
            // 爆発した場合
            return true;
        }
        
        self.revealed_cells += 1;
        
        // 周囲に地雷がなければ、周囲のセルも開く
        if self.cells[index].adjacent_mines == 0 {
            let neighbors = self.get_neighbor_indices(index);
            for &neighbor_idx in &neighbors {
                if !self.cells[neighbor_idx].is_revealed() && !self.cells[neighbor_idx].is_flagged() {
                    self.reveal_cell(neighbor_idx);
                }
            }
        }
        
        false
    }
    
    /// フラグを切り替える
    pub fn toggle_flag(&mut self, index: usize) {
        if index < self.cells.len() {
            self.cells[index].toggle_flag();
        }
    }
    
    /// 周囲のセルのインデックスを取得
    pub fn get_neighbor_indices(&self, index: usize) -> Vec<usize> {
        let width = self.config.width;
        let height = self.config.height;
        let total_cells = width * height;
        
        if index >= total_cells {
            return vec![];
        }
        
        let row = index / width;
        let col = index % width;
        
        let mut neighbors = Vec::with_capacity(8);
        
        // 周囲8方向のセルを調べる
        for i in -1..=1 {
            for j in -1..=1 {
                if i == 0 && j == 0 {
                    continue; // 自分自身は除外
                }
                
                let new_row = row as isize + i;
                let new_col = col as isize + j;
                
                // 範囲内かチェック
                if new_row >= 0 && new_row < height as isize && new_col >= 0 && new_col < width as isize {
                    let new_index = (new_row as usize) * width + (new_col as usize);
                    neighbors.push(new_index);
                }
            }
        }
        
        neighbors
    }
    
    /// ゲームが勝利条件を満たしているかチェック
    pub fn check_win_condition(&self) -> bool {
        // 全ての安全なセルが公開されているかチェック
        for cell in &self.cells {
            // 地雷でないセルが隠れている場合は未達成
            if !cell.is_mine && cell.state == CellState::Hidden {
                return false;
            }
        }
        
        // 全ての安全セルが公開されている
        true
    }
    
    /// すべての地雷を表示（ゲームオーバー時）
    pub fn reveal_all_mines(&mut self) {
        for cell in &mut self.cells {
            if cell.is_mine {
                if cell.state == CellState::Hidden {
                    cell.state = CellState::Flagged;
                }
            }
        }
    }
    
    /// ボードをリセット
    pub fn reset(&mut self) {
        let config = self.config.clone();
        *self = Self::new(config);
    }
    
    /// 座標からセルのインデックスを取得
    pub fn get_cell_index(&self, x: i32, y: i32) -> Option<usize> {
        let cell_size = self.config.cell_size as i32;
        let width = self.config.width;
        
        // 範囲外チェック
        if x < 0 || y < 0 {
            return None;
        }
        
        let col = x / cell_size;
        let row = y / cell_size;
        
        if col >= width as i32 || row >= self.config.height as i32 {
            return None;
        }
        
        Some((row as usize) * width + (col as usize))
    }
}

impl Default for BoardResource {
    fn default() -> Self {
        Self::new(BoardConfig::default())
    }
} 