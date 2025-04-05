/**
 * リソーストレイト
 * 
 * ECSシステムで使用するリソースの基本トレイト
 */
use std::any::Any;

/// リソーストレイト
/// ECSのリソースを表す基本トレイト
pub trait Resource: 'static {
    /// Anyトレイトへの変換（ダウンキャスト用）
    fn as_any(&self) -> &dyn Any;
    
    /// Anyトレイトへの可変変換（ダウンキャスト用）
    fn as_any_mut(&mut self) -> &mut dyn Any;
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