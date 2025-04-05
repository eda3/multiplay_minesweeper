/**
 * 入力システムモジュール
 * 
 * ユーザー入力の収集と処理を担当するシステムを含む
 */

// サブモジュールをエクスポート
pub mod ui_input_system;
pub mod gameplay_input_system;
pub mod input_collection_system;
pub mod input_processing_system;

// 各システムをエクスポート
pub use ui_input_system::UIInputSystem;
pub use gameplay_input_system::GameplayInputSystem;
pub use input_collection_system::InputCollectionSystem;
pub use input_processing_system::InputProcessingSystem;

use crate::system::SystemRegistry;
use crate::system::system_registry::SystemPhase;

/// 入力システムを登録する関数
pub fn register_input_systems(registry: &mut crate::system::system_registry::SystemRegistry) {
    // 入力収集システム
    let input_collection = InputCollectionSystem::new();
    registry.add_system(Box::new(input_collection));
    
    // 入力処理システム
    let input_processing = InputProcessingSystem::new();
    registry.add_system(Box::new(input_processing));
    
    // UI入力システム
    let ui_input = UIInputSystem::new();
    registry.add_system(Box::new(ui_input));
    
    // ゲームプレイ入力システム
    let gameplay_input = GameplayInputSystem::new();
    registry.add_system(Box::new(gameplay_input));
} 