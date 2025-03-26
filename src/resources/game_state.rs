/**
 * ゲーム状態リソース
 * 
 * ゲームの進行状態を管理するリソース
 */
use std::time::Duration;
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

/// ゲーム状態リソース
#[derive(Debug, Clone)]
pub struct GameState {
    /// 現在のゲームフェーズ
    pub phase: GamePhase,
    /// ゲーム開始時間
    pub start_time: Option<f64>,
    /// 経過時間（秒）
    pub elapsed_time: f64,
    /// ローカルプレイヤーID
    pub local_player_id: Option<String>,
    /// 最終更新時間
    pub last_update_time: f64,
    /// フレームカウンター
    pub frame_count: u64,
    /// 前回のフレーム時間
    pub last_frame_time: f64,
}

impl Default for GameState {
    fn default() -> Self {
        Self {
            phase: GamePhase::Title,
            start_time: None,
            elapsed_time: 0.0,
            local_player_id: None,
            last_update_time: Date::now(),
            frame_count: 0,
            last_frame_time: Date::now(),
        }
    }
}

impl GameState {
    /// 新しいゲーム状態を作成
    pub fn new() -> Self {
        Self::default()
    }
    
    /// ゲームの初期化
    pub fn initialize(&mut self) {
        self.phase = GamePhase::Ready;
        self.start_time = None;
        self.elapsed_time = 0.0;
        self.frame_count = 0;
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
    
    /// フレーム情報を更新
    pub fn update_frame(&mut self) -> Result<f64, JsValue> {
        let now = Date::now();
        let delta_time = (now - self.last_frame_time) / 1000.0; // 秒単位
        
        self.frame_count += 1;
        self.last_frame_time = now;
        self.last_update_time = now;
        
        // 経過時間も更新
        self.update_elapsed_time();
        
        Ok(delta_time)
    }
    
    /// 経過時間を文字列で取得（MM:SS形式）
    pub fn elapsed_time_string(&self) -> String {
        let minutes = (self.elapsed_time / 60.0) as u32;
        let seconds = (self.elapsed_time % 60.0) as u32;
        format!("{:02}:{:02}", minutes, seconds)
    }
    
    /// 現在のFPS（フレームレート）を計算
    pub fn current_fps(&self) -> f64 {
        if self.frame_count <= 1 {
            return 0.0;
        }
        
        let time_diff = self.last_frame_time - (self.start_time.unwrap_or(self.last_frame_time));
        if time_diff <= 0.0 {
            return 0.0;
        }
        
        (self.frame_count as f64 - 1.0) / (time_diff / 1000.0)
    }
} 