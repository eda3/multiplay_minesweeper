/**
 * レンダリング状態リソース
 * 
 * レンダリングに関する状態を管理するリソース
 */

use wasm_bindgen::JsValue;
use wasm_bindgen::JsCast;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};

#[derive(Debug)]
pub struct RenderResource {
    /// キャンバス要素
    pub canvas: Option<HtmlCanvasElement>,
    /// 2Dレンダリングコンテキスト
    pub context: Option<CanvasRenderingContext2d>,
    /// キャンバスの幅
    pub width: u32,
    /// キャンバスの高さ
    pub height: u32,
    /// セルのサイズ（ピクセル単位）
    pub cell_size: u32,
    /// スケール係数（拡大・縮小用）
    pub scale: f64,
}

impl Default for RenderResource {
    fn default() -> Self {
        Self {
            canvas: None,
            context: None,
            width: 800,
            height: 600,
            cell_size: 30,
            scale: 1.0,
        }
    }
}

impl RenderResource {
    /// 新しいレンダリングリソースを初期化
    pub fn new(width: u32, height: u32, cell_size: u32) -> Self {
        Self {
            canvas: None,
            context: None,
            width,
            height,
            cell_size,
            scale: 1.0,
        }
    }
    
    /// キャンバスとコンテキストを設定
    pub fn set_canvas(&mut self, canvas: HtmlCanvasElement) -> Result<(), JsValue> {
        // キャンバスのサイズを設定
        canvas.set_width(self.width);
        canvas.set_height(self.height);
        
        // 2Dコンテキストを取得
        let context = canvas
            .get_context("2d")?
            .ok_or_else(|| JsValue::from_str("Failed to get canvas context"))?
            .dyn_into::<CanvasRenderingContext2d>()
            .map_err(|_| JsValue::from_str("Failed to convert to CanvasRenderingContext2d"))?;
        
        self.canvas = Some(canvas);
        self.context = Some(context);
        
        Ok(())
    }
    
    /// コンテキストを取得（参照）
    pub fn get_context(&self) -> Option<&CanvasRenderingContext2d> {
        self.context.as_ref()
    }
    
    /// キャンバスをクリア
    pub fn clear(&self) -> Result<(), JsValue> {
        if let Some(context) = &self.context {
            context.clear_rect(0.0, 0.0, self.width as f64, self.height as f64);
        }
        Ok(())
    }
    
    /// 画面サイズを更新
    pub fn resize(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
        
        // キャンバスも更新
        if let Some(canvas) = &mut self.canvas {
            canvas.set_width(width);
            canvas.set_height(height);
        }
    }
    
    /// セルサイズを変更
    pub fn set_cell_size(&mut self, cell_size: u32) {
        self.cell_size = cell_size;
    }
    
    /// スケールを設定
    pub fn set_scale(&mut self, scale: f64) {
        self.scale = scale;
    }
    
    /// キャンバスのサイズを取得
    pub fn get_canvas_size(&self) -> (u32, u32) {
        (self.width, self.height)
    }
} 