/**
 * ECSパターンのエンティティ定義
 * 
 * このモジュールではマインスイーパーゲームのエンティティ
 * （一意のIDとコンポーネントの集合）を定義します
 */

// サブモジュールをエクスポート
mod entity;
mod entity_manager;
mod entity_id_generator;
mod cell_entity;
mod player_entity;
mod ui_entity;

// テストモジュール
#[cfg(test)]
mod tests;

// エンティティ関連の型を再エクスポート
pub use entity::{Entity, EntityId};
pub use entity_manager::{EntityManager, EntityBuilder, Hierarchy};
pub use entity_id_generator::EntityIdGenerator;
pub use cell_entity::{CellEntity, create_cell_entity};
pub use player_entity::{PlayerEntity, create_player_entity};
pub use ui_entity::{UIEntityType, create_ui_entity}; 