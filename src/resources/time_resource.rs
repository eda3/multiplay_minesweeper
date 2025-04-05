/**
 * 時間リソース
 * 
 * ゲーム内の時間管理を担当する
 */
use super::resource_trait::Resource;
use wasm_bindgen::prelude::*;
use js_sys::Date;
use std::any::Any;
use std::time::{Duration, Instant};

/// ゲーム内時間管理リソース
#[derive(Debug, Clone)]
pub struct TimeResource {
    /// 前回のフレーム時間
    pub previous_time: Instant,
    /// 現在の時間
    pub current_time: Instant,
    /// デルタタイム（前回のフレームからの経過時間、秒単位）
    pub delta_time: f64,
    /// フレーム数
    pub frame_count: u64,
    /// フレームレート（FPS）
    pub fps: f64,
    /// FPS計算用の時間累積
    pub fps_time_accumulator: f64,
    /// FPS計算用のフレーム数
    pub fps_frame_accumulator: u32,
    /// 固定デルタタイム（オプション）
    pub fixed_delta: Option<f64>,
}

impl Default for TimeResource {
    fn default() -> Self {
        let now = Instant::now();
        Self {
            previous_time: now,
            current_time: now,
            delta_time: 0.0,
            frame_count: 0,
            fps: 0.0,
            fps_time_accumulator: 0.0,
            fps_frame_accumulator: 0,
            fixed_delta: None,
        }
    }
}

impl TimeResource {
    /// 新しい時間リソースを作成
    pub fn new() -> Self {
        Self::default()
    }
    
    /// 時間を更新
    pub fn update(&mut self) {
        self.previous_time = self.current_time;
        self.current_time = Instant::now();
        
        // デルタタイムを計算（秒単位）
        let delta = self.current_time.duration_since(self.previous_time);
        self.delta_time = delta.as_secs_f64();
        
        // 固定デルタタイムがある場合はそれを使用
        if let Some(fixed) = self.fixed_delta {
            self.delta_time = fixed;
        }
        
        // フレームカウントを更新
        self.frame_count += 1;
        
        // FPS計算（1秒ごとに更新）
        self.fps_time_accumulator += self.delta_time;
        self.fps_frame_accumulator += 1;
        
        if self.fps_time_accumulator >= 1.0 {
            self.fps = self.fps_frame_accumulator as f64 / self.fps_time_accumulator;
            self.fps_time_accumulator = 0.0;
            self.fps_frame_accumulator = 0;
        }
    }
    
    /// ゲームが実行時間を取得（秒）
    pub fn total_time(&self) -> f64 {
        self.current_time.duration_since(self.previous_time).as_secs_f64()
    }
    
    /// 固定されたデルタタイムを取得（フレームレート平滑化用）
    pub fn fixed_delta_time(&self, max_delta: f64) -> f64 {
        self.delta_time.min(max_delta)
    }
    
    /// 合計の経過時間をDurationとして取得
    pub fn duration(&self) -> Duration {
        self.current_time.duration_since(self.previous_time)
    }
    
    /// 固定デルタタイムを設定
    pub fn set_fixed_delta(&mut self, delta: Option<f64>) {
        self.fixed_delta = delta;
    }
    
    /// フレームの開始処理（互換性のため）
    pub fn begin_frame(&mut self) {
        self.update();
    }
} 