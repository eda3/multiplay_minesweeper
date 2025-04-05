/**
 * ゲーム状態リソース
 * 
 * ゲームの現在の状態と進行状況を管理する
 */
use super::resource_trait::Resource;
use std::{fmt::{self, Debug}, time::Duration, collections::HashMap};

/// ゲームの段階
#[derive(Debug, Clone, PartialEq)]
pub enum GamePhase {
    /// スタート画面
    StartScreen,
    /// 準備中
    Loading,
    /// ゲームプレイ中
    Playing,
    /// 一時停止中
    Paused,
    /// ゲームオーバー（勝利または敗北）
    GameOver {
        /// 勝利したかどうか
        win: bool,
        /// スコア
        score: u32,
        /// 所要時間
        time: Duration,
    },
}

impl Default for GamePhase {
    fn default() -> Self {
        Self::StartScreen
    }
}

/// 難易度レベル
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DifficultyLevel {
    /// 初級
    Beginner,
    /// 中級
    Intermediate,
    /// 上級
    Expert,
    /// カスタム
    Custom,
}

impl fmt::Display for DifficultyLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Beginner => write!(f, "初級"),
            Self::Intermediate => write!(f, "中級"),
            Self::Expert => write!(f, "上級"),
            Self::Custom => write!(f, "カスタム"),
        }
    }
}

impl Default for DifficultyLevel {
    fn default() -> Self {
        Self::Intermediate
    }
}

/// ゲームモード
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameMode {
    /// シングルプレイヤー
    SinglePlayer,
    /// マルチプレイヤー
    MultiPlayer,
}

impl Default for GameMode {
    fn default() -> Self {
        Self::SinglePlayer
    }
}

/// ゲーム状態リソース
#[derive(Debug, Clone)]
pub struct GameStateResource {
    /// 現在のゲームフェーズ
    pub phase: GamePhase,
    /// 難易度レベル
    pub difficulty: DifficultyLevel,
    /// ゲームモード
    pub mode: GameMode,
    /// 現在のスコア
    pub score: u32,
    /// ハイスコア
    pub high_score: u32,
    /// ゲーム開始時刻（ミリ秒）
    pub start_time: f64,
    /// ゲーム経過時間（ミリ秒）
    pub elapsed_time: f64,
    /// 一時停止開始時刻（ミリ秒）
    pub pause_start_time: Option<f64>,
    /// 累積一時停止時間（ミリ秒）
    pub total_pause_time: f64,
    /// デバッグモードかどうか
    pub debug_mode: bool,
}

impl Default for GameStateResource {
    fn default() -> Self {
        Self {
            phase: GamePhase::Loading,
            difficulty: DifficultyLevel::default(),
            mode: GameMode::default(),
            score: 0,
            high_score: 0,
            start_time: 0.0,
            elapsed_time: 0.0,
            pause_start_time: None,
            total_pause_time: 0.0,
            debug_mode: false,
        }
    }
}

impl GameStateResource {
    /// 新しいゲーム状態リソースを作成
    pub fn new() -> Self {
        Self::default()
    }
    
    /// ゲームをスタート
    pub fn start_game(&mut self) {
        self.phase = GamePhase::Playing;
        self.score = 0;
        self.start_time = js_sys::Date::now();
        self.elapsed_time = 0.0;
        self.pause_start_time = None;
        self.total_pause_time = 0.0;
    }
    
    /// ゲームを一時停止
    pub fn pause_game(&mut self) {
        if self.phase == GamePhase::Playing {
            self.phase = GamePhase::Paused;
            self.pause_start_time = Some(js_sys::Date::now());
        }
    }
    
    /// ゲームを再開
    pub fn resume_game(&mut self) {
        if self.phase == GamePhase::Paused {
            self.phase = GamePhase::Playing;
            if let Some(pause_time) = self.pause_start_time {
                self.total_pause_time += js_sys::Date::now() - pause_time;
                self.pause_start_time = None;
            }
        }
    }
    
    /// ゲームオーバーを設定
    pub fn set_game_over(&mut self, win: bool) {
        let time = if win {
            self.update_elapsed_time();
            Duration::from_millis(self.elapsed_time as u64)
        } else {
            Duration::from_millis(self.elapsed_time as u64)
        };
        
        self.phase = GamePhase::GameOver { win, score: self.score, time };
        
        // ハイスコア更新
        if win && self.score > self.high_score {
            self.high_score = self.score;
        }
    }
    
