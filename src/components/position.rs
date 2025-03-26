/**
 * 位置情報を表すコンポーネント
 * 
 * セル、プレイヤー、UIなど様々なエンティティの位置を表現するためのデータ構造
 */
#[derive(Debug, Clone, Copy)]
pub struct Position {
    /// X座標
    pub x: f64,
    /// Y座標
    pub y: f64,
}

impl Position {
    /// 新しい位置を作成
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
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