/**
 * ボードに関するリソース定義
 */
use std::collections::HashMap;
use crate::components::{CellStateComponent, CellContentComponent, EntityId};
use crate::models::CellValue;
use super::resource_trait::Resource;

/**
 * ボード設定リソース
 * 
 * ボードの幅、高さ、地雷数などの設定情報を保持します。
 */
#[derive(Debug, Clone)]
pub struct BoardConfigResource {
    /// ボードの幅（列数）
    pub width: usize,
    /// ボードの高さ（行数）
    pub height: usize,
    /// 地雷の総数
    pub mine_count: usize,
    /// 最初のクリックで地雷に当たらないようにするかどうか
    pub safe_first_click: bool,
}

impl Resource for BoardConfigResource {}

impl Default for BoardConfigResource {
    fn default() -> Self {
        Self {
            width: 10,
            height: 10,
            mine_count: 10,
            safe_first_click: true,
        }
    }
}

impl BoardConfigResource {
    pub fn new(width: usize, height: usize, mine_count: usize, safe_first_click: bool) -> Self {
        Self {
            width,
            height,
            mine_count,
            safe_first_click,
        }
    }
    
    /// セルの総数を計算
    pub fn total_cells(&self) -> usize {
        self.width * self.height
    }
    
    /// 座標が有効かどうかを判定
    pub fn is_valid_position(&self, row: usize, col: usize) -> bool {
        row < self.height && col < self.width
    }
}

/**
 * ボード状態リソース
 * 
 * ボードの現在の状態（初期化済みか、ゲームオーバーか、勝利状態か）を保持します。
 * またセルエンティティのIDマップも管理します。
 */
#[derive(Debug, Clone)]
pub struct BoardStateResource {
    pub is_initialized: bool,
    pub is_game_over: bool,
    pub is_win: bool,
    pub first_click: bool,
    // グリッド位置（row, col）からエンティティIDへのマッピング
    pub cell_grid: HashMap<(usize, usize), EntityId>,
    // 残りの未公開かつ地雷でないセル数（勝利条件のチェックに使用）
    pub remaining_safe_cells: usize,
    // フラグを立てたセルの数
    pub flagged_count: usize,
}

impl Resource for BoardStateResource {}

impl Default for BoardStateResource {
    fn default() -> Self {
        Self {
            is_initialized: false,
            is_game_over: false,
            is_win: false,
            first_click: true,
            cell_grid: HashMap::new(),
            remaining_safe_cells: 0,
            flagged_count: 0,
        }
    }
}

impl BoardStateResource {
    pub fn new() -> Self {
        Self::default()
    }
    
    /// セルのエンティティIDを取得
    pub fn get_cell_entity(&self, row: usize, col: usize) -> Option<EntityId> {
        self.cell_grid.get(&(row, col)).copied()
    }
    
    /// セルの周囲8方向の座標を取得
    pub fn get_adjacent_cells(&self, row: usize, col: usize, 
                             board_width: usize, board_height: usize) -> Vec<(usize, usize)> {
        let mut adjacent_cells = Vec::new();
        
        // 周囲8方向の座標オフセット
        let offsets = [
            (-1, -1), (-1, 0), (-1, 1),
            (0, -1),           (0, 1),
            (1, -1),  (1, 0),  (1, 1)
        ];
        
        for (row_offset, col_offset) in offsets.iter() {
            let new_row = row as isize + row_offset;
            let new_col = col as isize + col_offset;
            
            // 範囲内の座標のみを追加
            if new_row >= 0 && new_row < board_height as isize &&
               new_col >= 0 && new_col < board_width as isize {
                adjacent_cells.push((new_row as usize, new_col as usize));
            }
        }
        
        adjacent_cells
    }
    
    /// セルの周囲8方向の座標を取得（BoardConfigResourceを使用）
    pub fn get_adjacent_positions(&self, row: usize, col: usize, 
                                board_config: &BoardConfigResource) -> Vec<(usize, usize)> {
        self.get_adjacent_cells(row, col, board_config.width, board_config.height)
    }
    
    /// グリッドを初期化する
    pub fn initialize_grid(&mut self, width: usize, height: usize) {
        // グリッドをクリア
        self.cell_grid.clear();
        
        // 初期化フラグを設定
        self.is_initialized = true;
    }
} 