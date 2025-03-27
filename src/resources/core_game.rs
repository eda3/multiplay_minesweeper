/**
 * コアゲームリソース
 * 
 * ゲームの進行状態、フェーズ、時間など基本的なゲーム状態を管理するリソース
 */
use wasm_bindgen::JsValue;
use js_sys::Date;

/// ゲームフェーズ（状態）
#[derive(Debug, Clone, PartialEq)]
pub enum GamePhase {
    /// タイトル画面
    Title,
    /// 準備完了、初手待ち
    Ready,
    /// ゲームプレイ中
    Playing,
    /// ゲーム一時停止中
    Paused,
    /// ゲームオーバー（勝敗含む）
    GameOver { win: bool },
}

/// コアゲームリソース
#[derive(Debug, Clone)]
pub struct CoreGameResource {
    /// 現在のゲームフェーズ
    pub phase: GamePhase,
    /// ゲーム開始時間
    pub start_time: Option<f64>,
    /// 経過時間（秒）
    pub elapsed_time: f64,
    /// ゲームスコア
    pub score: u32,
    /// 残り地雷数（表示用）
    pub remaining_mines: i32,
}

impl Default for CoreGameResource {
    fn default() -> Self {
        Self {
            phase: GamePhase::Title,
            start_time: None,
            elapsed_time: 0.0,
            score: 0,
            remaining_mines: 0,
        }
    }
}

impl CoreGameResource {
    /// 新しいコアゲームリソースを作成
    pub fn new() -> Self {
        Self::default()
    }
    
    /// ゲームの初期化
    pub fn initialize(&mut self, mine_count: i32) {
        self.phase = GamePhase::Ready;
        self.start_time = None;
        self.elapsed_time = 0.0;
        self.score = 0;
        self.remaining_mines = mine_count;
    }
    
    /// ゲームの開始
    pub fn start_game(&mut self) {
        self.phase = GamePhase::Playing;
        self.start_time = Some(Date::now());
    }
    
    /// ゲームの一時停止
    pub fn pause_game(&mut self) {
        if self.phase == GamePhase::Playing {
            self.phase = GamePhase::Paused;
        }
    }
    
    /// ゲームの再開
    pub fn resume_game(&mut self) {
        if self.phase == GamePhase::Paused {
            self.phase = GamePhase::Playing;
        }
    }
    
    /// ゲームの終了
    pub fn end_game(&mut self, win: bool) {
        self.phase = GamePhase::GameOver { win };
    }
    
    /// ゲームが開始されているかどうか
    pub fn is_game_started(&self) -> bool {
        matches!(self.phase, GamePhase::Playing | GamePhase::Paused | GamePhase::GameOver { .. })
    }
    
    /// ゲームがプレイ中かどうか
    pub fn is_playing(&self) -> bool {
        self.phase == GamePhase::Playing
    }
    
    /// ゲームがポーズ中かどうか
    pub fn is_paused(&self) -> bool {
        self.phase == GamePhase::Paused
    }
    
    /// ゲームがオーバーかどうか
    pub fn is_game_over(&self) -> bool {
        matches!(self.phase, GamePhase::GameOver { .. })
    }
    
    /// 勝利したかどうか
    pub fn is_win(&self) -> bool {
        matches!(self.phase, GamePhase::GameOver { win: true })
    }
    
    /// 経過時間を更新
    pub fn update_elapsed_time(&mut self) {
        if let Some(start_time) = self.start_time {
            if self.phase == GamePhase::Playing {
                self.elapsed_time = (Date::now() - start_time) / 1000.0;
            }
        }
    }
    
    /// スコアを加算
    pub fn add_score(&mut self, points: u32) {
        self.score += points;
    }
    
    /// 旗を立てた時に残り地雷数を更新
    pub fn update_remaining_mines(&mut self, is_flagged: bool) {
        if is_flagged {
            self.remaining_mines -= 1;
        } else {
            self.remaining_mines += 1;
        }
        
        // 下限を0に
        self.remaining_mines = self.remaining_mines.max(0);
    }
    
    /// 経過時間を文字列で取得（MM:SS形式）
    pub fn elapsed_time_string(&self) -> String {
        let minutes = (self.elapsed_time / 60.0) as u32;
        let seconds = (self.elapsed_time % 60.0) as u32;
        format!("{:02}:{:02}", minutes, seconds)
    }
} 