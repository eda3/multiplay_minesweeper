/**
 * レンダリングリソース
 * 
 * キャンバスのコンテキストとレンダリング状態を管理する
 */
use super::resource_trait::Resource;
use wasm_bindgen::prelude::*;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};
use std::any::Any;

/// レンダリングリソース
///
/// キャンバスのコンテキストとレンダリング状態を管理します。
/// 描画スケールやキャンバスサイズなどのプロパティを含みます。
pub struct RenderResource {
    /// キャンバス描画コンテキスト
    pub canvas: CanvasRenderingContext2d,
    /// キャンバスの幅
    pub width: u32,
    /// キャンバスの高さ
    pub height: u32,
    /// 描画スケール
    pub scale: f64,
}

impl RenderResource {
    /// 新しいレンダーリソースを作成します
    pub fn new(canvas: CanvasRenderingContext2d, width: u32, height: u32) -> Self {
        Self {
            canvas,
            width,
            height,
            scale: 1.0,
        }
    }
    
    /// 描画コンテキストを取得します
    pub fn context(&self) -> &CanvasRenderingContext2d {
        &self.canvas
    }
    
    /// キャンバスのサイズを取得
    pub fn canvas_size(&self) -> (u32, u32) {
        (self.width, self.height)
    }
    
    /// キャンバスをクリア
    pub fn clear(&self, color: &str) {
        self.canvas.set_fill_style(&color.into());
        self.canvas.fill_rect(0.0, 0.0, self.width as f64, self.height as f64);
    }
    
    /// スケールを設定
    pub fn set_scale(&mut self, scale: f64) {
        self.scale = scale;
        self.canvas.set_transform(scale, 0.0, 0.0, scale, 0.0, 0.0).unwrap();
    }
    
    // 互換性のためのメソッド
    
    /// 描画コンテキストを取得（互換性のため）
    pub fn get_context(&self) -> &CanvasRenderingContext2d {
        self.context()
    }
    
    /// キャンバスのサイズを取得（互換性のため）
    pub fn get_canvas_size(&self) -> (u32, u32) {
        self.canvas_size()
    }
}

// Resource実装は削除します - すでにリソーストレイトでブランケット実装されていたため不要

// CanvasRenderingContext2dはCloneを実装していないため、RenderResourceもCloneできません 