    /// 経過時間を更新
    pub fn update_elapsed_time(&mut self) {
        let now = js_sys::Date::now();
        
        // 一時停止中なら時間は進まない
        if self.phase == GamePhase::Paused {
            return;
        }
        
        match self.phase {
            GamePhase::Playing => {
                self.elapsed_time = now - self.start_time - self.total_pause_time;
            },
            GamePhase::GameOver { .. } => {
                // ゲームオーバー後は時間を更新しない
            },
            _ => {
                // その他の状態では経過時間をリセット
                self.elapsed_time = 0.0;
            }
        }
    }
    
    /// スコアを更新
    pub fn add_score(&mut self, points: u32) {
        self.score += points;
    }
    
    /// 難易度を設定
    pub fn set_difficulty(&mut self, difficulty: DifficultyLevel) {
        self.difficulty = difficulty;
    }
    
    /// ゲームモードを設定
    pub fn set_mode(&mut self, mode: GameMode) {
        self.mode = mode;
    }
    
    /// デバッグモードの切り替え
    pub fn toggle_debug_mode(&mut self) {
        self.debug_mode = !self.debug_mode;
    }
    
    /// ゲームがプレイ中かどうか
    pub fn is_playing(&self) -> bool {
        self.phase == GamePhase::Playing
    }
    
    /// ゲームが一時停止中かどうか
    pub fn is_paused(&self) -> bool {
        self.phase == GamePhase::Paused
    }
    
    /// ゲームがスタート画面かどうか
    pub fn is_start_screen(&self) -> bool {
        self.phase == GamePhase::StartScreen
    }
    
    /// ゲームがロード中かどうか
    pub fn is_loading(&self) -> bool {
        self.phase == GamePhase::Loading
    }
    
    /// ゲームがゲームオーバーかどうか
    pub fn is_game_over(&self) -> bool {
        matches!(self.phase, GamePhase::GameOver { .. })
    }
    
    /// プレイヤーが勝利したかどうか
    pub fn is_win(&self) -> bool {
        matches!(self.phase, GamePhase::GameOver { win: true, .. })
    }
    
    /// プレイヤーが敗北したかどうか
    pub fn is_loss(&self) -> bool {
        matches!(self.phase, GamePhase::GameOver { win: false, .. })
    }
    
    /// ゲームの所要時間を取得
    pub fn get_game_time(&self) -> Duration {
        match self.phase {
            GamePhase::GameOver { time, .. } => time,
            _ => Duration::from_millis(self.elapsed_time as u64),
        }
    }
    
    /// ゲームのフェーズ名を取得
    pub fn get_phase_name(&self) -> &'static str {
        match self.phase {
            GamePhase::StartScreen => "スタート画面",
            GamePhase::Loading => "ロード中",
            GamePhase::Playing => "プレイ中",
            GamePhase::Paused => "一時停止",
            GamePhase::GameOver { win: true, .. } => "勝利！",
            GamePhase::GameOver { win: false, .. } => "ゲームオーバー",
        }
    }

    /// カスタムボード設定
    pub fn set_custom_board(&mut self, width: usize, height: usize, mines: usize) {
        self.difficulty = DifficultyLevel::Custom;
        // 必要ならBoardResourceも更新
    }

    /// セルサイズを更新
    pub fn update_cell_size(&mut self, canvas_width: f64, canvas_height: f64) {
        // キャンバスサイズに合わせてセルサイズを更新
    }
}

#[derive(Debug, Clone)]
pub struct TimeResource {
    pub delta_time: f64,
    pub total_time: f64,
}

impl TimeResource {
    pub fn new() -> Self {
        Self {
            delta_time: 0.0,
            total_time: 0.0,
        }
    }

    pub fn update(&mut self, delta_time: f64) {
        self.delta_time = delta_time;
        self.total_time += delta_time;
    }
}

#[derive(Debug, Clone)]
pub struct PlayerStateResource {
    // ... existing code ...
}

impl PlayerStateResource {
    // ... existing code ...
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_resource_has_default_values() {
        let resource = GameStateResource::new();
        assert_eq!(resource.phase, GamePhase::StartScreen);
        assert_eq!(resource.difficulty, DifficultyLevel::Intermediate);
        assert_eq!(resource.mode, GameMode::SinglePlayer);
        assert_eq!(resource.score, 0);
        assert_eq!(resource.high_score, 0);
        assert_eq!(resource.start_time, 0.0);
        assert_eq!(resource.elapsed_time, 0.0);
        assert_eq!(resource.pause_start_time, None);
        assert_eq!(resource.total_pause_time, 0.0);
        assert_eq!(resource.debug_mode, false);
    }

    #[test]
    fn test_start_game_sets_correct_values() {
        let mut resource = GameStateResource::new();
        resource.start_game();
        assert_eq!(resource.phase, GamePhase::Playing);
        assert_eq!(resource.score, 0);
        assert_eq!(resource.start_time, js_sys::Date::now());
        assert_eq!(resource.elapsed_time, 0.0);
        assert_eq!(resource.pause_start_time, None);
        assert_eq!(resource.total_pause_time, 0.0);
    }

