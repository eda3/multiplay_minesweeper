/**
 * システムスケジューラ
 * 
 * システムの実行スケジュールを管理する
 */
use wasm_bindgen::JsValue;
use js_sys::Date;
use web_sys::console;
use std::any::TypeId;
use wasm_bindgen::prelude::*;
use std::fmt::{self, Debug};
use super::system_registry::SystemRegistry;
use crate::entities::EntityManager;
use crate::entities::entity::EntityId;
use super::resource_dependency::ResourceDependency;
use crate::resources::resource_manager::ResourceManager;

/// デルタ時間の型エイリアス（秒単位）
pub type DeltaTime = f32;

#[macro_export]
macro_rules! console_error {
    ($($arg:tt)*) => {
        web_sys::console::error_1(&format!($($arg)*).into());
    }
}

/// クロージャをSystemトレイトとして実装するためのラッパー
pub struct SystemFn<F> 
where 
    F: FnMut(&mut ResourceManager, DeltaTime) -> Result<(), JsValue> + Send + Sync + 'static
{
    /// 実行する関数
    func: F,
    /// システム名
    name: String,
    /// アクティブ状態
    active: bool,
}

impl<F> SystemFn<F> 
where 
    F: FnMut(&mut ResourceManager, DeltaTime) -> Result<(), JsValue> + Send + Sync + 'static
{
    /// 新しい関数システムを作成
    pub fn new(name: impl Into<String>, func: F) -> Self {
        Self {
            func,
            name: name.into(),
            active: true,
        }
    }
}

impl<F> Debug for SystemFn<F> 
where 
    F: FnMut(&mut ResourceManager, DeltaTime) -> Result<(), JsValue> + Send + Sync + 'static
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SystemFn")
            .field("name", &self.name)
            .field("active", &self.active)
            .finish()
    }
}

impl<F> super::system_trait::System for SystemFn<F> 
where 
    F: FnMut(&mut ResourceManager, DeltaTime) -> Result<(), JsValue> + Send + Sync + 'static
{
    fn name(&self) -> &str {
        &self.name
    }
    
    fn update(&mut self, _entity_manager: &mut EntityManager, delta_time: f32) {
        // 別のアプローチ: ここでは一時的にダミーリソースマネージャーを作成
        let mut dummy_resource_manager = ResourceManager::new();
        
        // ダミーリソースマネージャーを使ってシステム関数を呼び出す
        if let Err(e) = (self.func)(&mut dummy_resource_manager, delta_time) {
            console_error!("System error in {}: {:?}", self.name, e);
        }
    }
    
    fn is_active(&self) -> bool {
        self.active
    }
    
    fn set_active(&mut self, active: bool) {
        self.active = active;
    }
}

/// システムスケジューラ
/// 固定レート更新と可変レート更新のサポート
#[derive(Debug)]
pub struct SystemScheduler {
    /// 固定更新レート（秒あたりの更新回数）
    fixed_update_rate: f32,
    /// 固定更新のための累積時間
    accumulated_time: f32,
    /// 最後のフレーム時間
    last_frame_time: f64,
    /// 最大Delta時間（異常値を防ぐ）
    max_delta_time: f32,
    /// フレームカウント
    frame_count: u64,
    /// 最後の秒数
    last_second: f64,
    /// 現在のFPS
    current_fps: f32,
    /// デバッグ表示を有効にするか
    debug_mode: bool,
}

impl SystemScheduler {
    /// 新しいシステムスケジューラを作成
    pub fn new(fixed_update_rate: f32) -> Self {
        Self {
            fixed_update_rate,
            accumulated_time: 0.0,
            last_frame_time: Date::now(),
            max_delta_time: 0.1, // 100ミリ秒を超えるフレーム時間はキャップ
            frame_count: 0,
            last_second: Date::now() / 1000.0,
            current_fps: 0.0,
            debug_mode: false,
        }
    }
    
