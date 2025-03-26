/**
 * UIエンティティの定義
 * 
 * ゲームのUI要素をエンティティとして表現
 */
use crate::components::{Position, UIElement, Button};
use crate::entities::entity::{Entity, EntityId};
use crate::entities::entity_manager::EntityBuilder;

/// UIエンティティのタグ
pub const UI_TAG: &str = "ui";
pub const BUTTON_TAG: &str = "button";
pub const TEXT_TAG: &str = "text";

/// UIエンティティの種類
#[derive(Debug, Clone, PartialEq)]
pub enum UIEntityType {
    /// ボタン
    Button(EntityId, String), // ID, button_id
    /// テキスト
    Text(EntityId, String),   // ID, content
    /// アイコン
    Icon(EntityId, String),   // ID, name
}

impl UIEntityType {
    /// エンティティIDを取得
    pub fn id(&self) -> EntityId {
        match self {
            UIEntityType::Button(id, _) => *id,
            UIEntityType::Text(id, _) => *id,
            UIEntityType::Icon(id, _) => *id,
        }
    }
}

/// ボタンUIエンティティを作成
pub fn create_button(builder: EntityBuilder, id: &str, label: &str, x: f64, y: f64, width: f64, height: f64) -> Entity {
    let button = Button::new(id, label, width, height);
    
    builder
        .with_component(Position::new(x, y))
        .with_component(UIElement::Button(button))
        .with_tag(UI_TAG)
        .with_tag(BUTTON_TAG)
        .with_tag(id) // ボタンIDもタグとして使用
        .build()
}

/// テキストUIエンティティを作成
pub fn create_text(builder: EntityBuilder, content: &str, font: &str, size: f64, color: &str, x: f64, y: f64) -> Entity {
    builder
        .with_component(Position::new(x, y))
        .with_component(UIElement::Text {
            content: content.to_string(),
            font: font.to_string(),
            size,
            color: color.to_string(),
        })
        .with_tag(UI_TAG)
        .with_tag(TEXT_TAG)
        .build()
}

/// アイコンUIエンティティを作成
pub fn create_icon(builder: EntityBuilder, name: &str, size: f64, color: &str, x: f64, y: f64) -> Entity {
    builder
        .with_component(Position::new(x, y))
        .with_component(UIElement::Icon {
            name: name.to_string(),
            size,
            color: color.to_string(),
        })
        .with_tag(UI_TAG)
        .with_tag("icon")
        .build()
}

/// 汎用UIエンティティを作成（UIElement種類に応じて自動判断）
pub fn create_ui_entity(builder: EntityBuilder, element: UIElement, x: f64, y: f64) -> Entity {
    match &element {
        UIElement::Button(button) => {
            let id = &button.id;
            let label = &button.label;
            create_button(builder, id, label, x, y, button.width, button.height)
        },
        UIElement::Text { content, font, size, color } => {
            create_text(builder, content, font, *size, color, x, y)
        },
        UIElement::Icon { name, size, color } => {
            create_icon(builder, name, *size, color, x, y)
        },
    }
}

/// UIエンティティに対する操作
/// 実際のエンティティマネージャーとエンティティIDを使用してUI要素を操作
pub mod ui_operations {
    use super::*;
    use crate::entities::entity_manager::EntityManager;
    
    /// ボタンのヒットテスト（クリック判定）
    pub fn is_button_hit(manager: &EntityManager, id: EntityId, x: f64, y: f64) -> bool {
        if let Some(entity) = manager.get_entity(id) {
            if let (Some(position), Some(ui_element)) = (
                entity.get_component::<Position>(),
                entity.get_component::<UIElement>(),
            ) {
                if let UIElement::Button(button) = ui_element {
                    let click_pos = Position::new(x, y);
                    return button.is_hit(&click_pos, position);
                }
            }
        }
        
        false
    }
    
    /// テキストの内容を更新
    pub fn update_text_content(manager: &mut EntityManager, id: EntityId, new_content: &str) -> bool {
        if let Some(entity) = manager.get_entity_mut(id) {
            if let Some(ui_element) = entity.get_component_mut::<UIElement>() {
                if let UIElement::Text { content, .. } = ui_element {
                    *content = new_content.to_string();
                    return true;
                }
            }
        }
        
        false
    }
    
    /// ボタンを探す（IDで）
    pub fn find_button_by_id(manager: &EntityManager, button_id: &str) -> Option<EntityId> {
        // タグでボタンを検索（各ボタンにはIDがタグとして付与されている）
        manager.get_entities_with_tag(button_id)
            .into_iter()
            .find(|id| {
                if let Some(entity) = manager.get_entity(*id) {
                    return entity.has_tag(BUTTON_TAG);
                }
                false
            })
    }
    
    /// クリックされたボタンを探す
    pub fn find_clicked_button(manager: &EntityManager, x: f64, y: f64) -> Option<(EntityId, String)> {
        let button_entities = manager.get_entities_with_tag(BUTTON_TAG);
        
        for id in button_entities {
            if is_button_hit(manager, id, x, y) {
                if let Some(entity) = manager.get_entity(id) {
                    if let Some(UIElement::Button(button)) = entity.get_component::<UIElement>() {
                        return Some((id, button.id.clone()));
                    }
                }
            }
        }
        
        None
    }
    
    /// UIエレメントを非表示にする（削除予約）
    pub fn hide_ui_element(manager: &mut EntityManager, id: EntityId) {
        manager.remove_entity(id);
    }
    
    /// 特定タイプのUI要素を全て取得
    pub fn get_all_ui_elements_by_type(manager: &EntityManager, element_tag: &str) -> Vec<EntityId> {
        let ui_entities = manager.get_entities_with_tag(UI_TAG);
        
        ui_entities.into_iter()
            .filter(|id| {
                if let Some(entity) = manager.get_entity(*id) {
                    return entity.has_tag(element_tag);
                }
                false
            })
            .collect()
    }
} 