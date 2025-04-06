use std::any::Any;

use crate::events::EventData;

/// イベントロガーのトレイト
pub trait EventLogger: Send + Sync {
    /// 型消去されたイベントをログに記録
    fn log_event(&self, event: &dyn Any);
    
    /// 具体的なイベントデータをログに記録
    fn log_event_data(&self, event_data: &EventData);
    
    /// ロガーの名前を取得
    fn name(&self) -> &str;
}

/// コンソールにログを出力するシンプルなロガー
pub struct ConsoleEventLogger {
    name: String,
    verbose: bool,
}

impl ConsoleEventLogger {
    /// 新しいコンソールロガーを作成
    pub fn new(name: &str, verbose: bool) -> Self {
        Self {
            name: name.to_string(),
            verbose,
        }
    }
}

impl EventLogger for ConsoleEventLogger {
    fn log_event(&self, event: &dyn Any) {
        if self.verbose {
            println!("[{}] イベント受信: {:?}", self.name, event.type_id());
        }
    }
    
    fn log_event_data(&self, event_data: &EventData) {
        println!("[{}] イベント: {} ({})", self.name, event_data.name(), event_data.timestamp());
    }
    
    fn name(&self) -> &str {
        &self.name
    }
} 