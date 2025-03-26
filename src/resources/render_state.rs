/**
 * レンダリング状態リソース
 * 
 * 描画に関する状態を管理するリソース
 */
use wasm_bindgen::JsValue;
use wasm_bindgen::JsCast;
use web_sys::{HtmlCanvasElement, CanvasRenderingContext2d};

/// レンダリング状態リソース
#[derive(Debug)]
pub struct RenderState {
    /// キャンバス要素
    pub canvas: HtmlCanvasElement,
    /// 描画コンテキスト
    pub context: CanvasRenderingContext2d,
    /// キャンバスの幅
    pub canvas_width: f64,
    /// キャンバスの高さ
    pub canvas_height: f64,
    /// デバッグモードかどうか
    pub debug_mode: bool,
    /// グリッド線を表示するかどうか
    pub show_grid: bool,
    /// FPSを表示するかどうか
    pub show_fps: bool,
    /// UIを表示するかどうか
    pub show_ui: bool,
}

impl RenderState {
    /// 新しいレンダリング状態を作成
    pub fn new(canvas: HtmlCanvasElement) -> Result<Self, JsValue> {
        // キャンバスから2Dコンテキストを取得
        let context = canvas
            .get_context("2d")?
            .unwrap()
            .dyn_into::<CanvasRenderingContext2d>()?;
        
        let canvas_width = canvas.width() as f64;
        let canvas_height = canvas.height() as f64;
        
        Ok(Self {
            canvas,
            context,
            canvas_width,
            canvas_height,
            debug_mode: false,
            show_grid: true,
            show_fps: false,
            show_ui: true,
        })
    }
    
    /// キャンバスのサイズを更新
    pub fn update_canvas_size(&mut self) {
        self.canvas_width = self.canvas.width() as f64;
        self.canvas_height = self.canvas.height() as f64;
    }
    
    /// 画面をクリア
    pub fn clear(&self) {
        self.context.set_fill_style(&JsValue::from_str("#f0f0f0"));
        self.context.fill_rect(0.0, 0.0, self.canvas_width, self.canvas_height);
    }
    
    /// デバッグモードの切り替え
    pub fn toggle_debug_mode(&mut self) {
        self.debug_mode = !self.debug_mode;
        
        // デバッグモードがオフになったらFPS表示もオフに
        if !self.debug_mode {
            self.show_fps = false;
        }
    }
    
    /// グリッド表示の切り替え
    pub fn toggle_grid(&mut self) {
        self.show_grid = !self.show_grid;
    }
    
    /// FPS表示の切り替え
    pub fn toggle_fps(&mut self) {
        self.show_fps = !self.show_fps;
        
        // FPS表示をオンにする場合はデバッグモードも有効に
        if self.show_fps {
            self.debug_mode = true;
        }
    }
    
    /// UI表示の切り替え
    pub fn toggle_ui(&mut self) {
        self.show_ui = !self.show_ui;
    }
} 