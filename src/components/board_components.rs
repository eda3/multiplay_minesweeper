/**
 * ボードシステム用のコンポーネント定義
 */
use std::any::{Any, TypeId};
use std::fmt::{self, Debug};
use crate::components::component_trait::Component;
use crate::entities::EntityId;
use crate::models::CellValue;

/// セルの状態を表す列挙型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CellState {
    /// 隠れた状態
    Hidden,
    /// フラグが立てられた状態
    Flagged,
    /// 公開された状態
    Revealed,
    /// 疑問符が付けられた状態
    Questioned,
}

/**
 * セルの状態を表すコンポーネント
 */
#[derive(Debug, Clone)]
pub struct CellStateComponent {
    /// セルの現在の状態
    pub state: CellState,
}

impl Component for CellStateComponent {
    // Componentトレイトのデフォルト実装を使用
}

impl Default for CellStateComponent {
    fn default() -> Self {
        Self {
            state: CellState::Hidden,
        }
    }
}

/**
 * セルの内容を表すコンポーネント
 */
#[derive(Debug, Clone)]
pub struct CellContentComponent {
    /// セルの値（地雷または周囲の地雷数）
    pub value: CellValue,
}

impl Component for CellContentComponent {
    // Componentトレイトのデフォルト実装を使用
}

impl Default for CellContentComponent {
    fn default() -> Self {
        Self {
            value: CellValue::Empty(0),
        }
    }
}

/**
 * グリッド上の位置を表すコンポーネント
 */
#[derive(Debug, Clone)]
pub struct GridPositionComponent {
    /// 行（Y座標）
    pub row: usize,
    /// 列（X座標）
    pub col: usize,
}

impl Component for GridPositionComponent {
    // Componentトレイトのデフォルト実装を使用
}

impl Default for GridPositionComponent {
    fn default() -> Self {
        Self {
            row: 0,
            col: 0,
        }
    }
} 