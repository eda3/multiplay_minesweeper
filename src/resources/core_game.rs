/**
 * コアゲームリソース
 * 
 * ゲームの進行状態、フェーズ、時間など基本的なゲーム状態を管理するリソース
 */
use wasm_bindgen::prelude::*;
use js_sys::Date;
use super::resource_trait::Resource;

/// ゲームの状態を表す列挙型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GamePhase {
    /// ゲーム開始前の準備状態
    Ready,
    /// ゲームプレイ中
    Playing,
    /// 一時停止中
    Paused,
    /// ゲーム終了（勝敗あり）
    GameOver {
        /// ゲームに勝利したかどうか
        win: bool,
    },
}

/// ゲームの核となる状態を管理するリソース
#[derive(Debug, Clone)]
pub struct CoreGameResource {
    /// 現在のゲームフェーズ
    pub phase: GamePhase,
    /// ゲーム開始時刻
    pub start_time: Option<f64>,
    /// 経過時間（ミリ秒）
    pub elapsed_time: f64,
    /// ゲームのスコア
    pub score: u32,
    /// 残りの地雷数
    pub remaining_mines: u32,
    pub is_multiplayer: bool,
}

impl Default for CoreGameResource {
    fn default() -> Self {
        Self::new()
    }
}

impl CoreGameResource {
    /// 新しいゲームリソースを作成
    pub fn new() -> Self {
        Self {
            phase: GamePhase::Ready,
            start_time: None,
            elapsed_time: 0.0,
            score: 0,
            remaining_mines: 0,
            is_multiplayer: false,
        }
    }

    /// ゲームを初期化
    pub fn initialize(&mut self, mine_count: u32) {
        self.phase = GamePhase::Ready;
        self.start_time = None;
        self.elapsed_time = 0.0;
        self.score = 0;
        self.remaining_mines = mine_count;
    }

    /// 現在のゲームフェーズを取得
    pub fn phase(&self) -> GamePhase {
        self.phase
    }

    /// ゲームを開始
    pub fn start_game(&mut self) {
        if self.phase == GamePhase::Ready {
            self.phase = GamePhase::Playing;
            self.start_time = Some(Date::now());
        }
    }

    /// ゲームを一時停止
    pub fn pause_game(&mut self) {
        if self.phase == GamePhase::Playing {
            self.phase = GamePhase::Paused;
            // 経過時間を記録
            self.update_elapsed_time();
        }
    }

    /// ゲームを再開
    pub fn resume_game(&mut self) {
        if self.phase == GamePhase::Paused {
            self.phase = GamePhase::Playing;
            // 開始時間を再設定（すでに経過した時間を考慮）
            self.start_time = Some(Date::now() - self.elapsed_time);
        }
    }

    /// ゲームを終了
    pub fn end_game(&mut self, win: bool) {
        self.update_elapsed_time();
        self.phase = GamePhase::GameOver { win };
    }

    /// ゲームが実行中かどうか
    pub fn is_playing(&self) -> bool {
        matches!(self.phase, GamePhase::Playing)
    }

    /// ゲームが一時停止中かどうか
    pub fn is_paused(&self) -> bool {
        matches!(self.phase, GamePhase::Paused)
    }

    /// ゲームが終了したかどうか
    pub fn is_game_over(&self) -> bool {
        matches!(self.phase, GamePhase::GameOver { .. })
    }

    /// ゲームに勝利したかどうか
    pub fn is_win(&self) -> bool {
        matches!(self.phase, GamePhase::GameOver { win: true })
    }

    /// 経過時間を更新
    pub fn update_elapsed_time(&mut self) {
        if let (Some(start), true) = (self.start_time, self.is_playing()) {
            self.elapsed_time = Date::now() - start;
        }
    }

    /// 経過時間を取得
    pub fn elapsed_time(&self) -> f64 {
        self.elapsed_time
    }

    /// スコアを取得
    pub fn score(&self) -> u32 {
        self.score
    }

    /// スコアを追加
    pub fn add_score(&mut self, points: u32) {
        self.score += points;
    }

    /// 残りの地雷数を取得
    pub fn remaining_mines(&self) -> u32 {
        self.remaining_mines
    }

    /// 残りの地雷数を設定
    pub fn set_remaining_mines(&mut self, count: u32) {
        self.remaining_mines = count;
    }

    /// 旗を立てた時に残りの地雷数を減らす
    pub fn decrement_mines(&mut self) {
        if self.remaining_mines > 0 {
            self.remaining_mines -= 1;
        }
    }

    /// 旗を外した時に残りの地雷数を増やす
    pub fn increment_mines(&mut self) {
        self.remaining_mines += 1;
    }

    /// 経過時間を文字列で取得（MM:SS形式）
    pub fn format_elapsed_time(&self) -> String {
        let total_seconds = (self.elapsed_time / 1000.0) as u32;
        let minutes = total_seconds / 60;
        let seconds = total_seconds % 60;
        format!("{:02}:{:02}", minutes, seconds)
    }

    pub fn set_phase(&mut self, phase: GamePhase) {
        self.phase = phase;
    }

    pub fn is_game_active(&self) -> bool {
        matches!(self.phase, GamePhase::Playing)
    }

    pub fn is_game_over(&self) -> bool {
        matches!(self.phase, GamePhase::GameOver { .. })
    }
}

// Resourceトレイトの実装
impl Resource for CoreGameResource {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_resource_has_default_values() {
        let resource = CoreGameResource::new();
        assert_eq!(resource.phase, GamePhase::Ready);
        assert_eq!(resource.start_time, None);
        assert_eq!(resource.elapsed_time, 0.0);
        assert_eq!(resource.score, 0);
        assert_eq!(resource.remaining_mines, 0);
    }

    #[test]
    fn test_initialize_sets_correct_values() {
        let mut resource = CoreGameResource::new();
        resource.initialize(10);
        assert_eq!(resource.phase, GamePhase::Ready);
        assert_eq!(resource.start_time, None);
        assert_eq!(resource.elapsed_time, 0.0);
        assert_eq!(resource.score, 0);
        assert_eq!(resource.remaining_mines, 10);
    }

    #[test]
    fn test_game_phase_transitions() {
        let mut resource = CoreGameResource::new();
        resource.initialize(10);
        
        // Ready -> Playing
        resource.start_game();
        assert!(resource.is_playing());
        assert!(resource.start_time.is_some());
        
        // Playing -> Paused
        resource.pause_game();
        assert!(resource.is_paused());
        
        // Paused -> Playing
        resource.resume_game();
        assert!(resource.is_playing());
        
        // Playing -> GameOver(win)
        resource.end_game(true);
        assert!(resource.is_game_over());
        assert!(resource.is_win());
        
        // GameOver -> Ready (新しいゲーム)
        resource.initialize(15);
        assert_eq!(resource.phase, GamePhase::Ready);
        assert_eq!(resource.remaining_mines, 15);
    }
} 