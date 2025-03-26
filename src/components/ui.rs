/**
 * UI関連のコンポーネント
 * 
 * ゲームのUI要素を表すデータ構造
 */
use crate::components::Position;

/// UI要素の種類
#[derive(Debug, Clone)]
pub enum UIElement {
    /// ボタン
    Button(Button),
    /// テキスト
    Text {
        content: String,
        font: String,
        size: f64,
        color: String,
    },
    /// アイコン
    Icon {
        name: String,
        size: f64,
        color: String,
    },
}

/// ボタンコンポーネント
#[derive(Debug, Clone)]
pub struct Button {
    /// ボタンの幅
    pub width: f64,
    /// ボタンの高さ
    pub height: f64,
    /// ボタンのラベル
    pub label: String,
    /// 背景色
    pub bg_color: String,
    /// テキスト色
    pub text_color: String,
    /// 丸み
    pub border_radius: f64,
    /// ボタンのID（識別用）
    pub id: String,
}

impl Button {
    /// 新しいボタンを作成
    pub fn new(id: &str, label: &str, width: f64, height: f64) -> Self {
        Self {
            width,
            height,
            label: label.to_string(),
            bg_color: "#4a6bdf".to_string(),
            text_color: "#ffffff".to_string(),
            border_radius: 8.0,
            id: id.to_string(),
        }
    }
    
    /// プライマリボタンのスタイル
    pub fn primary(mut self) -> Self {
        self.bg_color = "#4a6bdf".to_string();
        self.text_color = "#ffffff".to_string();
        self
    }
    
    /// セカンダリボタンのスタイル
    pub fn secondary(mut self) -> Self {
        self.bg_color = "#6c757d".to_string();
        self.text_color = "#ffffff".to_string();
        self
    }
    
    /// 危険ボタンのスタイル
    pub fn danger(mut self) -> Self {
        self.bg_color = "#dc3545".to_string();
        self.text_color = "#ffffff".to_string();
        self
    }
    
    /// 成功ボタンのスタイル
    pub fn success(mut self) -> Self {
        self.bg_color = "#28a745".to_string();
        self.text_color = "#ffffff".to_string();
        self
    }
    
    /// ボタンの位置を含めたヒットテスト
    pub fn is_hit(&self, position: &Position, button_pos: &Position) -> bool {
        let half_width = self.width / 2.0;
        let half_height = self.height / 2.0;
        
        position.x >= button_pos.x - half_width &&
        position.x <= button_pos.x + half_width &&
        position.y >= button_pos.y - half_height &&
        position.y <= button_pos.y + half_height
    }
} 