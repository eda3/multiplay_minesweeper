/**
 * システムグループ
 * 
 * 関連するシステムをグループ化し、一括管理するための機能
 */
use crate::entities::EntityManager;
use super::system_trait::System;
use std::fmt::Debug;

/// システムグループ
/// 複数のシステムをグループ化して管理する
#[derive(Debug)]
pub struct SystemGroup {
    /// グループ名
    name: String,
    /// システムのコレクション
    systems: Vec<Box<dyn System>>,
    /// グループの優先度
    priority: i32,
    /// グループが有効かどうか
    is_active: bool,
}

impl SystemGroup {
    /// 新しいシステムグループを作成
    pub fn new<S: Into<String>>(name: S, priority: i32) -> Self {
        Self {
            name: name.into(),
            systems: Vec::new(),
            priority,
            is_active: true,
        }
    }
    
    /// システムをグループに追加
    pub fn add<S: System + 'static>(&mut self, system: S) {
        self.systems.push(Box::new(system));
    }
    
    /// グループ内の全システムを更新
    pub fn update_all(&mut self, entity_manager: &mut EntityManager, delta_time: f32) {
        if !self.is_active {
            return;
        }
        
        // 優先度でシステムをソート
        self.systems.sort_by_key(|sys| sys.priority());
        
        // 実行条件を満たすシステムのみ更新
        for system in &mut self.systems {
            if system.is_active() && system.is_runnable(entity_manager) {
                system.update(entity_manager, delta_time);
            }
        }
    }
    
    /// グループ内の全システムを初期化
    pub fn init_all(&mut self, entity_manager: &mut EntityManager) {
        for system in &mut self.systems {
            system.init(entity_manager);
        }
    }
    
    /// グループ内の全システムをシャットダウン
    pub fn shutdown_all(&mut self, entity_manager: &mut EntityManager) {
        for system in &mut self.systems {
            system.shutdown(entity_manager);
        }
    }
    
    /// グループ名を取得
    pub fn name(&self) -> &str {
        &self.name
    }
    
    /// グループの優先度を取得
    pub fn priority(&self) -> i32 {
        self.priority
    }
    
    /// グループの有効/無効を設定
    pub fn set_active(&mut self, active: bool) {
        self.is_active = active;
    }
    
    /// グループが有効かどうかを取得
    pub fn is_active(&self) -> bool {
        self.is_active
    }
    
    /// グループに特定の名前のシステムが含まれているか確認
    pub fn contains_system(&self, system_name: &str) -> bool {
        self.systems.iter().any(|sys| sys.name() == system_name)
    }
    
    /// グループ内のシステム数を取得
    pub fn system_count(&self) -> usize {
        self.systems.len()
    }
    
    /// 特定の名前のシステムを取得
    pub fn get_system(&self, system_name: &str) -> Option<&dyn System> {
        self.systems.iter()
            .find(|sys| sys.name() == system_name)
            .map(|boxed| boxed.as_ref())
    }
    
    /// 特定の名前のシステムを取得（可変）
    pub fn get_system_mut(&mut self, system_name: &str) -> Option<&mut dyn System> {
        self.systems.iter_mut()
            .find(|sys| sys.name() == system_name)
            .map(|boxed| boxed.as_mut())
    }
} 