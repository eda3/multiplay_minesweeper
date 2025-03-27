/**
 * ゲーム設定リソース
 * 
 * マインスイーパーゲームの設定を管理するリソース
 */
use wasm_bindgen::prelude::*;
use js_sys::Date;

/// ゲームの難易度
#[derive(Debug, Clone, PartialEq)]
pub enum Difficulty {
    /// 初級（9x9、10地雷）
    Easy,
    /// 中級（16x16、40地雷）
    Medium,
    /// 上級（30x16、99地雷）
    Hard,
    /// カスタム
    Custom,
}

/// ボードの設定
#[derive(Debug, Clone)]
pub struct BoardConfig {
    /// ボードの幅
    pub width: usize,
    /// ボードの高さ
    pub height: usize,
    /// 地雷の数
    pub mine_count: usize,
    /// セルのサイズ（ピクセル）
    pub cell_size: f64,
}

impl BoardConfig {
    /// 新しいBoardConfigインスタンスを作成
    pub fn new(width: usize, height: usize, mine_count: usize, cell_size: f64) -> Self {
        // 最小値の制約を適用
        let width = width.max(5);
        let height = height.max(5);
        // 総セル数を超えないように地雷数を調整
        let max_mines = width * height - 9; // 初手で開ける領域を確保
        let mine_count = mine_count.min(max_mines);
        
        Self {
            width,
            height,
            mine_count,
            cell_size,
        }
    }

    /// 総セル数を取得
    pub fn total_cells(&self) -> usize {
        self.width * self.height
    }

    /// 地雷の密度を取得
    pub fn mine_ratio(&self) -> f64 {
        self.mine_count as f64 / self.total_cells() as f64
    }

    /// セルサイズを更新
    pub fn update_cell_size(&mut self, cell_size: f64) {
        self.cell_size = cell_size;
    }
}

/// ゲーム設定リソース
/// ボード設定やゲームルールなどのプレイ設定を管理
#[derive(Debug, Clone)]
pub struct GameConfigResource {
    /// ボード設定
    pub board_config: BoardConfig,
    /// 自動フラグ機能の使用有無
    pub auto_flag: bool,
    /// 初手が地雷になることを防ぐかどうか
    pub first_click_safe: bool,
    /// セルを全て明らかにすることで勝利するか、フラグを立てるだけでもよいか
    pub win_by_revealing: bool,
    /// タイマーを使用するかどうか
    pub use_timer: bool,
    /// 最大スコア
    pub max_score: u32,
    /// マルチプレイヤーモードかどうか
    pub multiplayer: bool,
    /// 難易度設定
    pub difficulty: Difficulty,
}

impl GameConfigResource {
    /// 新しいGameConfigResourceインスタンスを作成
    pub fn new() -> Self {
        Self {
            board_config: BoardConfig::new(9, 9, 10, 30.0), // デフォルトは初級
            auto_flag: false,
            first_click_safe: true,
            win_by_revealing: true,
            use_timer: true,
            max_score: 10000,
            multiplayer: true,
            difficulty: Difficulty::Easy,
        }
    }

    /// 難易度を設定
    pub fn set_difficulty(&mut self, difficulty: Difficulty) {
        let (width, height, mine_count) = match difficulty {
            Difficulty::Easy => (9, 9, 10),
            Difficulty::Medium => (16, 16, 40),
            Difficulty::Hard => (30, 16, 99),
            Difficulty::Custom => (
                self.board_config.width,
                self.board_config.height,
                self.board_config.mine_count
            ),
        };
        
        self.board_config = BoardConfig::new(
            width,
            height,
            mine_count,
            self.board_config.cell_size,
        );
        self.difficulty = difficulty;
    }

    /// カスタムボードを設定
    pub fn set_custom_board(&mut self, width: usize, height: usize, mine_count: usize) {
        self.board_config = BoardConfig::new(
            width,
            height,
            mine_count,
            self.board_config.cell_size,
        );
    }

