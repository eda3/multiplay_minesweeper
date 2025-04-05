/**
 * ユーティリティシステムモジュール
 * 
 * 共通処理や補助機能を提供するシステム群
 */

// サブモジュールをエクスポート
pub mod system_initializer;

// 関数をエクスポート
pub use system_initializer::{initialize_systems, initialize_resources, initialize_entities}; 