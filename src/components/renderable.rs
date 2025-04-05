use std::any::Any;
use std::fmt::Debug;
use crate::components::component_trait::Component;

/// レンダリング方法を指定するコンポーネント
#[derive(Debug, Clone)]
pub enum RenderType {
    /// 画像レンダリング
    Image(String),
    /// 円形レンダリング
    Circle {
        radius: f64,
        color: String,
    },
    /// テキストレンダリング
    Text {
        text: String,
        font: String,
        color: String,
    },
    /// 矩形レンダリング
    Rectangle {
        width: f64,
        height: f64,
        color: String,
    },
}

impl Component for RenderType {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn clone_box(&self) -> Box<dyn Component> {
        Box::new(self.clone())
    }
} 