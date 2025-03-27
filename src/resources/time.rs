/**
 * 時間管理リソース
 * 
 * ゲームのフレームタイミング、FPS、ゲーム内時間を管理するリソース
 */
use wasm_bindgen::JsValue;
use js_sys::Date;
use std::collections::VecDeque;

/// 時間管理リソース
#[derive(Debug, Clone)]
pub struct TimeResource {
    /// フレーム間の経過時間（秒）
    pub delta_time: f64,
    /// ゲーム開始からの総経過時間（秒）
    pub total_time: f64,
    /// 現在のフレーム数
    pub frame_count: u64,
    /// 前回のフレーム時間
    pub last_frame_time: f64,
    /// FPS計算用の過去フレーム時間履歴
    frame_times: VecDeque<f64>,
    /// 最大FPS計測サンプル数
    max_samples: usize,
    /// 現在のFPS
    pub fps: f64,
    /// 現在のフレームタイムスタンプ
    pub current_time: f64,
    /// ポーズ状態
    pub is_paused: bool,
    /// 時間スケール（1.0が標準速度）
    pub time_scale: f64,
}

impl Default for TimeResource {
    fn default() -> Self {
        Self {
            delta_time: 0.0,
            total_time: 0.0,
            frame_count: 0,
            last_frame_time: Date::now(),
            frame_times: VecDeque::with_capacity(60),
            max_samples: 60,
            fps: 0.0,
            current_time: Date::now(),
            is_paused: false,
            time_scale: 1.0,
        }
    }
}

impl TimeResource {
    /// 新しい時間管理リソースを作成
    pub fn new() -> Self {
        Self::default()
    }
    
    /// フレームの開始時に呼び出し、時間を更新
    pub fn begin_frame(&mut self) -> Result<f64, JsValue> {
        // 現在の時間を取得
        let now = Date::now();
        self.current_time = now;
        
        // デルタタイムを計算（秒単位）
        self.delta_time = (now - self.last_frame_time) / 1000.0;
        
        // 異常に大きいデルタタイム（例：タブが非アクティブだった場合）を制限
        if self.delta_time > 0.1 {
            self.delta_time = 0.1;
        }
        
        // ポーズ中ならデルタタイムを0に
        if self.is_paused {
            self.delta_time = 0.0;
        } else {
            // 時間スケールを適用
            self.delta_time *= self.time_scale;
            
            // 総時間を更新
            self.total_time += self.delta_time;
        }
        
        // フレームカウンター更新
        self.frame_count += 1;
        
        // FPS計算
        self.update_fps(now);
        
        // 現在の時間を記録
        self.last_frame_time = now;
        
        Ok(self.delta_time)
    }
    
    /// FPSを更新
    fn update_fps(&mut self, now: f64) {
        // 新しいフレーム時間を追加
        self.frame_times.push_back(now);
        
        // 古いサンプルを削除
        while self.frame_times.len() > self.max_samples {
            self.frame_times.pop_front();
        }
        
        // サンプルが十分あればFPSを計算
        if self.frame_times.len() >= 2 {
            let oldest = self.frame_times.front().unwrap();
            let newest = self.frame_times.back().unwrap();
            let elapsed = (newest - oldest) / 1000.0; // 秒単位
            
            if elapsed > 0.0 {
                self.fps = (self.frame_times.len() as f64 - 1.0) / elapsed;
            }
        }
    }
    
    /// ポーズ状態を設定
    pub fn set_paused(&mut self, paused: bool) {
        self.is_paused = paused;
    }
    
    /// 時間スケールを設定（スローモーション/早送り）
    pub fn set_time_scale(&mut self, scale: f64) {
        self.time_scale = scale.max(0.1).min(10.0); // 0.1〜10.0の範囲に制限
    }
    
    /// 特定の間隔ごとにtrueを返す（定期的なイベントに便利）
    pub fn every_seconds(&self, interval: f64) -> bool {
        let mod_time = self.total_time % interval;
        mod_time < self.delta_time
    }
    
    /// ミリ秒をフォーマットした文字列にする（デバッグ表示用）
    pub fn format_ms(&self, ms: f64) -> String {
        format!("{:.1}ms", ms)
    }
    
    /// 現在のFPSをフォーマットした文字列にする
    pub fn format_fps(&self) -> String {
        format!("{:.1} FPS", self.fps)
    }
} 