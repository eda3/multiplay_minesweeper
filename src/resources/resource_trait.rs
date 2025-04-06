/**
 * リソーストレイト
 * 
 * ECSシステムで使用するリソースの基本トレイト
 */
use std::any::Any;
use std::collections::HashSet;

/// リソーストレイト
/// ECSのリソースを表す基本トレイト
pub trait Resource: 'static {
    /// Anyトレイトへの変換（ダウンキャスト用）
    fn as_any(&self) -> &dyn Any;
    
    /// Anyトレイトへの可変変換（ダウンキャスト用）
    fn as_any_mut(&mut self) -> &mut dyn Any;
    
    /// リソースの初期化処理
    /// 
    /// リソースマネージャに追加された後に呼び出される。
    /// 他のリソースに依存する初期化処理をここで実装する。
    fn initialize(&mut self) -> Result<(), String> {
        Ok(()) // デフォルト実装は何もしない
    }
    
    /// リソースの終了処理
    /// 
    /// リソースがリソースマネージャから削除される前に呼び出される。
    /// リソースが確保したリソースの解放などをここで実装する。
    fn shutdown(&mut self) -> Result<(), String> {
        Ok(()) // デフォルト実装は何もしない
    }
    
    /// このリソースが依存する他のリソースタイプのリスト
    /// 
    /// 返されたリストに含まれるリソースタイプは、このリソースの前に
    /// 初期化されることが保証される。
    fn dependencies(&self) -> HashSet<std::any::TypeId> {
        HashSet::new() // デフォルト実装は依存関係なし
    }
}

// Blanket実装でResource traitをすべてのAny型に実装
impl<T: Any + 'static> Resource for T {
    fn as_any(&self) -> &dyn Any {
        self
    }
    
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
} 