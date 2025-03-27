/**
 * Entity Component System (ECS)
 * 
 * ゲームエンジンのコアとなるECSアーキテクチャを実装するモジュール
 */

pub mod world;

pub use world::World;
// システム関連のモジュールを再エクスポート
pub use crate::system::{System, SystemRegistry};
pub use crate::system::system_registry::SystemPhase;

/**
 * ECSパターンの基本モジュール
 * 
 * Entity-Component-System アーキテクチャの基本構造を提供します
 */

/// System - ゲームロジックを実装するためのシステムインターフェース
pub mod system; 