    #[test]
    fn test_pause_game_sets_correct_values() {
        let mut resource = GameStateResource::new();
        resource.start_game();
        resource.pause_game();
        assert_eq!(resource.phase, GamePhase::Paused);
        assert_eq!(resource.pause_start_time, Some(js_sys::Date::now()));
    }

    #[test]
    fn test_resume_game_sets_correct_values() {
        let mut resource = GameStateResource::new();
        resource.start_game();
        resource.pause_game();
        resource.resume_game();
        assert_eq!(resource.phase, GamePhase::Playing);
        assert_eq!(resource.pause_start_time, None);
    }

    #[test]
    fn test_set_game_over_updates_correct_values() {
        let mut resource = GameStateResource::new();
        resource.start_game();
        resource.set_game_over(true);
        assert_eq!(resource.phase, GamePhase::GameOver { win: true, score: 0, time: Duration::from_millis(0) });
        assert_eq!(resource.high_score, 0);
    }

    #[test]
    fn test_update_elapsed_time_updates_correct_values() {
        let mut resource = GameStateResource::new();
        resource.start_game();
        resource.update_elapsed_time();
        assert_eq!(resource.elapsed_time, js_sys::Date::now() - resource.start_time);
    }

    #[test]
    fn test_add_score_updates_correct_values() {
        let mut resource = GameStateResource::new();
        resource.start_game();
        resource.add_score(10);
        assert_eq!(resource.score, 10);
    }

    #[test]
    fn test_set_difficulty_updates_correct_values() {
        let mut resource = GameStateResource::new();
        resource.set_difficulty(DifficultyLevel::Expert);
        assert_eq!(resource.difficulty, DifficultyLevel::Expert);
    }

    #[test]
    fn test_set_mode_updates_correct_values() {
        let mut resource = GameStateResource::new();
        resource.set_mode(GameMode::MultiPlayer);
        assert_eq!(resource.mode, GameMode::MultiPlayer);
    }

    #[test]
    fn test_toggle_debug_mode_updates_correct_values() {
        let mut resource = GameStateResource::new();
        resource.toggle_debug_mode();
        assert_eq!(resource.debug_mode, true);
    }

    #[test]
    fn test_is_playing_returns_correct_value() {
        let mut resource = GameStateResource::new();
        resource.start_game();
        assert!(resource.is_playing());
        resource.pause_game();
        assert!(!resource.is_playing());
    }

    #[test]
    fn test_is_paused_returns_correct_value() {
        let mut resource = GameStateResource::new();
        resource.start_game();
        assert!(!resource.is_paused());
        resource.pause_game();
        assert!(resource.is_paused());
    }

    #[test]
    fn test_is_start_screen_returns_correct_value() {
        let mut resource = GameStateResource::new();
        assert!(resource.is_start_screen());
        resource.start_game();
        assert!(!resource.is_start_screen());
    }

    #[test]
    fn test_is_loading_returns_correct_value() {
        let mut resource = GameStateResource::new();
        assert!(resource.is_loading());
        resource.start_game();
        assert!(!resource.is_loading());
    }

    #[test]
    fn test_is_game_over_returns_correct_value() {
        let mut resource = GameStateResource::new();
        resource.start_game();
        assert!(!resource.is_game_over());
        resource.set_game_over(true);
        assert!(resource.is_game_over());
    }

    #[test]
    fn test_is_win_returns_correct_value() {
        let mut resource = GameStateResource::new();
        resource.start_game();
        assert!(!resource.is_win());
        resource.set_game_over(true);
        assert!(resource.is_win());
    }

    #[test]
    fn test_is_loss_returns_correct_value() {
        let mut resource = GameStateResource::new();
        resource.start_game();
        assert!(!resource.is_loss());
        resource.set_game_over(false);
        assert!(resource.is_loss());
    }

    #[test]
    fn test_get_game_time_returns_correct_value() {
        let mut resource = GameStateResource::new();
        resource.start_game();
        assert_eq!(resource.get_game_time(), Duration::from_millis(0));
        resource.update_elapsed_time();
        assert_eq!(resource.get_game_time(), Duration::from_millis(resource.elapsed_time as u64));
    }

    #[test]
    fn test_get_phase_name_returns_correct_value() {
        let resource = GameStateResource::new();
        assert_eq!(resource.get_phase_name(), "スタート画面");
        let mut resource = GameStateResource::new();
        resource.start_game();
        assert_eq!(resource.get_phase_name(), "プレイ中");
        let mut resource = GameStateResource::new();
        resource.set_game_over(true);
        assert_eq!(resource.get_phase_name(), "勝利！");
    }
} 