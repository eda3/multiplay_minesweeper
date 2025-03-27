/**
 * 最適化されたシステムレジストリモジュール
 * 
 * システムの登録、依存関係解決、実行を管理する機能群
 */

// モジュールをエクスポート
mod system_trait;
mod system_group;
mod system_registry;
mod parallel_executor;
mod system_scheduler;
mod resource_dependency;
#[cfg(test)]
mod tests;

// 公開コンポーネント
pub use system_trait::System;
pub use system_group::SystemGroup;
pub use system_registry::SystemRegistry;
pub use parallel_executor::ParallelExecutor;
pub use system_scheduler::{SystemScheduler, RateControlledSystem};
pub use resource_dependency::{ResourceDependency, ReadResource, WriteResource, ResourceSet, NoResources}; 