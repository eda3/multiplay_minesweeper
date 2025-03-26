/**
 * ボード設定リソース
 * 
 * マインスイーパーのボードに関する設定を管理するリソース
 */

/// ボードの難易度
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Difficulty {
    /// 初級（9x9、地雷10個）
    Beginner,
    /// 中級（16x16、地雷40個）
    Intermediate,
    /// 上級（30x16、地雷99個）
    Expert,
    /// カスタム
    Custom,
}

/// ボード設定リソース
#[derive(Debug, Clone)]
pub struct BoardConfig {
    /// ボードの幅（セル数）
    pub width: usize,
    /// ボードの高さ（セル数）
    pub height: usize,
    /// 地雷の数
    pub mine_count: usize,
    /// セルのサイズ（ピクセル）
    pub cell_size: f64,
    /// 難易度設定
    pub difficulty: Difficulty,
}

impl Default for BoardConfig {
    fn default() -> Self {
        // デフォルトは中級難易度
        Self::intermediate()
    }
}

impl BoardConfig {
    /// 初級難易度の設定を作成
    pub fn beginner() -> Self {
        Self {
            width: 9,
            height: 9,
            mine_count: 10,
            cell_size: 40.0,
            difficulty: Difficulty::Beginner,
        }
    }
    
    /// 中級難易度の設定を作成
    pub fn intermediate() -> Self {
        Self {
            width: 16,
            height: 16,
            mine_count: 40,
            cell_size: 30.0,
            difficulty: Difficulty::Intermediate,
        }
    }
    
    /// 上級難易度の設定を作成
    pub fn expert() -> Self {
        Self {
            width: 30,
            height: 16,
            mine_count: 99,
            cell_size: 25.0,
            difficulty: Difficulty::Expert,
        }
    }
    
    /// カスタム設定を作成
    pub fn custom(width: usize, height: usize, mine_count: usize) -> Self {
        Self {
            width,
            height,
            mine_count,
            cell_size: 30.0,
            difficulty: Difficulty::Custom,
        }
    }
    
    /// セルサイズを計算して更新（キャンバスサイズに合わせる）
    pub fn update_cell_size(&mut self, canvas_width: f64, canvas_height: f64) {
        // 余白を確保（各端20ピクセル）
        let available_width = canvas_width - 40.0;
        let available_height = canvas_height - 40.0;
        
        // 幅と高さに合わせてセルサイズを計算
        let cell_size_width = available_width / self.width as f64;
        let cell_size_height = available_height / self.height as f64;
        
        // 小さい方を採用（はみ出し防止）
        self.cell_size = cell_size_width.min(cell_size_height);
    }
    
    /// 総セル数を取得
    pub fn total_cells(&self) -> usize {
        self.width * self.height
    }
    
    /// セル数と地雷の比率を取得（0.0 - 1.0）
    pub fn mine_ratio(&self) -> f64 {
        self.mine_count as f64 / self.total_cells() as f64
    }
} 