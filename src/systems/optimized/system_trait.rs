/**
 * 拡張システムトレイト
 * 
 * より柔軟で機能的なシステムトレイトの定義
 */
use crate::entities::EntityManager;
use std::fmt::Debug;

/// 拡張されたシステムトレイト
pub trait System: Send + Sync + 'static + Debug {
    /// システムの一意の名前
    fn name(&self) -> &str;
    
    /// システムの初期化時に呼ばれる
    fn init(&mut self, _entity_manager: &mut EntityManager) {}
    
    /// メインの更新ロジック
    fn update(&mut self, entity_manager: &mut EntityManager, delta_time: f32);
    
    /// システムが破棄される前に呼ばれる
    fn shutdown(&mut self, _entity_manager: &mut EntityManager) {}
    
    /// このシステムが依存する他のシステムの名前のリスト
    fn dependencies(&self) -> Vec<&str> {
        Vec::new()
    }
    
    /// システムが現在実行可能かを判断する条件
    fn is_runnable(&self, _entity_manager: &EntityManager) -> bool {
        true
    }
    
    /// システムの優先度（低いほど先に実行）
    fn priority(&self) -> i32 {
        0
    }
    
    /// システムがアクティブかどうか
    fn is_active(&self) -> bool {
        true
    }
    
    /// システムの有効/無効を切り替える
    fn set_active(&mut self, _active: bool) {
        // デフォルト実装では何もしない
        // 実際の実装はこれをオーバーライドする必要がある
    }
} 