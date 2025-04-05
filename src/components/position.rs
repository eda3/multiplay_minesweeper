/**
 * 位置情報を表すコンポーネント
 * 
 * セル、プレイヤー、UIなど様々なエンティティの位置を表現するためのデータ構造
 */
use std::any::Any;
use crate::components::component_trait::Component;
use crate::entities::EntityId;

/// 位置を表すコンポーネント
#[derive(Debug, Clone)]
pub struct Position {
    /// X座標
    pub x: f64,
    /// Y座標
    pub y: f64,
}

impl Position {
    /// 新しい位置コンポーネントを作成
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }
    
    /// ゼロ位置（原点）を作成
    pub fn zero() -> Self {
        Self { x: 0.0, y: 0.0 }
    }
    
    /// セルの位置（グリッド座標）を作成
    pub fn cell(row: usize, col: usize) -> Self {
        Self {
            x: col as f64,
            y: row as f64,
        }
    }
    
    /// 2つの位置の距離を計算
    pub fn distance(&self, other: &Position) -> f64 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        (dx * dx + dy * dy).sqrt()
    }
}

// Componentトレイトの実装
impl Component for Position {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn clone_box(&self) -> Box<dyn Component> {
        Box::new(self.clone())
    }

    fn on_init(&mut self, _entity_id: EntityId) {
        // 初期化時の処理は特にない
    }
} 