/**
 * イベントシステムのテスト
 */

// テストモジュール
mod typed_event_tests;
mod typed_event_extended_tests;
mod typed_event_timestamp_test;

// テストを再エクスポート
pub use typed_event_tests::*;
pub use typed_event_extended_tests::*; 