/**
 * 最適化されたシステムレジストリ
 * 
 * システムの登録、依存関係解決、実行を管理する拡張レジストリ
 */
use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use std::any::{Any, TypeId};
use std::rc::Rc;
use std::cell::RefCell;

use wasm_bindgen::JsValue;
use web_sys::console;

use crate::entities::EntityManager;
use crate::resources::{
    TimeResource, CoreGameResource, PlayerStateResource, 
    GameConfigResource, ResourceManager, ResourceBatch, ResourceBatchMut
};
use super::system_trait::System;
use super::system_group::SystemGroup;
use super::resource_dependency::ResourceDependency;

/// システムレジストリ
/// 全システムとグループの管理を行う
#[derive(Debug)]
pub struct SystemRegistry {
    /// 登録されたシステム
    systems: Vec<Box<dyn System>>,
    /// 名前でシステムを検索するためのマップ
    system_name_map: HashMap<String, usize>,
    /// システムグループ
    groups: HashMap<String, SystemGroup>,
    /// リソースマネージャー
    resource_manager: ResourceManager,
    /// システムごとのリソース依存関係（システムインデックス -> リソース依存情報）
    system_resource_dependencies: HashMap<usize, (Vec<TypeId>, Vec<TypeId>)>,
    /// 依存関係キャッシュ（システム名 -> 依存するシステム名のセット）
    dependency_cache: HashMap<String, HashSet<String>>,
    /// 実行順序キャッシュ
    execution_order: Vec<usize>,
    /// 前回のフレーム時間
    last_frame_time: f64,
    /// キャッシュが有効かどうか
    cache_valid: bool,
}

impl Default for SystemRegistry {
    fn default() -> Self {
        Self {
            systems: Vec::new(),
            system_name_map: HashMap::new(),
            groups: HashMap::new(),
            resource_manager: ResourceManager::new(),
            system_resource_dependencies: HashMap::new(),
            dependency_cache: HashMap::new(),
            execution_order: Vec::new(),
            last_frame_time: 0.0,
            cache_valid: false,
        }
    }
}

impl SystemRegistry {
    /// 新しいシステムレジストリを作成
    pub fn new() -> Self {
        Self::default()
    }
    
    /// システムを登録して、リソース依存関係も登録
    pub fn register<S: System + 'static>(&mut self, system: S) {
        let name = system.name().to_string();
        let index = self.systems.len();
        
        // リソース依存関係を記録
        let read_resources = system.read_resources();
        let write_resources = system.write_resources();
        self.system_resource_dependencies.insert(index, (read_resources, write_resources));
        
        // 名前をマップに登録
        self.system_name_map.insert(name, index);
        
        // システムを追加
        self.systems.push(Box::new(system));
        
