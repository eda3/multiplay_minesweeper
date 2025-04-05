use serde::{Serialize, Deserialize};

/**
 * 画面状態を表す列挙型
 */
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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

/// 座標モジュール
pub mod coordinate {
    use serde::{Serialize, Deserialize};
    
    /**
     * 座標を表す構造体
     */
    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub struct Coordinate {
        /// 行（Y座標）
        pub row: u32,
        /// 列（X座標）
        pub col: u32,
    }

    impl Coordinate {
        /// 新しい座標を作成
        pub fn new(row: u32, col: u32) -> Self {
            Self { row, col }
        }
    }
}

/// セル関連のモジュール
pub mod cell {
    use serde::{Serialize, Deserialize};
    
    /**
     * セルの値を表す列挙型
     */
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    pub enum CellValue {
        /// 地雷
        Mine,
        /// 空のセル（周囲の地雷数）
        Empty(u8),
        /// 未確定の値（イベント処理用）
        Unknown,
    }
    
    /**
     * セルの状態を表す列挙型
     */
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    pub enum CellState {
        /// 隠れている（まだ開かれていない）
        Hidden,
        /// 開示済み
        Revealed,
        /// フラグ付き
        Flagged,
        /// クエスチョンマーク付き
        Questioned,
    }
    
    /**
     * セル構造体（値と状態を含む）
     */
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Cell {
        /// セルの値
        pub value: CellValue,
        /// セルの状態
        pub state: CellState,
        /// 開示済みかどうか
        pub is_revealed: bool,
        /// フラグが立っているかどうか
        pub is_flagged: bool,
    }
    
    impl Cell {
        /// 新しいセルを作成
        pub fn new(value: CellValue) -> Self {
            Self {
                value,
                state: CellState::Hidden,
                is_revealed: false,
                is_flagged: false,
            }
        }
        
        /// セルを開示する
        pub fn reveal(&mut self) -> bool {
            if self.state == CellState::Hidden {
                self.state = CellState::Revealed;
                self.is_revealed = true;
                true
            } else {
                false
            }
        }
        
        /// フラグを切り替える
        pub fn toggle_flag(&mut self) -> bool {
            if self.state == CellState::Hidden {
                self.state = CellState::Flagged;
                self.is_flagged = true;
                true
            } else if self.state == CellState::Flagged {
                self.state = CellState::Hidden;
                self.is_flagged = false;
                true
            } else {
                false
            }
        }
    }
}

/// 難易度モジュール
pub mod difficulty {
    use serde::{Serialize, Deserialize};
    
    /**
     * 難易度設定を表す列挙型
     */
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    pub enum Difficulty {
        /// 初級
        Beginner,
        /// 中級
        Intermediate,
        /// 上級
        Expert,
        /// カスタム
        Custom,
    }
    
    impl Difficulty {
        /// 難易度に基づいたボードサイズと地雷数を取得
        pub fn get_board_config(&self, custom_width: Option<u32>, custom_height: Option<u32>, custom_mines: Option<u32>) -> (u32, u32, u32) {
            match self {
                Difficulty::Beginner => (9, 9, 10),
                Difficulty::Intermediate => (16, 16, 40),
                Difficulty::Expert => (30, 16, 99),
                Difficulty::Custom => {
                    let width = custom_width.unwrap_or(16);
                    let height = custom_height.unwrap_or(16);
                    let mines = custom_mines.unwrap_or((width * height) / 6);
                    (width, height, mines)
                }
            }
        }
        
        /// 難易度の名前を取得
        pub fn name(&self) -> &'static str {
            match self {
                Difficulty::Beginner => "初級",
                Difficulty::Intermediate => "中級",
                Difficulty::Expert => "上級",
                Difficulty::Custom => "カスタム",
            }
        }
    }
} 