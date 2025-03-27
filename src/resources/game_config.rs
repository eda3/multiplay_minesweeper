/**
 * ゲーム設定リソース
 * 
 * マインスイーパーゲームの設定を管理するリソース
 */
use crate::resources::board_config::{BoardConfig, Difficulty};

/// ゲーム設定リソース
#[derive(Debug, Clone)]
pub struct GameConfigResource {
    /// ボード設定
    pub board_config: BoardConfig,
    /// 自動フラグ機能の有効/無効
    pub auto_flag: bool,
    /// 初回クリック保護（初回クリックでは必ず安全なセルが開くようにする）
    pub first_click_safe: bool,
    /// 勝利条件（true: 全非地雷セルを開く, false: 全地雷にフラグを立てる）
    pub win_by_revealing: bool,
    /// タイマーの使用
    pub use_timer: bool,
    /// 最大スコア
    pub max_score: u32,
    /// マルチプレイ設定
    pub multiplayer_enabled: bool,
}

impl Default for GameConfigResource {
    fn default() -> Self {
        Self {
            board_config: BoardConfig::default(),
            auto_flag: false,
            first_click_safe: true,
            win_by_revealing: true,
            use_timer: true,
            max_score: 999,
            multiplayer_enabled: true,
        }
    }
}

impl GameConfigResource {
    /// 新しいゲーム設定リソースを作成
    pub fn new() -> Self {
        Self::default()
    }
    
    /// 難易度を設定
    pub fn set_difficulty(&mut self, difficulty: Difficulty) {
        self.board_config = match difficulty {
            Difficulty::Beginner => BoardConfig::beginner(),
            Difficulty::Intermediate => BoardConfig::intermediate(),
            Difficulty::Expert => BoardConfig::expert(),
            Difficulty::Custom => self.board_config.clone(),
        };
    }
    
    /// カスタム設定を適用
    pub fn set_custom_board(&mut self, width: usize, height: usize, mine_count: usize) {
        self.board_config = BoardConfig::custom(width, height, mine_count);
    }
    
    /// キャンバスサイズに合わせてセルサイズを調整
    pub fn update_cell_size(&mut self, canvas_width: f64, canvas_height: f64) {
        self.board_config.update_cell_size(canvas_width, canvas_height);
    }
    
    /// 全セル数を取得
    pub fn total_cells(&self) -> usize {
        self.board_config.total_cells()
    }
    
    /// 地雷の比率を取得
    pub fn mine_ratio(&self) -> f64 {
        self.board_config.mine_ratio()
    }
    
    /// ランダムシード値を取得（時間などに基づく）
    pub fn get_random_seed(&self) -> u64 {
        use std::time::{SystemTime, UNIX_EPOCH};
        
        // 現在のUnixタイムスタンプをシードとして使用
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
            
        // ボードのサイズなども含める
        now ^ (self.board_config.width as u64 * 13)
            ^ (self.board_config.height as u64 * 17)
            ^ (self.board_config.mine_count as u64 * 31)
    }
    
    /// スコア計算（残り時間や残りフラグなどに基づく）
    pub fn calculate_score(&self, cleared_cells: usize, time_taken: f64, flags_used: usize) -> u32 {
        if !self.use_timer {
            return 0;
        }
        
        // 基本スコア = クリアしたセル数 × 10
        let base_score = cleared_cells as u32 * 10;
        
        // 時間係数（長い時間かかるほど低くなる）
        let time_factor = if time_taken <= 0.0 {
            1.0
        } else {
            let max_time = 999.0; // 最大計測時間（秒）
            1.0 - (time_taken / max_time).min(0.9) // 最低でも0.1は残す
        };
        
        // フラグボーナス（正確に使うほど高い）
        let flag_accuracy = if flags_used > 0 && self.board_config.mine_count > 0 {
            let used_ratio = flags_used as f64 / self.board_config.mine_count as f64;
            // 地雷数と同じフラグ数で最大ボーナス
            if used_ratio <= 1.0 {
                used_ratio
            } else {
                // 余分にフラグを使うとボーナス減少
                2.0 - used_ratio
            }
        } else {
            0.0
        };
        
        // 最終スコア = 基本スコア × 時間係数 × (1 + フラグボーナス)
        let score = (base_score as f64 * time_factor * (1.0 + flag_accuracy * 0.5)) as u32;
        
        // 最大スコアを超えないように
        score.min(self.max_score)
    }
} 