/**
 * 型安全なボード関連イベント
 * 
 * ボードの状態やセルの操作に関連する型安全なイベント定義
 * 新しいマクロシステムを使用して定義
 */
use serde::{Serialize, Deserialize};
use crate::models::cell::{CellState, CellValue};
use crate::models::coordinate::Coordinate;

// マクロをインポート
use crate::{define_event, event_handler, typed_event_processor};
use crate::resources::ResourceManager;
use crate::events::typed_event_bus::EventPriority;

// 型安全なセル状態変更イベント
define_event! {
    SafeCellStateChangeEvent,
    coord: Coordinate,
    new_state: CellState,
    old_state: CellState,
    value: CellValue
}

// 型安全なボード初期化イベント
define_event! {
    SafeBoardInitializedEvent,
    width: u32,
    height: u32,
    mine_count: u32,
    first_click: Option<Coordinate>
}

// 型安全なセル開示イベント
define_event! {
    SafeCellRevealedEvent,
    coord: Coordinate,
    value: CellValue,
    is_chain: bool
}

// イベントハンドラの定義例（マクロを使わない方法）
pub fn handle_cell_revealed(event: &SafeCellRevealedEvent) {
    log::info!(
        "セルが開示されました: ({}, {}) 値={:?}, チェーン反応={}", 
        event.coord.x, event.coord.y, event.value, event.is_chain
    );
}

// TypedEventHandler作成関数
pub fn register_handle_cell_revealed(event_bus: &crate::events::typed_event_bus::TypedEventBus) 
    -> crate::events::typed_handler::TypedEventHandler<SafeCellRevealedEvent> {
    let handler = move |event: &SafeCellRevealedEvent| {
        handle_cell_revealed(event)
    };
    
    event_bus.subscribe(
        "handle_cell_revealed",
        handler
    )
}

// 型安全なイベントプロセッサの定義例
typed_event_processor! {
    /// ボード初期化イベントのプロセッサ
    /// 
    /// ボードが初期化されたときの処理を実装
    fn process_board_initialized(
        event: &SafeBoardInitializedEvent, 
        resources: &mut ResourceManager
    ) -> Result<(), String> {
        log::info!("ボードが初期化されました: {}x{}, 地雷数={}", 
            event.width, event.height, event.mine_count);
            
        // 実際の処理はここに実装
        // ...
        
        Ok(())
    }
}

// 複数のパラメータを持つイベントハンドラ例（マクロを使わない方法）
pub fn handle_cell_state_change(
    event: &SafeCellStateChangeEvent, 
    debug_mode: bool
) {
    if debug_mode {
        log::debug!(
            "セル状態変更: ({}, {}) {:?} -> {:?}, 値={:?}", 
            event.coord.x, event.coord.y, 
            event.old_state, event.new_state, event.value
        );
    }
}

// TypedEventHandler作成関数
pub fn register_handle_cell_state_change(
    event_bus: &crate::events::typed_event_bus::TypedEventBus,
    debug_mode: bool
) -> crate::events::typed_handler::TypedEventHandler<SafeCellStateChangeEvent> {
    let debug_mode_clone = debug_mode.clone();
    let handler = move |event: &SafeCellStateChangeEvent| {
        handle_cell_state_change(event, debug_mode_clone)
    };
    
    event_bus.subscribe(
        "handle_cell_state_change",
        handler
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::typed_event_bus::TypedEventBus;
    
    #[test]
    fn test_safe_event_creation() {
        // イベント作成のテスト
        let event = SafeCellRevealedEvent::new(
            Coordinate::new(5, 10),
            CellValue::Empty(3),
            false,
        );
        
        // フィールドの検証
        assert_eq!(event.coord.x, 5);
        assert_eq!(event.coord.y, 10);
        assert_eq!(event.value, CellValue::Empty(3));
        assert_eq!(event.is_chain, false);
    }
    
    #[test]
    fn test_event_handler_registration() {
        let event_bus = TypedEventBus::new();
        
        // ハンドラを登録
        let handler = register_handle_cell_revealed(&event_bus);
        
        // ハンドラIDが有効であることを確認
        assert!(event_bus.handler_count::<SafeCellRevealedEvent>() > 0);
        
        // イベントを発行
        let event = SafeCellRevealedEvent::new(
            Coordinate::new(3, 7),
            CellValue::Mine,
            true,
        );
        
        // 優先度を指定してイベントを発行
        event_bus.publish_with_priority(event, EventPriority::High);
    }
} 