        // キャッシュを無効化
        self.cache_valid = false;
    }
    
    /// リソース依存関係を持つシステムを登録
    pub fn register_with_deps<S, D>(&mut self, mut system: S, dependencies: D) 
    where 
        S: System + 'static,
        D: ResourceDependency,
    {
        let name = system.name().to_string();
        let index = self.systems.len();
        
        // 名前をマップに登録
        self.system_name_map.insert(name, index);
        
        // リソース依存関係を解析
        let type_ids = dependencies.resource_type_ids();
        
        // 現在の実装ではシンプルに - より高度な読み書き分離は今後の課題
        let read_res: Vec<TypeId> = type_ids.clone();
        let write_res: Vec<TypeId> = vec![];
        
        // 依存関係を記録
        self.system_resource_dependencies.insert(index, (read_res, write_res));
        
        // システムを追加
        self.systems.push(Box::new(system));
        
        // キャッシュを無効化
        self.cache_valid = false;
    }
    
    /// システムグループを登録
    pub fn register_group(&mut self, group: SystemGroup) {
        let name = group.name().to_string();
        self.groups.insert(name, group);
        
        // キャッシュを無効化
        self.cache_valid = false;
    }
    
    /// 名前でシステムグループを取得
    pub fn get_group(&self, name: &str) -> Option<&SystemGroup> {
        self.groups.get(name)
    }
    
    /// 名前でシステムグループを取得（可変）
    pub fn get_group_mut(&mut self, name: &str) -> Option<&mut SystemGroup> {
        self.groups.get_mut(name)
    }
    
    /// 特定グループの全システムを更新
    pub fn update_group(&mut self, group_name: &str, entity_manager: &mut EntityManager, delta_time: f32) -> Result<(), String> {
        if let Some(group) = self.groups.get_mut(group_name) {
            group.update_all(entity_manager, delta_time);
            Ok(())
        } else {
            Err(format!("グループが見つかりません: {}", group_name))
        }
    }
    
    /// 全システムを更新（依存関係を考慮）
    pub fn update_all(&mut self, entity_manager: &mut EntityManager, delta_time: f32) -> Result<(), String> {
        // 最初の実行または無効化された場合は依存関係を再解決
        if !self.cache_valid {
            self.resolve_dependencies()?;
            self.cache_valid = true;
        }
        
        // システムを実行順序に従って実行
        for &index in &self.execution_order {
            let system = &mut self.systems[index];
            if system.is_active() && system.is_runnable(entity_manager) {
                system.update(entity_manager, delta_time);
            }
        }
        
        Ok(())
    }
    
    /// リソースを追加
    pub fn add_resource<T: 'static>(&mut self, resource: T) {
        self.resource_manager.insert(resource);
    }
    
    /// リソースを取得
    pub fn get_resource<T: 'static>(&self) -> Option<&T> {
        self.resource_manager.get::<T>()
    }
    
    /// リソースを取得（可変）
    pub fn get_resource_mut<T: 'static>(&mut self) -> Option<&mut T> {
        self.resource_manager.get_mut::<T>()
    }
    
    /// 安全な読み取り専用バッチアクセス
    pub fn with_resources<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&ResourceBatch) -> R,
    {
        self.resource_manager.batch(f)
    }
    
    /// バッチ処理（読み書き）のための安全なアクセスを提供
    pub fn with_resources_mut<F, R>(&mut self, f: F) -> R
    where
        F: FnOnce(&mut ResourceBatchMut) -> R,
    {
        self.resource_manager.batch_mut(f)
    }
    
    /// リソースが存在するかチェック
    pub fn has_resource<T: 'static>(&self) -> bool {
        self.resource_manager.contains::<T>()
    }
    
    /// リソースを削除
    pub fn remove_resource<T: 'static>(&mut self) -> Option<T> {
        self.resource_manager.remove::<T>()
    }
    
    /// システムの依存関係に基づいて実行順序を決定
    fn resolve_dependencies(&mut self) -> Result<(), String> {
        // 名前から依存関係へのマッピングを構築
        self.dependency_cache.clear();
        for (i, system) in self.systems.iter().enumerate() {
            let name = system.name().to_string();
            let deps: HashSet<String> = system.dependencies()
                .into_iter()
                .map(|s| s.to_string())
                .collect();
                
            self.dependency_cache.insert(name, deps);
        }
        
        // リソース依存関係からシステム依存関係を推論
        self.infer_dependencies_from_resources();
        
        // トポロジカルソートのための状態
        let mut result = Vec::new();
        let mut visited = HashSet::new();
        let mut temp_mark = HashSet::new();
        
        // 全てのシステムに対してトポロジカルソートを実行
        for i in 0..self.systems.len() {
            if !visited.contains(&i) {
                self.visit(i, &mut visited, &mut temp_mark, &mut result)?;
            }
        }
        
        // 実行順序を設定（トポロジカルソートの結果は逆順）
        self.execution_order = result;
        
        Ok(())
    }
    
    /// リソース依存関係からシステム依存関係を推論
    fn infer_dependencies_from_resources(&mut self) {
        // 書き込みリソースを持つシステムのマップを作成
        let mut write_resource_systems: HashMap<TypeId, Vec<usize>> = HashMap::new();
        
        // 各システムの書き込みリソースを登録
        for (index, (_, write_resources)) in &self.system_resource_dependencies {
            for &type_id in write_resources {
                write_resource_systems
                    .entry(type_id)
                    .or_insert_with(Vec::new)
                    .push(*index);
            }
        }
        
        // 各システムについて、依存関係を推論
        for (index, (read_resources, write_resources)) in &self.system_resource_dependencies {
            let system_name = self.systems[*index].name().to_string();
            let mut deps = self.dependency_cache
                .entry(system_name.clone())
                .or_insert_with(HashSet::new);
            
            // 読み取りリソースの依存関係
            for &read_type_id in read_resources {
                // このリソースを書き込むシステムに依存する
                if let Some(writers) = write_resource_systems.get(&read_type_id) {
                    for &writer_index in writers {
                        // 自分自身には依存しない
                        if writer_index != *index {
                            let writer_name = self.systems[writer_index].name().to_string();
                            deps.insert(writer_name);
                        }
                    }
                }
            }
            
            // 書き込みリソースの依存関係
            for &write_type_id in write_resources {
                // 同じリソースを書き込む他のシステムとの依存関係
                if let Some(writers) = write_resource_systems.get(&write_type_id) {
                    for &writer_index in writers {
                        // 自分自身には依存しない
                        if writer_index != *index && writer_index < *index {
                            let writer_name = self.systems[writer_index].name().to_string();
                            deps.insert(writer_name);
                        }
                    }
                }
                
                // このリソースを読み取るシステムが、このシステムに依存するようにする
                for (reader_index, (reader_resources, _)) in &self.system_resource_dependencies {
                    if reader_index != index && reader_resources.contains(&write_type_id) {
                        let reader_name = self.systems[*reader_index].name().to_string();
                        let reader_deps = self.dependency_cache
                            .entry(reader_name)
                            .or_insert_with(HashSet::new);
                        reader_deps.insert(system_name.clone());
                    }
                }
            }
        }
    }
    
    /// トポロジカルソートのための再帰的訪問関数
    fn visit(
        &self,
        index: usize,
        visited: &mut HashSet<usize>,
        temp_mark: &mut HashSet<usize>,
        result: &mut Vec<usize>,
    ) -> Result<(), String> {
        // 循環依存のチェック
        if temp_mark.contains(&index) {
            return Err(format!("システム依存関係の循環が検出されました: {}", self.systems[index].name()));
        }
        
        // 既に訪問済みならスキップ
        if visited.contains(&index) {
            return Ok(());
        }
        
        // 一時マークを設定
        temp_mark.insert(index);
        
        // このシステムの依存関係を取得
        let system_name = self.systems[index].name().to_string();
        if let Some(deps) = self.dependency_cache.get(&system_name) {
            // 各依存システムを処理
            for dep_name in deps {
                if let Some(&dep_index) = self.system_name_map.get(dep_name) {
                    // 依存先を再帰的に訪問
                    self.visit(dep_index, visited, temp_mark, result)?;
                } else {
                    return Err(format!("未登録の依存システム: {} -> {}", system_name, dep_name));
                }
            }
        }
        
        // 一時マークを解除
        temp_mark.remove(&index);
        
        // 訪問済みとしてマーク
        visited.insert(index);
        
        // 結果に追加
        result.push(index);
        
        Ok(())
    }
    
    /// 全システムを初期化
    pub fn init_all(&mut self, entity_manager: &mut EntityManager) {
        for system in &mut self.systems {
            system.init(entity_manager);
        }
    }
    
    /// 全システムをシャットダウン
    pub fn shutdown_all(&mut self, entity_manager: &mut EntityManager) {
        for system in &mut self.systems {
            system.shutdown(entity_manager);
        }
    }
    
    /// 名前でシステムを有効/無効化
    pub fn set_system_active(&mut self, name: &str, active: bool) -> bool {
        if let Some(&index) = self.system_name_map.get(name) {
            self.systems[index].set_active(active);
            true
        } else {
            false
        }
    }
    
    /// 名前でグループを有効/無効化
    pub fn set_group_active(&mut self, name: &str, active: bool) -> bool {
        if let Some(group) = self.groups.get_mut(name) {
            group.set_active(active);
            true
        } else {
            false
        }
    }
    
    /// 基本リソースの初期化
    pub fn init_core_resources(&mut self) {
        self.add_resource(TimeResource::new());
        self.add_resource(CoreGameResource::new());
        self.add_resource(PlayerStateResource::new());
        self.add_resource(GameConfigResource::new());
    }
    
    /// デバッグ情報を出力
    pub fn debug_info(&self) -> String {
        let mut info = String::new();
        
        info.push_str("=== システムレジストリ情報 ===\n");
        info.push_str(&format!("登録システム数: {}\n", self.systems.len()));
        info.push_str(&format!("グループ数: {}\n", self.groups.len()));
        info.push_str(&format!("リソース数: {}\n", self.resource_manager.len()));
        
        info.push_str("\n--- システム一覧 ---\n");
        for (i, system) in self.systems.iter().enumerate() {
            info.push_str(&format!("{}. {} (優先度: {}, アクティブ: {})\n", 
                i, system.name(), system.priority(), system.is_active()));
            
            // 依存関係
            let deps = system.dependencies();
            if !deps.is_empty() {
                info.push_str("   依存: ");
                for (j, dep) in deps.iter().enumerate() {
                    if j > 0 {
                        info.push_str(", ");
                    }
                    info.push_str(dep);
                }
                info.push_str("\n");
            }
            
            // リソース依存関係
            if let Some((read, write)) = self.system_resource_dependencies.get(&i) {
                if !read.is_empty() {
                    info.push_str("   読み取りリソース: ");
                    for (j, _) in read.iter().enumerate() {
                        if j > 0 {
                            info.push_str(", ");
                        }
                        if j < system.resource_dependency_names().len() {
                            info.push_str(system.resource_dependency_names()[j]);
                        } else {
                            info.push_str("不明");
                        }
                    }
                    info.push_str("\n");
                }
                
                if !write.is_empty() {
                    info.push_str("   書き込みリソース: ");
                    for (j, _) in write.iter().enumerate() {
                        if j > 0 {
                            info.push_str(", ");
                        }
                        if j + read.len() < system.resource_dependency_names().len() {
                            info.push_str(system.resource_dependency_names()[j + read.len()]);
                        } else {
                            info.push_str("不明");
                        }
                    }
                    info.push_str("\n");
                }
            }
        }
        
        info.push_str("\n--- グループ一覧 ---\n");
        for (name, group) in &self.groups {
            info.push_str(&format!("グループ: {} (優先度: {}, アクティブ: {}, システム数: {})\n",
                name, group.priority(), group.is_active(), group.system_count()));
        }
        
        info.push_str("\n--- 実行順序 ---\n");
        for (i, &index) in self.execution_order.iter().enumerate() {
            info.push_str(&format!("{}. {}\n", i, self.systems[index].name()));
        }
        
        info
    }
    
    /// デバッグ情報をコンソールに出力
    pub fn print_debug_info(&self) {
        let info = self.debug_info();
        console::log_1(&JsValue::from_str(&info));
    }
    
    /// ボード関連のシステムを登録
    pub fn register_board_systems(&mut self) {
        use crate::systems::board_systems::{
            board_init_system,
            cell_reveal_system,
            flag_toggle_system,
            win_condition_system
        };
        
        self.add_system("board_init", board_init_system, SystemPriority::PreUpdate);
        self.add_system("cell_reveal", cell_reveal_system, SystemPriority::Update);
        self.add_system("flag_toggle", flag_toggle_system, SystemPriority::Update);
        self.add_system("win_condition", win_condition_system, SystemPriority::PostUpdate);
    }
} 