    /// キャンバスサイズに基づいてセルサイズを更新
    pub fn update_cell_size(&mut self, canvas_width: f64, canvas_height: f64) {
        let width_based_size = canvas_width / self.board_config.width as f64;
        let height_based_size = canvas_height / self.board_config.height as f64;
        
        // 小さい方のサイズを選択して、ボード全体が画面に収まるようにする
        let cell_size = width_based_size.min(height_based_size).max(15.0).min(50.0);
        self.board_config.update_cell_size(cell_size);
    }

    /// 総セル数を取得
    pub fn total_cells(&self) -> usize {
        self.board_config.total_cells()
    }

    /// 地雷の密度を取得
    pub fn mine_ratio(&self) -> f64 {
        self.board_config.mine_ratio()
    }

    /// ランダムシードを生成
    pub fn get_random_seed(&self) -> u64 {
        let now = Date::now();
        let seed = (now * 1000.0).floor() as u64;
        
        // ボード設定に基づいて追加のハッシュを作成
        let additional = self.board_config.width as u64 * 31 +
            self.board_config.height as u64 * 17 +
            self.board_config.mine_count as u64 * 13;
        
        seed.wrapping_add(additional)
    }

    /// 現在の設定でスコアを計算する
    /// 
    /// ゲームの難易度、ボードサイズ、経過時間に基づいてスコアを計算
    pub fn calculate_score(&self, elapsed_time: f64, win: bool) -> u32 {
        if !win {
            return 0;
        }
        
        // 基本スコア（難易度による）
        let base_score = match self.difficulty {
            Difficulty::Easy => 100,
            Difficulty::Medium => 200,
            Difficulty::Hard => 500,
            Difficulty::Custom => 300,
        };
        
        // 時間ボーナス（早いほど高い）
        let time_bonus = if elapsed_time > 0.0 {
            let seconds = elapsed_time / 1000.0;
            (5000.0 / seconds).min(1000.0) as u32
        } else {
            0
        };
        
        // ボードサイズ・地雷数による追加ポイント
        let additional = self.board_config.width as u64 * 31 +
            self.board_config.height as u64 * 17 +
            self.board_config.mine_count as u64 * 13;
        
        base_score + time_bonus + additional as u32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_board_config() {
        let config = BoardConfig::new(10, 10, 20, 30.0);
        assert_eq!(config.width, 10);
        assert_eq!(config.height, 10);
        assert_eq!(config.mine_count, 20);
        assert_eq!(config.cell_size, 30.0);
        assert_eq!(config.total_cells(), 100);
        assert_eq!(config.mine_ratio(), 0.2);
    }

    #[test]
    fn test_mine_count_limits() {
        // 地雷数が多すぎる場合は制限される
        let config = BoardConfig::new(10, 10, 100, 30.0);
        assert_eq!(config.mine_count, 91); // 100 - 9 = 91 (最大地雷数)
    }

    #[test]
    fn test_set_difficulty() {
        let mut config = GameConfigResource::new();
        
        // 初級設定
        config.set_difficulty(Difficulty::Easy);
        assert_eq!(config.board_config.width, 9);
        assert_eq!(config.board_config.height, 9);
        assert_eq!(config.board_config.mine_count, 10);
        
        // 中級設定
        config.set_difficulty(Difficulty::Medium);
        assert_eq!(config.board_config.width, 16);
        assert_eq!(config.board_config.height, 16);
        assert_eq!(config.board_config.mine_count, 40);
        
        // 上級設定
        config.set_difficulty(Difficulty::Hard);
        assert_eq!(config.board_config.width, 30);
        assert_eq!(config.board_config.height, 16);
        assert_eq!(config.board_config.mine_count, 99);
    }

    #[test]
    fn test_score_calculation() {
        let config = GameConfigResource::new();
        
        // 基本ケース
        let score1 = config.calculate_score(60.0, true);
        
        // 時間がかかるとスコアが下がる
        let score2 = config.calculate_score(120.0, true);
        assert!(score1 > score2);
    }
} 