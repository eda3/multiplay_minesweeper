/**
 * 描画オプションリソース
 * 
 * ゲーム描画に関するオプションを管理するリソース
 */

/// 描画オプション
#[derive(Debug, Clone)]
pub struct DrawOptions {
    /// キャンバスの幅
    pub width: u32,
    /// キャンバスの高さ
    pub height: u32,
    /// セルのサイズ（ピクセル単位）
    pub cell_size: u32,
    /// 描画スケール
    pub scale: f64,
    /// グリッド線の色
    pub grid_color: String,
    /// 背景色
    pub background_color: String,
    /// 地雷の色
    pub mine_color: String,
    /// 数字の色（0-8）
    pub number_colors: [String; 9],
    /// 未開放セルの色
    pub unrevealed_color: String,
    /// 旗の色
    pub flag_color: String,
    /// ホバー効果の色
    pub hover_color: String,
    /// ゲームオーバー時の色
    pub game_over_color: String,
    /// 勝利時の色
    pub win_color: String,
}

impl Default for DrawOptions {
    fn default() -> Self {
        Self {
            width: 800,
            height: 600,
            cell_size: 30,
            scale: 1.0,
            grid_color: "#555555".to_string(),
            background_color: "#f0f0f0".to_string(),
            mine_color: "#ff0000".to_string(),
            number_colors: [
                "#e6e6e6".to_string(), // 0: 空白
                "#0000ff".to_string(), // 1: 青
                "#008000".to_string(), // 2: 緑
                "#ff0000".to_string(), // 3: 赤
                "#000080".to_string(), // 4: 紺
                "#800000".to_string(), // 5: 茶
                "#008080".to_string(), // 6: 青緑
                "#000000".to_string(), // 7: 黒
                "#808080".to_string(), // 8: グレー
            ],
            unrevealed_color: "#c0c0c0".to_string(),
            flag_color: "#ff0000".to_string(),
            hover_color: "rgba(255, 255, 0, 0.3)".to_string(),
            game_over_color: "rgba(255, 0, 0, 0.3)".to_string(),
            win_color: "rgba(0, 255, 0, 0.3)".to_string(),
        }
    }
}

impl DrawOptions {
    /// 新しい描画オプションを作成
    pub fn new(width: u32, height: u32, cell_size: u32) -> Self {
        Self {
            width,
            height,
            cell_size,
            ..Default::default()
        }
    }
    
    /// 画面サイズを計算
    pub fn screen_size(&self) -> (u32, u32) {
        (self.width, self.height)
    }
    
    /// セルサイズを設定
    pub fn set_cell_size(&mut self, size: u32) {
        self.cell_size = size;
    }
    
    /// スケールを設定
    pub fn set_scale(&mut self, scale: f64) {
        self.scale = scale;
    }
    
    /// グリッド線の色を設定
    pub fn set_grid_color(&mut self, color: &str) {
        self.grid_color = color.to_string();
    }
    
    /// 背景色を設定
    pub fn set_background_color(&mut self, color: &str) {
        self.background_color = color.to_string();
    }
    
    /// 指定した数字の色を取得
    pub fn get_number_color(&self, number: usize) -> &str {
        if number < self.number_colors.len() {
            &self.number_colors[number]
        } else {
            &self.number_colors[0]
        }
    }
    
    /// 数字の色を設定
    pub fn set_number_color(&mut self, number: usize, color: &str) {
        if number < self.number_colors.len() {
            self.number_colors[number] = color.to_string();
        }
    }
}
