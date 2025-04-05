/**
 * イベントシステムのエントリポイント
 * 
 * 型安全なイベントシステムを提供し、各種イベントの定義とイベントバスを管理
 */

// エクスポートするサブモジュール
pub mod event_trait;
pub mod event_handler;
pub mod event_bus;
pub mod game_events;
pub mod board_events;
pub mod input_events;
pub mod network_events;

#[cfg(test)]
mod tests;

// マクロの定義
#[macro_export]
macro_rules! impl_event {
    ($event_type:ty, $event_name:expr) => {
        impl Event for $event_type {
            fn name(&self) -> &'static str {
                $event_name
            }
        }
    };
}

// 主要なコンポーネントの再エクスポート
pub use event_trait::Event;
pub use event_handler::{EventHandler, EventHandlerFn};
pub use event_bus::EventBus;

// 各種イベント型の再エクスポート
pub use game_events::*;
pub use board_events::*;
pub use input_events::*;
pub use network_events::*;

// イベントデータを包含するEnum型
#[derive(Debug, Clone)]
pub enum EventData {
    // ゲームイベント
    GameStart(game_events::GameStartEvent),
    GameEnd(game_events::GameEndEvent),
    GameStateChange(game_events::GameStateChangeEvent),
    Timer(game_events::TimerEvent),
    DifficultyChange(game_events::DifficultyChangeEvent),
    ScoreUpdate(game_events::ScoreUpdateEvent),
    
    // ボードイベント
    CellStateChange(board_events::CellStateChangeEvent),
    BulkCellStateChange(board_events::BulkCellStateChangeEvent),
    FlagPlaced(board_events::FlagPlacedEvent),
    CellRevealed(board_events::CellRevealedEvent),
    MultipleCellsRevealed(board_events::MultipleCellsRevealedEvent),
    MineExploded(board_events::MineExplodedEvent),
    BoardInitialized(board_events::BoardInitializedEvent),
    GameProgress(board_events::GameProgressEvent),
    
    // 入力イベント
    MouseMove(input_events::MouseMoveEvent),
    MouseClick(input_events::MouseClickEvent),
    Keyboard(input_events::KeyboardEvent),
    UIClick(input_events::UIClickEvent),
    Hotkey(input_events::HotkeyEvent),
    
    // ネットワークイベント
    NetworkConnect(network_events::NetworkConnectEvent),
    NetworkDisconnect(network_events::NetworkDisconnectEvent),
    NetworkError(network_events::NetworkErrorEvent),
    DataReceived(network_events::DataReceivedEvent),
    DataSent(network_events::DataSentEvent),
    SessionJoin(network_events::SessionJoinEvent),
    SessionLeave(network_events::SessionLeaveEvent),
    PlayerJoined(network_events::PlayerJoinedEvent),
    PlayerLeft(network_events::PlayerLeftEvent),
    LagMeasurement(network_events::LagMeasurementEvent),
}

impl EventData {
    /// イベントの名前を取得
    pub fn name(&self) -> &'static str {
        match self {
            // ゲームイベント
            Self::GameStart(_) => "GameStart",
            Self::GameEnd(_) => "GameEnd",
            Self::GameStateChange(_) => "GameStateChange",
            Self::Timer(_) => "Timer",
            Self::DifficultyChange(_) => "DifficultyChange",
            Self::ScoreUpdate(_) => "ScoreUpdate",
            
            // ボードイベント
            Self::CellStateChange(_) => "CellStateChange",
            Self::BulkCellStateChange(_) => "BulkCellStateChange",
            Self::FlagPlaced(_) => "FlagPlaced",
            Self::CellRevealed(_) => "CellRevealed",
            Self::MultipleCellsRevealed(_) => "MultipleCellsRevealed",
            Self::MineExploded(_) => "MineExploded",
            Self::BoardInitialized(_) => "BoardInitialized",
            Self::GameProgress(_) => "GameProgress",
            
            // 入力イベント
            Self::MouseMove(_) => "MouseMove",
            Self::MouseClick(_) => "MouseClick",
            Self::Keyboard(_) => "Keyboard",
            Self::UIClick(_) => "UIClick",
            Self::Hotkey(_) => "Hotkey",
            
            // ネットワークイベント
            Self::NetworkConnect(_) => "NetworkConnect",
            Self::NetworkDisconnect(_) => "NetworkDisconnect",
            Self::NetworkError(_) => "NetworkError",
            Self::DataReceived(_) => "DataReceived",
            Self::DataSent(_) => "DataSent",
            Self::SessionJoin(_) => "SessionJoin",
            Self::SessionLeave(_) => "SessionLeave",
            Self::PlayerJoined(_) => "PlayerJoined",
            Self::PlayerLeft(_) => "PlayerLeft",
            Self::LagMeasurement(_) => "LagMeasurement",
        }
    }
}

/// イベントシステムの初期化関数
pub fn init() -> EventBus {
    EventBus::new()
} 