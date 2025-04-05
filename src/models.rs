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
    /// X座標
    pub x: f64,
    /// Y座標
    pub y: f64,
    /// プレイヤーカラー
    pub color: String,
    /// ローカルプレイヤーかどうか
    pub is_local: bool,
    /// ホストプレイヤーかどうか
    pub is_host: bool,
    /// 生存しているかどうか
    pub is_alive: bool,
    /// 公開したセル数
    pub cells_revealed: u32,
}

impl Player {
    /// 新しいプレイヤーを作成
    pub fn new(id: String, name: String) -> Self {
        Self {
            id,
            name,
            score: 0,
            x: 0.0,
            y: 0.0,
            color: "#0000FF".to_string(),
            is_local: false,
            is_host: false,
            is_alive: true,
            cells_revealed: 0,
        }
    }
} 