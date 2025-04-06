use std::any::Any;
use std::collections::VecDeque;

use crate::events::EventData;

/// イベント履歴を管理するクラス
pub struct EventHistory {
    /// イベント履歴の最大サイズ
    max_size: usize,
    /// イベントデータのキュー
    events: VecDeque<EventData>,
}

impl EventHistory {
    /// 新しいイベント履歴を作成
    pub fn new(max_size: usize) -> Self {
        Self {
            max_size,
            events: VecDeque::with_capacity(max_size),
        }
    }
    
    /// イベントを追加
    pub fn add_event(&mut self, event: EventData) {
        // 最大サイズに達したら古いイベントを削除
        if self.events.len() >= self.max_size {
            self.events.pop_front();
        }
        
        self.events.push_back(event);
    }
    
    /// イベント履歴を取得
    pub fn get_events(&self) -> &VecDeque<EventData> {
        &self.events
    }
    
    /// イベント履歴をクリア
    pub fn clear(&mut self) {
        self.events.clear();
    }
    
    /// イベント履歴のサイズを取得
    pub fn size(&self) -> usize {
        self.events.len()
    }
    
    /// 最後のイベントを取得
    pub fn last_event(&self) -> Option<&EventData> {
        self.events.back()
    }
} 