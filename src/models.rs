use serde::{Serialize, Deserialize};

/**
 * セルの値を表す列挙型
 */
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CellValue {
    /// 地雷
    Mine,
    /// 空のセル（周囲の地雷数）
    Empty(u8),
}

/**
 * 画面状態を表す列挙型
 */
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Title,  // タイトル画面
    Game,   // ゲーム画面
}

/**
 * プレイヤーモデル
 */
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Player {
    /// プレイヤーID
    pub id: String,
    /// プレイヤー名
    pub name: String,
    /// スコア
    pub score: u32,
} 