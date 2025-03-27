/**
 * 時間管理リソース
 * 
 * ゲームのフレームタイミング、FPS、ゲーム内時間を管理するリソース
 */
use wasm_bindgen::prelude::*;
use js_sys::Date;
use std::collections::VecDeque;

/// ゲーム内の時間管理を行うリソース
#[derive(Debug)]
pub struct TimeResource {
    /// 前回のフレームからの経過時間（秒）
    pub delta_time: f64,
    /// ゲーム開始からの合計時間（秒）
    pub total_time: f64,
    /// 描画フレーム数
    pub frame_count: u64,
    /// 前回のフレーム時刻（ミリ秒）
    pub last_frame_time: f64,
    /// FPS計算用のフレーム時間サンプル
    frame_times: VecDeque<f64>,
    /// FPS計算のサンプル数
    max_samples: usize,
    /// 現在のFPS（フレーム/秒）
    pub fps: f64,
    /// 現在時刻（ミリ秒）
    pub current_time: f64,
    /// 一時停止中かどうか
    pub is_paused: bool,
    /// 時間スケール（1.0が通常速度）
    pub time_scale: f64,
}

impl TimeResource {
    /// 新しいTimeResourceインスタンスを作成
    pub fn new() -> Self {
        Self {
            delta_time: 0.0,
            total_time: 0.0,
            frame_count: 0,
            last_frame_time: 0.0,
            frame_times: VecDeque::with_capacity(60),
            max_samples: 60,
            fps: 0.0,
            current_time: Date::now(),
            is_paused: false,
            time_scale: 1.0,
        }
    }

    /// 新しいフレームの開始処理
    pub fn begin_frame(&mut self) -> f64 {
        let now = Date::now();
        
        // 初回フレームの場合
        if self.last_frame_time == 0.0 {
            self.last_frame_time = now;
            self.delta_time = 0.0;
            self.current_time = now;
            return self.delta_time;
        }
        
        // 経過時間の計算（秒単位）
        self.delta_time = (now - self.last_frame_time) / 1000.0;
        
        // 一時停止中は時間を進めない
        if self.is_paused {
            self.delta_time = 0.0;
        } else {
            // 時間スケールの適用
            self.delta_time *= self.time_scale;
            self.total_time += self.delta_time;
        }
        
        // フレームカウントの更新
        self.frame_count += 1;
        
        // FPSの更新
        self.update_fps(now);
        
        // 時間情報の更新
        self.last_frame_time = now;
        self.current_time = now;
        
        self.delta_time
    }

    /// FPS（フレームレート）の更新
    fn update_fps(&mut self, now: f64) {
        // 現在のフレーム時間をサンプルに追加
        self.frame_times.push_back(now);
        
        // サンプル数が上限を超えたら古いものを削除
        if self.frame_times.len() > self.max_samples {
            self.frame_times.pop_front();
        }
        
        // 少なくとも2つのサンプルがあればFPSを計算
        if self.frame_times.len() >= 2 {
            let oldest = self.frame_times.front().unwrap();
            let newest = self.frame_times.back().unwrap();
            let time_span = newest - oldest;
            let frame_count = self.frame_times.len() as f64 - 1.0;
            
            // 時間差があればFPSを計算
            if time_span > 0.0 {
                self.fps = (frame_count / time_span) * 1000.0;
            }
        }
    }

    /// 一時停止状態を設定
    pub fn set_paused(&mut self, paused: bool) {
        self.is_paused = paused;
    }

    /// 時間スケールを設定
    pub fn set_time_scale(&mut self, scale: f64) {
        self.time_scale = scale.max(0.0);
    }

    /// 指定した間隔（秒）ごとにtrueを返す
    pub fn every_seconds(&self, interval: f64) -> bool {
        // 安全のために最小間隔を設定
        let interval = interval.max(0.001);
        let elapsed = (self.total_time / interval).round();
        (elapsed * interval).round() == self.total_time.round()
    }

    /// ミリ秒を読みやすい形式に変換（MM:SS.MS）
    pub fn format_ms(&self, ms: f64) -> String {
        let seconds = (ms / 1000.0).floor();
        let minutes = (seconds / 60.0).floor();
        let seconds = seconds % 60.0;
        let ms_part = ms % 1000.0;
        
        format!("{:02}:{:02}.{:03}", minutes as u32, seconds as u32, ms_part as u32)
    }

    /// 現在のFPSを文字列で取得
    pub fn format_fps(&self) -> String {
        format!("{:.1} FPS", self.fps)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread::sleep;
    use std::time::Duration;

    #[test]
    fn test_new_time_resource() {
        let resource = TimeResource::new();
        assert_eq!(resource.delta_time, 0.0);
        assert_eq!(resource.total_time, 0.0);
        assert_eq!(resource.frame_count, 0);
        assert!(!resource.is_paused);
        assert_eq!(resource.time_scale, 1.0);
    }

    #[test]
    fn test_format_ms() {
        let resource = TimeResource::new();
        assert_eq!(resource.format_ms(1000.0), "00:01.000");
        assert_eq!(resource.format_ms(61000.0), "01:01.000");
        assert_eq!(resource.format_ms(3661500.0), "61:01.500");
    }

    #[test]
    fn test_paused_state() {
        let mut resource = TimeResource::new();
        assert!(!resource.is_paused);
        
        resource.set_paused(true);
        assert!(resource.is_paused);
        
        // 一時停止中はdelta_timeが0になることを確認
        resource.last_frame_time = Date::now() - 100.0; // 100ms前
        let dt = resource.begin_frame();
        assert_eq!(dt, 0.0);
        
        // 一時停止解除後はdelta_timeが更新されることを確認
        resource.set_paused(false);
        sleep(Duration::from_millis(10)); // 少し待つ
        let dt = resource.begin_frame();
        assert!(dt > 0.0);
    }
} 