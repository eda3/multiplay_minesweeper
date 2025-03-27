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