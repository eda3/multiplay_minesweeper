/**
 * イベントシステムのテスト
 */

// テストモジュール
mod event_bus_tests;
mod event_handler_tests;
mod typed_event_tests;

// 一部のテストを再エクスポート
pub use event_bus_tests::*;
pub use event_handler_tests::*;
pub use typed_event_tests::*; 