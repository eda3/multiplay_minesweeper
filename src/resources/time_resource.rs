/**
 * 時間リソース
 * 
 * ゲーム内の時間管理を行うリソース
 */

use std::time::{Duration, Instant};

/// 時間管理リソース
#[derive(Debug, Clone)]
pub struct TimeResource {
    /// ゲーム開始時刻
    pub start_time: Option<Instant>,
    /// 最後のフレーム時刻
    pub last_frame_time: Instant,
    /// フレーム間の経過時間（秒）
    pub delta_time: f64,
    /// 合計経過時間（秒）
    pub total_time: f64,
    /// 前回のティック時刻
    pub last_tick_time: Instant,
    /// ティック間隔（秒）
    pub tick_interval: f64,
    /// ポーズ状態
    pub paused: bool,
}

impl Default for TimeResource {
    fn default() -> Self {
        let now = Instant::now();
        Self {
            start_time: None,
            last_frame_time: now,
            delta_time: 0.0,
            total_time: 0.0,
            last_tick_time: now,
            tick_interval: 0.016, // 約60FPS
            paused: false,
        }
    }
}

impl TimeResource {
    /// 新しい時間リソースを作成
    pub fn new() -> Self {
        Self::default()
    }
    
    /// ゲームを開始
    pub fn start_game(&mut self) {
        self.start_time = Some(Instant::now());
        self.total_time = 0.0;
        self.paused = false;
    }
    
    /// ゲームをリセット
    pub fn reset(&mut self) {
        self.start_time = Some(Instant::now());
        self.total_time = 0.0;
        self.delta_time = 0.0;
        self.paused = false;
    }
    
    /// ポーズ状態をトグル
    pub fn toggle_pause(&mut self) {
        self.paused = !self.paused;
    }
    
    /// 時間を更新
    pub fn update(&mut self) {
        let now = Instant::now();
        
        // ポーズ中は時間を進めない
        if !self.paused {
            self.delta_time = (now - self.last_frame_time).as_secs_f64();
            self.total_time += self.delta_time;
        } else {
            self.delta_time = 0.0;
        }
        
        self.last_frame_time = now;
    }
    
    /// ティックが必要かどうか判定
    pub fn should_tick(&mut self) -> bool {
        if self.paused {
            return false;
        }
        
        let now = Instant::now();
        let elapsed = (now - self.last_tick_time).as_secs_f64();
        
        if elapsed >= self.tick_interval {
            self.last_tick_time = now;
            return true;
        }
        
        false
    }
    
    /// ゲーム開始からの経過時間を取得
    pub fn elapsed(&self) -> Duration {
        if let Some(start) = self.start_time {
            Instant::now().duration_since(start)
        } else {
            Duration::from_secs(0)
        }
    }
    
    /// 経過時間を文字列で取得（MM:SS形式）
    pub fn elapsed_str(&self) -> String {
        let secs = self.elapsed().as_secs();
        let minutes = secs / 60;
        let seconds = secs % 60;
        
        format!("{:02}:{:02}", minutes, seconds)
    }
    
    /// ティック間隔を設定（秒）
    pub fn set_tick_interval(&mut self, interval: f64) {
        self.tick_interval = interval;
    }
    
    /// デルタタイムを取得
    pub fn get_delta_time(&self) -> f64 {
        self.delta_time
    }
    
    /// 現在のFPSを計算して取得
    pub fn get_fps(&self) -> f64 {
        if self.delta_time > 0.0 {
            1.0 / self.delta_time
        } else {
            0.0 // ゼロ除算を避ける
        }
    }
} 