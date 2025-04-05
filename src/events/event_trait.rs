/**
 * イベントトレイト
 * 
 * すべてのイベントが実装する必要のあるトレイト定義。型安全性を確保。
 */
use std::fmt::Debug;
use std::any::Any;

/// すべてのイベントが実装すべきトレイト
pub trait Event: Debug + Clone + Send + Sync + 'static {
    /// イベントの名前を取得する
    fn name(&self) -> &'static str;
    
    /// イベントをAny型にダウンキャストする（内部処理用）
    fn as_any(&self) -> &dyn Any where Self: Sized {
        self
    }
    
    /// イベントをAny型に変換する可変参照（内部処理用）
    fn as_any_mut(&mut self) -> &mut dyn Any where Self: Sized {
        self
    }
    
    /// イベントをボックス化されたAny型にダウンキャストする（内部処理用）
    fn into_any(self) -> Box<dyn Any> where Self: Sized {
        Box::new(self)
    }
}

/// イベントのクローン実装のマクロ
#[macro_export]
macro_rules! impl_event_clone {
    ($type:ty) => {
        fn box_clone(&self) -> Box<dyn Event> {
            Box::new(self.clone())
        }
    };
}

/// イベントの基本実装のマクロ
#[macro_export]
macro_rules! impl_event {
    ($type:ty, $event_type:expr) => {
        impl Event for $type {
            fn event_type(&self) -> &'static str {
                $event_type
            }
            
            fn as_any(&self) -> &dyn std::any::Any {
                self
            }
            
            fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
                self
            }
            
            impl_event_clone!($type);
        }
    };
}

/// イベントの登録用マクロ
#[macro_export]
macro_rules! register_event {
    ($dispatcher:expr, $event_type:ty, $handler:expr) => {
        $dispatcher.subscribe::<$event_type, _>($handler)
    };
} 