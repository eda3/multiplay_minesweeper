/**
 * ボードシステムモジュール
 * 
 * ボード操作に関連するすべてのシステムを提供します
 */

// 各種ボードシステム
pub mod cell_reveal_system;
pub mod cell_reveal_handler_system;
pub mod typed_cell_reveal_system; // 新しい型安全なセル公開システム
// pub mod flag_system; // 未実装
// pub mod board_state_system; // 未実装
// pub mod board_initialization_system; // 未実装
// pub mod board_rendering_system; // 未実装

// 必要なトレイトをインポート
use crate::ecs::system::System;
use crate::entities::EntityManager;
use crate::resources::ResourceManager;
use crate::systems::system_registry::DeltaTime;
use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
use std::any::Any;
use wasm_bindgen::JsValue;

// システムのエクスポート
pub use cell_reveal_system::cell_reveal_system;
pub use cell_reveal_handler_system::CellRevealHandlerSystem;
pub use typed_cell_reveal_system::TypedCellRevealSystem; // 型安全なシステムをエクスポート
// pub use flag_system::flag_system; // 未実装
// pub use board_state_system::board_state_system; // 未実装
// pub use board_initialization_system::board_initialization_system; // 未実装
// pub use board_rendering_system::board_rendering_system; // 未実装

// 静的なシステム名の定義
static CELL_REVEAL_HANDLER_NAME: &'static str = "CellRevealHandlerSystem";
static TYPED_CELL_REVEAL_NAME: &'static str = "TypedCellRevealSystem";

// 静的なシステム関数（CellRevealHandlerSystem用）
pub fn cell_reveal_handler_system_fn(
    entity_manager: &mut EntityManager, 
    resources: &mut HashMap<&'static str, Rc<RefCell<dyn Any>>>, 
    _delta_time: DeltaTime
) -> Result<(), JsValue> {
    let mut handler = CellRevealHandlerSystem::new();
    let mut resource_manager = ResourceManager::new(); // 適切なResourceManagerの作成方法を確認する必要があります
    handler.update(entity_manager, &mut resource_manager);
    Ok(())
}

// 静的なシステム関数（TypedCellRevealSystem用）
pub fn typed_cell_reveal_system_fn(
    entity_manager: &mut EntityManager, 
    resources: &mut HashMap<&'static str, Rc<RefCell<dyn Any>>>, 
    _delta_time: DeltaTime
) -> Result<(), JsValue> {
    let mut typed_system = TypedCellRevealSystem::new();
    let mut resource_manager = ResourceManager::new(); // 適切なResourceManagerの作成方法を確認する必要があります
    typed_system.update(entity_manager, &mut resource_manager);
    Ok(())
}

/// ボードシステムをシステムレジストリに登録する
pub fn register_board_systems(registry: &mut crate::systems::SystemRegistry) {
    // 古いシステム（互換性のため維持）
    let cell_reveal_handler = CellRevealHandlerSystem::new();
    
    // 静的関数を使用する
    registry.register_system(
        CELL_REVEAL_HANDLER_NAME, 
        cell_reveal_handler_system_fn, 
        crate::systems::system_registry::SystemPriority::Update
    );
    
    // 新しい型安全なシステム
    let typed_cell_reveal = TypedCellRevealSystem::new();
    
    // 静的関数を使用する
    registry.register_system(
        TYPED_CELL_REVEAL_NAME, 
        typed_cell_reveal_system_fn, 
        crate::systems::system_registry::SystemPriority::Update
    );
    
    // その他のボードシステムも登録
    // ...
} 