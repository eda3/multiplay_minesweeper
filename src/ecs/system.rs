/**
 * ECSシステムインターフェース
 * 
 * ゲームロジックを実装するためのシステムトレイトを定義します
 */
use crate::entities::EntityManager;
use crate::resources::ResourceManager;
use std::fmt::Debug;

/// システムの実行結果
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SystemResult {
    /// 正常終了
    Ok,
    /// 変更があり、依存システムの再実行が必要
    NeedRerun,
    /// エラー発生
    Error,
}

/// システムトレイト
/// ゲームロジックを実装するためのインターフェース
pub trait System: Send + Sync + 'static {
    /// システムの更新処理
    fn update(&mut self, entity_manager: &mut EntityManager, resources: &mut ResourceManager) -> SystemResult;
    
    /// システムの初期化処理（オプション）
    fn initialize(&mut self, _entity_manager: &mut EntityManager, _resources: &mut ResourceManager) -> SystemResult {
        SystemResult::Ok
    }
    
    /// システムのクリーンアップ処理（オプション）
    fn cleanup(&mut self, _entity_manager: &mut EntityManager, _resources: &mut ResourceManager) -> SystemResult {
        SystemResult::Ok
    }
    
    /// システム名を取得（オプション）
    fn name(&self) -> &str {
        "UnnamedSystem"
    }
} 