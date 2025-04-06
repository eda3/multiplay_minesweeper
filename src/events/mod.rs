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
pub mod typed_event;     // 新しい型安全なイベントトレイト
pub mod typed_handler;   // 新しい型安全なハンドラ
pub mod typed_event_bus; // 新しい型安全なイベントバス

#[cfg(test)]
mod tests;

// 主要なコンポーネントの再エクスポート
pub use event_trait::Event;
pub use event_handler::{EventHandler, EventHandlerFn};
pub use event_bus::EventBus;

// 型安全なコンポーネントの再エクスポート
pub use typed_event::{TypedEvent, HandlerId, TypedEventId};
pub use typed_handler::{TypedEventHandler, TypedHandlerCollection};
pub use typed_event_bus::TypedEventBus;

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
    KeyboardInput(input_events::KeyboardEvent),
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
    PlayerScoreUpdate(network_events::PlayerScoreUpdateEvent),
    ChatMessage(network_events::ChatMessageEvent),
    RoomState(network_events::RoomStateEvent),
    ConnectionState(network_events::ConnectionStateEvent),
    
    // システム関連イベント
    ResourceLoad(input_events::ResourceLoadEvent),
    AppState(input_events::AppStateEvent),
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
            Self::KeyboardInput(_) => "KeyboardInput",
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
            Self::PlayerScoreUpdate(_) => "PlayerScoreUpdate",
            Self::ChatMessage(_) => "ChatMessage",
            Self::RoomState(_) => "RoomState",
            Self::ConnectionState(_) => "ConnectionState",
            
            // システム関連イベント
            Self::ResourceLoad(_) => "ResourceLoad",
            Self::AppState(_) => "AppState",
        }
    }
    
    // タイムスタンプを取得する便利メソッドを追加
    pub fn timestamp(&self) -> u64 {
        match self {
            // ゲームイベント
            Self::GameStart(e) => e.timestamp(),
            Self::GameEnd(e) => e.timestamp(),
            Self::GameStateChange(e) => e.timestamp(),
            Self::Timer(e) => e.timestamp(),
            Self::DifficultyChange(e) => e.timestamp(),
            Self::ScoreUpdate(e) => e.timestamp(),
            
            // ボードイベント
            Self::CellStateChange(e) => e.timestamp(),
            Self::BulkCellStateChange(e) => e.timestamp(),
            Self::FlagPlaced(e) => e.timestamp(),
            Self::CellRevealed(e) => e.timestamp(),
            Self::MultipleCellsRevealed(e) => e.timestamp(),
            Self::MineExploded(e) => e.timestamp(),
            Self::BoardInitialized(e) => e.timestamp(),
            Self::GameProgress(e) => e.timestamp(),
            
            // 入力イベント
            Self::MouseMove(e) => e.timestamp(),
            Self::MouseClick(e) => e.timestamp(),
            Self::KeyboardInput(e) => e.timestamp(),
            Self::UIClick(e) => e.timestamp(),
            Self::Hotkey(e) => e.timestamp(),
            
            // ネットワークイベント
            Self::NetworkConnect(e) => e.timestamp(),
            Self::NetworkDisconnect(e) => e.timestamp(),
            Self::NetworkError(e) => e.timestamp(),
            Self::DataReceived(e) => e.timestamp(),
            Self::DataSent(e) => e.timestamp(),
            Self::SessionJoin(e) => e.timestamp(),
            Self::SessionLeave(e) => e.timestamp(),
            Self::PlayerJoined(e) => e.timestamp(),
            Self::PlayerLeft(e) => e.timestamp(),
            Self::LagMeasurement(e) => e.timestamp(),
            Self::PlayerScoreUpdate(e) => e.timestamp(),
            Self::ChatMessage(e) => e.timestamp(),
            Self::RoomState(e) => e.timestamp(),
            Self::ConnectionState(e) => e.timestamp(),
            
            // システム関連イベント
            Self::ResourceLoad(e) => e.timestamp(),
            Self::AppState(e) => e.timestamp(),
        }
    }
}

/// イベントシステムの初期化関数
pub fn init() -> EventBus {
    EventBus::new()
}

/// 型安全なイベントシステムの初期化関数
pub fn init_typed() -> TypedEventBus {
    TypedEventBus::new()
} 