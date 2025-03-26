/**
 * コンポーネント定義
 * 
 * エンティティに付与できるコンポーネントをまとめたモジュール
 */
use std::rc::Rc;
use web_sys::HtmlImageElement;

/// 位置コンポーネント
#[derive(Debug, Clone, Copy)]
pub struct Position {
    /// X座標
    pub x: f64,
    /// Y座標
    pub y: f64,
}

/// 速度コンポーネント
#[derive(Debug, Clone, Copy)]
pub struct Velocity {
    /// X方向の速度
    pub vx: f64,
    /// Y方向の速度
    pub vy: f64,
}

/// プレイヤーコンポーネント
#[derive(Debug, Clone)]
pub struct Player {
    /// プレイヤーID
    pub id: String,
    /// プレイヤー名
    pub name: String,
    /// プレイヤーカラー
    pub color: String,
}

/// マウス状態コンポーネント
#[derive(Debug, Clone)]
pub struct MouseState {
    /// マウスボタンが押されているか
    pub is_pressed: bool,
    /// 押されているボタン（0: 左, 1: 中, 2: 右）
    pub button: u16,
}

/// 描画タイプ
#[derive(Debug, Clone)]
pub enum RenderType {
    /// 画像描画
    Image(Rc<HtmlImageElement>),
    /// 円描画
    Circle {
        /// 半径
        radius: f64,
        /// 色
        color: String,
    },
    /// テキスト描画
    Text {
        /// テキスト内容
        text: String,
        /// フォント指定
        font: String,
        /// 色
        color: String,
    },
    /// 矩形描画
    Rectangle {
        /// 幅
        width: f64,
        /// 高さ
        height: f64,
        /// 色
        color: String,
    },
}

/// 描画コンポーネント
#[derive(Debug, Clone)]
pub struct Renderable {
    /// 描画タイプ
    pub render_type: RenderType,
    /// 描画優先度（値が大きいほど前面に描画）
    pub z_index: i32,
}

/// ボードコンポーネント
#[derive(Debug, Clone)]
pub struct Board {
    /// ボードの幅（セル数）
    pub width: usize,
    /// ボードの高さ（セル数）
    pub height: usize,
    /// 地雷の数
    pub mine_count: usize,
    /// セルの値（-1: 地雷, 0: 空白, 1-8: 周囲の地雷数）
    pub cells: Vec<i8>,
    /// セルが公開されているかどうか
    pub revealed: Vec<bool>,
    /// セルにフラグが立てられているかどうか
    pub flagged: Vec<bool>,
    /// ゲームオーバーフラグ
    pub game_over: bool,
    /// 勝利フラグ
    pub game_won: bool,
}

/// アニメーションコンポーネント
#[derive(Debug, Clone)]
pub struct Animation {
    /// 現在のフレーム
    pub current_frame: usize,
    /// 総フレーム数
    pub total_frames: usize,
    /// フレーム間の時間（秒）
    pub frame_time: f64,
    /// 経過時間
    pub elapsed_time: f64,
    /// ループするかどうか
    pub looping: bool,
    /// アニメーション完了したかどうか
    pub completed: bool,
}

/// クリック可能コンポーネント
#[derive(Debug, Clone)]
pub struct Clickable {
    /// クリック領域の幅
    pub width: f64,
    /// クリック領域の高さ
    pub height: f64,
    /// ホバー状態かどうか
    pub is_hovered: bool,
    /// クリック状態かどうか
    pub is_clicked: bool,
    /// クリック時のコールバックID
    pub callback_id: usize,
}

/// タグコンポーネント（エンティティの分類用）
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tag(pub String);

impl Board {
    /// 新しいボードを作成
    pub fn new(width: usize, height: usize, mine_count: usize) -> Self {
        let size = width * height;
        Self {
            width,
            height,
            mine_count,
            cells: vec![0; size],
            revealed: vec![false; size],
            flagged: vec![false; size],
            game_over: false,
            game_won: false,
        }
    }
    
    /// 特定の座標のセルインデックスを取得
    pub fn get_index(&self, x: usize, y: usize) -> Option<usize> {
        if x < self.width && y < self.height {
            Some(y * self.width + x)
        } else {
            None
        }
    }
} 