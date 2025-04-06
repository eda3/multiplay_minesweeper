/**
 * イベントキューリソース
 * 
 * システム間で通信するためのイベントキューを管理
 */
use std::collections::VecDeque;

/// ゲームイベントの種類
#[derive(Debug, Clone)]
pub enum GameEvent {
    /// セルを公開するイベント（行, 列）
    CellReveal(usize, usize),
    /// フラグを切り替えるイベント（行, 列）
    FlagToggle(usize, usize),
    /// ゲームをリセットするイベント
    GameReset,
    /// ゲームを一時停止/再開するイベント
    PauseToggle,
    /// マウス移動イベント（x, y）
    MouseMove(f64, f64),
    /// UI要素クリックイベント（要素ID）
    UIClick(String),
    /// 設定変更イベント（設定名, 値）
    SettingChange(String, String),
}

/// イベントキューリソース
#[derive(Debug, Default)]
pub struct EventQueueResource {
    /// イベントのキュー
    events: VecDeque<GameEvent>,
}

impl EventQueueResource {
    /// 新しいイベントキューリソースを作成
    pub fn new() -> Self {
        Self {
            // WASM環境でのメモリ効率化のため、初期容量16に設定
            // マインスイーパーでは1フレームあたりの平均イベント数は10前後と想定
            events: VecDeque::with_capacity(16),
        }
    }
    
    /// 特定の初期容量でイベントキューリソースを作成
    /// 
    /// # 引数
    /// 
    /// * `capacity` - キューの初期容量
    /// 
    /// # 例
    /// 
    /// ```
    /// // 大量のイベントが予想される場合は大きめの容量を指定
    /// let queue = EventQueueResource::with_capacity(64);
    /// ```
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            events: VecDeque::with_capacity(capacity),
        }
    }
    
    /// イベントをキューに追加
    pub fn push_event(&mut self, event: GameEvent) {
        self.events.push_back(event);
    }
    
    /// キューからイベントを取得（消費）
    pub fn poll_event(&mut self) -> Option<GameEvent> {
        self.events.pop_front()
    }
    
    /// キューからイベントを覗き見（消費しない）
    pub fn peek_event(&self) -> Option<&GameEvent> {
        self.events.front()
    }
    
    /// キュー内のイベント数を取得
    pub fn event_count(&self) -> usize {
        self.events.len()
    }
    
    /// キューが空かどうかをチェック
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }
    
    /// キューを空にする
    pub fn clear(&mut self) {
        self.events.clear();
    }
    
    /// イベントを一括追加
    pub fn push_events(&mut self, events: Vec<GameEvent>) {
        for event in events {
            self.events.push_back(event);
        }
    }
    
    /// 特定の型のイベントだけを取得
    pub fn poll_event_of_type<F>(&mut self, predicate: F) -> Option<GameEvent>
    where
        F: Fn(&GameEvent) -> bool,
    {
        if let Some(pos) = self.events.iter().position(predicate) {
            return Some(self.events.remove(pos).unwrap());
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_event_queue_basic_operations() {
        let mut queue = EventQueueResource::new();
        
        // 空のキューをテスト
        assert!(queue.is_empty());
        assert_eq!(queue.event_count(), 0);
        assert!(queue.poll_event().is_none());
        
        // イベントを追加
        queue.push_event(GameEvent::GameReset);
        assert!(!queue.is_empty());
        assert_eq!(queue.event_count(), 1);
        
        // イベントを覗き見
        if let Some(event) = queue.peek_event() {
            match event {
                GameEvent::GameReset => {},  // OK
                _ => panic!("間違ったイベントタイプ"),
            }
        } else {
            panic!("イベントがない");
        }
        
        // イベントがまだ存在する
        assert_eq!(queue.event_count(), 1);
        
        // イベントを取得
        if let Some(event) = queue.poll_event() {
            match event {
                GameEvent::GameReset => {},  // OK
                _ => panic!("間違ったイベントタイプ"),
            }
        } else {
            panic!("イベントがない");
        }
        
        // キューが空になった
        assert!(queue.is_empty());
    }
    
    #[test]
    fn test_event_queue_multiple_events() {
        let mut queue = EventQueueResource::new();
        
        // 複数のイベントを追加
        queue.push_event(GameEvent::CellReveal(1, 2));
        queue.push_event(GameEvent::FlagToggle(3, 4));
        queue.push_event(GameEvent::MouseMove(10.0, 20.0));
        
        assert_eq!(queue.event_count(), 3);
        
        // 順番にイベントを取得
        if let Some(GameEvent::CellReveal(row, col)) = queue.poll_event() {
            assert_eq!(row, 1);
            assert_eq!(col, 2);
        } else {
            panic!("CellRevealイベントがない");
        }
        
        if let Some(GameEvent::FlagToggle(row, col)) = queue.poll_event() {
            assert_eq!(row, 3);
            assert_eq!(col, 4);
        } else {
            panic!("FlagToggleイベントがない");
        }
        
        if let Some(GameEvent::MouseMove(x, y)) = queue.poll_event() {
            assert_eq!(x, 10.0);
            assert_eq!(y, 20.0);
        } else {
            panic!("MouseMoveイベントがない");
        }
        
        // すべてのイベントを取得した後はキューが空
        assert!(queue.is_empty());
    }
    
    #[test]
    fn test_event_type_filtering() {
        let mut queue = EventQueueResource::new();
        
        // いくつかのイベントを追加
        queue.push_event(GameEvent::CellReveal(1, 2));
        queue.push_event(GameEvent::MouseMove(10.0, 20.0));
        queue.push_event(GameEvent::FlagToggle(3, 4));
        queue.push_event(GameEvent::MouseMove(30.0, 40.0));
        
        assert_eq!(queue.event_count(), 4);
        
        // MouseMoveイベントだけを取得
        if let Some(GameEvent::MouseMove(x, y)) = queue.poll_event_of_type(|e| matches!(e, GameEvent::MouseMove(_, _))) {
            assert_eq!(x, 10.0);
            assert_eq!(y, 20.0);
        } else {
            panic!("MouseMoveイベントが見つかりませんでした");
        }
        
        // 残りのイベント数を確認
        assert_eq!(queue.event_count(), 3);
        
        // キューをクリア
        queue.clear();
        assert!(queue.is_empty());
    }
} 