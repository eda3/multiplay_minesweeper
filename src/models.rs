use serde::{Serialize, Deserialize};

/**
 * セルの状態を表す列挙型
 */
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum CellValue {
    Mine,           // 地雷
    Empty(u8),      // 空白（周囲の地雷数）
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
 * プレイヤー情報を表す構造体
 */
#[derive(Clone, Serialize, Deserialize)]
pub struct Player {
    pub id: String,      // プレイヤーID
    pub x: f64,          // X座標
    pub y: f64,          // Y座標
    pub color: String,   // カーソルの色
} 