    /// システムの更新を実行
    pub fn update(
        &mut self,
        registry: &mut SystemRegistry,
        entity_manager: &mut EntityManager,
    ) -> Result<(), JsValue> {
        // 現在の時間を取得
        let now = Date::now();
        
        // デルタ時間を計算（秒単位）
        let mut delta_time = (now - self.last_frame_time) / 1000.0;
        
        // 異常に大きなdelta_timeをキャップ
        if delta_time > self.max_delta_time as f64 {
            delta_time = self.max_delta_time as f64;
        }
        
        // 最終フレーム時間を更新
        self.last_frame_time = now;
        
        // フレームカウンタを更新
        self.frame_count += 1;
        let current_second = now / 1000.0;
        if current_second - self.last_second >= 1.0 {
            self.current_fps = self.frame_count as f32;
            self.frame_count = 0;
            self.last_second = current_second;
            
            if self.debug_mode {
                console::log_1(&JsValue::from_str(&format!("FPS: {:.1}", self.current_fps)));
            }
        }
        
        // 可変レート更新を実行（毎フレーム）
        registry.update_group("VariableUpdate", entity_manager, delta_time as f32)
            .map_err(|e| JsValue::from_str(&e))?;
        
        // 固定レート更新
        let fixed_dt = 1.0 / self.fixed_update_rate;
        self.accumulated_time += delta_time as f32;
        
        while self.accumulated_time >= fixed_dt {
            registry.update_group("FixedUpdate", entity_manager, fixed_dt)
                .map_err(|e| JsValue::from_str(&e))?;
                
            self.accumulated_time -= fixed_dt;
        }
        
        // レンダリング関連システムを実行
        registry.update_group("PreRender", entity_manager, delta_time as f32)
            .map_err(|e| JsValue::from_str(&e))?;
            
        registry.update_group("Render", entity_manager, delta_time as f32)
            .map_err(|e| JsValue::from_str(&e))?;
            
        registry.update_group("PostRender", entity_manager, delta_time as f32)
            .map_err(|e| JsValue::from_str(&e))?;
        
        Ok(())
    }
    
    /// 現在のFPSを取得
    pub fn get_fps(&self) -> f32 {
        self.current_fps
    }
    
    /// 固定更新レートを設定
    pub fn set_fixed_update_rate(&mut self, rate: f32) {
        self.fixed_update_rate = rate;
    }
    
    /// 最大デルタ時間を設定
    pub fn set_max_delta_time(&mut self, max_delta: f32) {
        self.max_delta_time = max_delta;
    }
    
    /// デバッグモードを設定
    pub fn set_debug_mode(&mut self, debug: bool) {
        self.debug_mode = debug;
    }
}

/// 更新頻度を制御するシステムラッパー
/// 実行頻度を制限したいシステムを装飾するためのデコレーター
#[derive(Debug)]
pub struct RateControlledSystem<S> {
    /// 内部システム
    system: S,
    /// 更新間隔（秒）
    update_interval: f32,
    /// 最後の更新からの経過時間
    time_since_last_update: f32,
    /// アクティブかどうか
    is_active: bool,
}

impl<S: super::system_trait::System> RateControlledSystem<S> {
    /// 新しいレート制御システムを作成
    pub fn new(system: S, updates_per_second: f32) -> Self {
        Self {
            system,
            update_interval: 1.0 / updates_per_second,
            time_since_last_update: 0.0,
            is_active: true,
        }
    }
    
    /// 更新頻度を設定
    pub fn set_update_rate(&mut self, updates_per_second: f32) {
        self.update_interval = 1.0 / updates_per_second;
    }
}

impl<S: super::system_trait::System> super::system_trait::System for RateControlledSystem<S> {
    fn name(&self) -> &str {
        self.system.name()
    }
    
    fn init(&mut self, entity_manager: &mut EntityManager) {
        self.system.init(entity_manager);
    }
    
    fn update(&mut self, entity_manager: &mut EntityManager, delta_time: f32) {
        self.time_since_last_update += delta_time;
        
        if self.time_since_last_update >= self.update_interval {
            self.system.update(entity_manager, self.time_since_last_update);
            self.time_since_last_update = 0.0;
        }
    }
    
    fn shutdown(&mut self, entity_manager: &mut EntityManager) {
        self.system.shutdown(entity_manager);
    }
    
    fn dependencies(&self) -> Vec<&str> {
        self.system.dependencies()
    }
    
    fn is_runnable(&self, entity_manager: &EntityManager) -> bool {
        self.system.is_runnable(entity_manager)
    }
    
    fn priority(&self) -> i32 {
        self.system.priority()
    }
    
    fn is_active(&self) -> bool {
        self.is_active && self.system.is_active()
    }
    
    fn set_active(&mut self, active: bool) {
        self.is_active = active;
    }
} 