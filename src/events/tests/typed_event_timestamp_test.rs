/**
 * タイムスタンプ付きイベントのテスト
 * 
 * イベントにタイムスタンプが正しく設定され、アクセスできることを確認します。
 */
use std::time::{SystemTime, UNIX_EPOCH};
use crate::events::typed_event::{TypedEvent, TimestampedEvent};
use crate::impl_typed_event;
use crate::impl_timestamped_event;
use crate::events::board_events::{CellRevealedEvent, MineExplodedEvent};
use crate::models::cell::CellValue;
use crate::models::coordinate::Coordinate;

#[test]
fn test_timestamp_on_mine_exploded_event() {
    // 地雷爆発イベントを作成
    let coord = Coordinate::new(5, 5);
    let event = MineExplodedEvent::new(coord, true);
    
    // タイムスタンプが現在時刻に近いことを確認
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64;
    
    // 少なくとも同じ秒内にあるはず
    assert!(event.timestamp() > 0);
    assert!(now >= event.timestamp());
    assert!(now - event.timestamp() < 1000); // 1秒以内の誤差
    
    // TypedEventトレイトを使用してアクセスできることを確認
    let timestamp = <MineExplodedEvent as TypedEvent>::timestamp(&event);
    assert_eq!(timestamp, event.timestamp());
    
    // TimestampedEventトレイトを使用してアクセスできることを確認
    let timestamped_timestamp = <MineExplodedEvent as TimestampedEvent>::timestamp(&event);
    assert_eq!(timestamped_timestamp, event.timestamp());
}

#[test]
fn test_timestamp_on_cell_revealed_event() {
    // セル開示イベントを作成
    let coord = Coordinate::new(3, 3);
    let event = CellRevealedEvent::new(coord, CellValue::Empty(0), false);
    
    // タイムスタンプが現在時刻に近いことを確認
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64;
    
    // 少なくとも同じ秒内にあるはず
    assert!(event.timestamp() > 0);
    assert!(now >= event.timestamp());
    assert!(now - event.timestamp() < 1000); // 1秒以内の誤差
    
    // TypedEventトレイトを使用してアクセスできることを確認
    let timestamp = <CellRevealedEvent as TypedEvent>::timestamp(&event);
    assert_eq!(timestamp, event.timestamp());
}

// カスタムのタイムスタンプ付きイベント型をテスト用に定義
#[derive(Debug, Clone)]
struct TestTimestampedEvent {
    name: String,
    _timestamp: u64,
}

impl TestTimestampedEvent {
    fn new(name: &str) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
            
        Self {
            name: name.to_string(),
            _timestamp: now,
        }
    }
}

impl_typed_event!(TestTimestampedEvent);
impl_timestamped_event!(TestTimestampedEvent);

#[test]
fn test_custom_timestamped_event() {
    // カスタムイベントを作成
    let event = TestTimestampedEvent::new("test-event");
    
    // タイムスタンプが現在時刻に近いことを確認
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64;
    
    assert!(event.timestamp() > 0);
    assert!(now >= event.timestamp());
    assert!(now - event.timestamp() < 1000); // 1秒以内の誤差
} 