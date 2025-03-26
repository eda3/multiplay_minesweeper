/**
 * セルエンティティの定義
 * 
 * マインスイーパーのセルをエンティティとして表現
 */
use crate::components::{CellContent, CellState, Position};
use crate::entities::entity::{Entity, EntityId};
use crate::entities::entity_manager::EntityBuilder;

/// セルエンティティのタグ
pub const CELL_TAG: &str = "cell";

/// セルエンティティの種類を表す型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CellEntity {
    /// 地雷セル
    Mine(EntityId),
    /// 通常セル（周囲の地雷数）
    Empty(EntityId, u8),
}

impl CellEntity {
    /// エンティティIDを取得
    pub fn id(&self) -> EntityId {
        match self {
            CellEntity::Mine(id) => *id,
            CellEntity::Empty(id, _) => *id,
        }
    }
    
    /// 地雷かどうか
    pub fn is_mine(&self) -> bool {
        matches!(self, CellEntity::Mine(_))
    }
    
    /// 周囲の地雷数（地雷の場合は0）
    pub fn adjacent_mines(&self) -> u8 {
        match self {
            CellEntity::Mine(_) => 0,
            CellEntity::Empty(_, count) => *count,
        }
    }
}

/// 地雷セルエンティティを作成
pub fn create_mine_cell(builder: EntityBuilder, row: usize, col: usize) -> Entity {
    builder
        .with_component(Position::cell(row, col))
        .with_component(CellContent::Mine)
        .with_component(CellState::new())
        .with_tag(CELL_TAG)
        .with_tag("mine")
        .build()
}

/// 空のセルエンティティを作成（周囲の地雷数付き）
pub fn create_empty_cell(builder: EntityBuilder, row: usize, col: usize, adjacent_mines: u8) -> Entity {
    builder
        .with_component(Position::cell(row, col))
        .with_component(CellContent::Empty(adjacent_mines))
        .with_component(CellState::new())
        .with_tag(CELL_TAG)
        .with_tag("empty")
        .build()
}

/// 汎用セルエンティティを作成（コンテンツに応じて自動的に種類を判断）
pub fn create_cell_entity(builder: EntityBuilder, row: usize, col: usize, content: CellContent) -> Entity {
    match content {
        CellContent::Mine => {
            create_mine_cell(builder, row, col)
        },
        CellContent::Empty(count) => {
            create_empty_cell(builder, row, col, count)
        },
    }
}

/// セルエンティティに対する操作
/// 実際のエンティティマネージャーとエンティティIDを使用してセルを操作
pub mod cell_operations {
    use super::*;
    use crate::entities::entity_manager::EntityManager;
    
    /// セルを開く
    pub fn reveal_cell(manager: &mut EntityManager, id: EntityId) -> bool {
        if let Some(entity) = manager.get_entity_mut(id) {
            if let Some(state) = entity.get_component_mut::<CellState>() {
                // すでに開かれているか、フラグが立っている場合は何もしない
                if state.is_revealed || state.is_flagged {
                    return false;
                }
                
                // セルを開く
                state.is_revealed = true;
                
                // 地雷だったらゲームオーバー
                if let Some(content) = entity.get_component::<CellContent>() {
                    return matches!(content, CellContent::Mine);
                }
            }
        }
        
        false
    }
    
    /// フラグを切り替える
    pub fn toggle_flag(manager: &mut EntityManager, id: EntityId) -> bool {
        if let Some(entity) = manager.get_entity_mut(id) {
            if let Some(state) = entity.get_component_mut::<CellState>() {
                // すでに開かれている場合は何もしない
                if state.is_revealed {
                    return false;
                }
                
                // フラグを切り替え
                state.is_flagged = !state.is_flagged;
                return true;
            }
        }
        
        false
    }
    
    /// セルの内容を取得
    pub fn get_cell_content(manager: &EntityManager, id: EntityId) -> Option<CellContent> {
        manager.get_entity(id)
            .and_then(|entity| entity.get_component::<CellContent>())
            .cloned()
    }
    
    /// セルの状態を取得
    pub fn get_cell_state(manager: &EntityManager, id: EntityId) -> Option<CellState> {
        manager.get_entity(id)
            .and_then(|entity| entity.get_component::<CellState>())
            .cloned()
    }
    
    /// セルの位置を取得
    pub fn get_cell_position(manager: &EntityManager, id: EntityId) -> Option<(usize, usize)> {
        manager.get_entity(id)
            .and_then(|entity| entity.get_component::<Position>())
            .map(|pos| (pos.y as usize, pos.x as usize)) // rowとcolに変換
    }
} 