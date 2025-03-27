/**
 * パラレルシステムエグゼキューター
 * 
 * 依存関係を考慮した並列システム実行機能
 */
use std::collections::{HashMap, HashSet};
use wasm_bindgen::prelude::*;
use web_sys::console;
use js_sys::Promise;

use crate::entities::EntityManager;
use super::system_trait::System;

/// 実行レベル（依存関係に基づくグループ）
#[derive(Debug)]
struct ExecutionLevel {
    /// 実行レベルに含まれるシステムのインデックス
    system_indices: Vec<usize>,
}

/// パラレルシステムエグゼキューター
#[derive(Debug)]
pub struct ParallelExecutor {
    /// 実行レベル（依存関係に基づいて分離された層）
    levels: Vec<ExecutionLevel>,
    /// システム間の依存関係マップ
    dependencies: HashMap<usize, HashSet<usize>>,
}

impl ParallelExecutor {
    /// 新しいパラレルエグゼキューターを作成
    pub fn new() -> Self {
        Self {
            levels: Vec::new(),
            dependencies: HashMap::new(),
        }
    }
    
    /// 依存関係グラフからレベルを構築
    pub fn build_from_systems(
        &mut self,
        systems: &[Box<dyn System>],
        name_to_index: &HashMap<String, usize>,
    ) -> Result<(), String> {
        // 依存関係マップをクリア
        self.dependencies.clear();
        self.levels.clear();
        
        // システム間の依存関係を構築
        for (index, system) in systems.iter().enumerate() {
            let deps: HashSet<usize> = system.dependencies()
                .into_iter()
                .filter_map(|name| name_to_index.get(name).copied())
                .collect();
                
            self.dependencies.insert(index, deps);
        }
        
        // レベルごとにシステムをグループ化
        let mut remaining: HashSet<usize> = (0..systems.len()).collect();
        
        while !remaining.is_empty() {
            let mut current_level = Vec::new();
            
            // 依存関係を持たないシステムを見つける
            for &index in &remaining.clone() {
                let has_remaining_deps = self.dependencies.get(&index)
                    .map(|deps| deps.iter().any(|dep| remaining.contains(dep)))
                    .unwrap_or(false);
                    
                if !has_remaining_deps {
                    current_level.push(index);
                    remaining.remove(&index);
                }
            }
            
            // 循環依存のチェック
            if current_level.is_empty() && !remaining.is_empty() {
                return Err("循環依存が検出されました".to_string());
            }
            
            // このレベルを追加
            self.levels.push(ExecutionLevel {
                system_indices: current_level,
            });
        }
        
        Ok(())
    }
    
    /// 各レベルを順次実行し、各レベル内のシステムは並列で実行
    pub fn execute(
        &self,
        systems: &mut [Box<dyn System>],
        entity_manager: &mut EntityManager,
        delta_time: f32,
    ) {
        // WebWorkerが利用可能な環境では並列実行
        // 注：Rustの標準的な並列ライブラリ（std::thread など）は
        // WebAssemblyでは使用できないため、ここでは
        // 将来的なWeb Worker実装のためのプレースホルダーとしています
        
        // 現状では、レベルごとに逐次処理
        for level in &self.levels {
            // このレベル内のシステムを実行
            for &index in &level.system_indices {
                let system = &mut systems[index];
                if system.is_active() && system.is_runnable(entity_manager) {
                    system.update(entity_manager, delta_time);
                }
            }
        }
    }
    
    /// デバッグ情報を出力
    pub fn print_levels(&self) {
        let mut debug_info = String::from("=== パラレル実行レベル ===\n");
        
        for (i, level) in self.levels.iter().enumerate() {
            debug_info.push_str(&format!("レベル {}: ", i));
            for (j, &index) in level.system_indices.iter().enumerate() {
                if j > 0 {
                    debug_info.push_str(", ");
                }
                debug_info.push_str(&format!("{}", index));
            }
            debug_info.push('\n');
        }
        
        console::log_1(&JsValue::from_str(&debug_info));
    }
    
    /// レベル数を取得
    pub fn level_count(&self) -> usize {
        self.levels.len()
    }
}

/// 将来的なWeb Worker対応を視野に入れた並列実行関数
#[wasm_bindgen]
pub fn execute_system_in_worker(_system_index: usize, _entity_manager_ptr: usize, _delta_time: f32) -> Promise {
    // 将来的な実装のためのプレースホルダー
    // Web Workerベースの並列処理をサポートする場合に実装する
    
    // 現状では単にPromiseを返す
    Promise::resolve(&JsValue::NULL)
} 