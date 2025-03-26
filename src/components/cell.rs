/**
 * セル関連のコンポーネント
 * 
 * マインスイーパーのセルに関連するデータ構造を定義します
 */
use serde::{Serialize, Deserialize};

/// セルの内容（地雷または数字）
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CellContent {
    /// 地雷セル
    Mine,
    /// 空のセル（周囲の地雷数を含む）
    Empty(u8),
}

/// セルの状態
#[derive(Debug, Clone, Default)]
pub struct CellState {
    /// セルが開かれているかどうか
    pub is_revealed: bool,
    /// セルにフラグが立てられているかどうか
    pub is_flagged: bool,
}

impl CellState {
    /// 新しいセル状態を作成
    pub fn new() -> Self {
        Self {
            is_revealed: false,
            is_flagged: false,
        }
    }
    
    /// 開かれた状態のセルを作成
    pub fn revealed() -> Self {
        Self {
            is_revealed: true,
            is_flagged: false,
        }
    }
    
    /// フラグが立てられた状態のセルを作成
    pub fn flagged() -> Self {
        Self {
            is_revealed: false,
            is_flagged: true,
        }
    }
} 