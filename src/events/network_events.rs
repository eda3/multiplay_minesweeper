/**
 * ネットワーク関連のイベント
 * 
 * マルチプレイ時のネットワーク接続や通信に関連するイベント型を定義
 */
use std::fmt::Debug;
use serde::{Serialize, Deserialize};
use crate::events::event_trait::Event;
use crate::events::typed_event::TypedEvent;
use crate::events::EventData;
use crate::impl_event;
use crate::impl_typed_event;

/// ネットワーク接続イベント - サーバーへの接続を表す
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConnectEvent {
    /// 接続先サーバーURL
    pub server_url: String,
    /// 接続ID
    pub connection_id: Option<String>,
    /// ユーザーID
    pub user_id: Option<String>,
}

/// ネットワーク切断イベント - サーバーとの接続切断を表す
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkDisconnectEvent {
    /// 切断理由
    pub reason: Option<String>,
    /// 接続ID
    pub connection_id: Option<String>,
}

/// ネットワークエラーイベント - ネットワークエラーを表す
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkErrorEvent {
    /// エラーコード
    pub error_code: u32,
    /// エラーメッセージ
    pub message: String,
    /// リトライ可能かどうか
    pub is_retryable: bool,
}

/// データ受信イベント - サーバーからのデータ受信を表す
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataReceivedEvent {
    /// イベントタイプ
    pub event_type: String,
    /// ペイロード（JSON文字列）
    pub payload: String,
    /// 送信元ユーザーID
    pub sender_id: Option<String>,
    /// タイムスタンプ
    pub timestamp: u64,
}

/// データ送信イベント - サーバーへのデータ送信を表す
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataSentEvent {
    /// イベントタイプ
    pub event_type: String,
    /// ペイロード（JSON文字列）
    pub payload: String,
    /// 送信先ユーザーID（Noneの場合はブロードキャスト）
    pub recipient_id: Option<String>,
    /// メッセージID
    pub message_id: String,
}

/// セッション参加イベント - マルチプレイセッションへの参加を表す
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionJoinEvent {
    /// セッションID
    pub session_id: String,
    /// 参加者の名前
    pub player_name: String,
    /// 参加者のID
    pub player_id: String,
    /// ホストかどうか
    pub is_host: bool,
}

/// セッション退出イベント - マルチプレイセッションからの退出を表す
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionLeaveEvent {
    /// セッションID
    pub session_id: String,
    /// 退出者のID
    pub player_id: String,
    /// 退出理由
    pub reason: Option<String>,
}

/// プレイヤー参加イベント - 新しいプレイヤーの参加を表す
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerJoinedEvent {
    /// プレイヤーのID
    pub player_id: String,
    /// プレイヤーの名前
    pub player_name: String,
    /// 参加時刻
    pub joined_at: u64,
}

/// プレイヤー退出イベント - プレイヤーの退出を表す
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerLeftEvent {
    /// プレイヤーのID
    pub player_id: String,
    /// 退出時刻
    pub left_at: u64,
}

/// ラグ測定イベント - ネットワークラグの測定結果を表す
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LagMeasurementEvent {
    /// 往復時間（ミリ秒）
    pub round_trip_time: u32,
    /// サーバー時間との差（ミリ秒）
    pub time_diff: i32,
}

// PlayerScoreUpdateEvent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerScoreUpdateEvent {
    /// プレイヤーのID
    pub player_id: String,
    /// 新しいスコア
    pub new_score: u32,
    /// タイムスタンプ
    pub timestamp: u64,
}

// ChatMessageEvent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessageEvent {
    /// プレイヤーのID
    pub player_id: String,
    /// メッセージ
    pub message: String,
    /// タイムスタンプ
    pub timestamp: u64,
}

// RoomStateEvent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoomStateEvent {
    /// ルームID
    pub room_id: String,
    /// プレイヤー情報
    pub players: Vec<PlayerInfo>,
    /// ゲーム状態
    pub game_state: String,
    /// タイムスタンプ
    pub timestamp: u64,
}

// ConnectionStateEvent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionStateEvent {
    /// 接続状態
    pub state: String,
    /// エラーメッセージ
    pub error_message: Option<String>,
    /// タイムスタンプ
    pub timestamp: u64,
}

// プレイヤー情報構造体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerInfo {
    /// プレイヤーID
    pub id: String,
    /// プレイヤー名
    pub name: String,
    /// プレイヤーカラー
    pub color: String,
    /// プレイヤースコア
    pub score: u32,
    /// アクティブかどうか
    pub is_active: bool,
}

// Event実装
impl_event!(NetworkConnectEvent, "NetworkConnect");
impl_event!(NetworkDisconnectEvent, "NetworkDisconnect");
impl_event!(NetworkErrorEvent, "NetworkError");
impl_event!(DataReceivedEvent, "DataReceived");
impl_event!(DataSentEvent, "DataSent");
impl_event!(SessionJoinEvent, "SessionJoin");
impl_event!(SessionLeaveEvent, "SessionLeave");
impl_event!(PlayerJoinedEvent, "PlayerJoined");
impl_event!(PlayerLeftEvent, "PlayerLeft");
impl_event!(LagMeasurementEvent, "LagMeasurement");
impl_event!(PlayerScoreUpdateEvent, "PlayerScoreUpdate");
impl_event!(ChatMessageEvent, "ChatMessage");
impl_event!(RoomStateEvent, "RoomState");
impl_event!(ConnectionStateEvent, "ConnectionState");

// TypedEvent実装
impl_typed_event!(NetworkConnectEvent);
impl_typed_event!(NetworkDisconnectEvent);
impl_typed_event!(NetworkErrorEvent);
impl_typed_event!(DataReceivedEvent);
impl_typed_event!(DataSentEvent);
impl_typed_event!(SessionJoinEvent);
impl_typed_event!(SessionLeaveEvent);
impl_typed_event!(PlayerJoinedEvent);
impl_typed_event!(PlayerLeftEvent);
impl_typed_event!(LagMeasurementEvent);
impl_typed_event!(PlayerScoreUpdateEvent);
impl_typed_event!(ChatMessageEvent);
impl_typed_event!(RoomStateEvent);
impl_typed_event!(ConnectionStateEvent); 