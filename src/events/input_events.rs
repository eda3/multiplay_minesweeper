/**
 * 入力関連のイベント
 * 
 * マウスやキーボードなどの入力に関連するイベント型を定義
 */
use std::fmt::Debug;
use serde::{Serialize, Deserialize};
use crate::events::event_trait::Event;
use crate::events::typed_event::TypedEvent;
use crate::events::EventData;
use crate::impl_event;
use crate::impl_typed_event;

/// マウス移動イベント - マウスの移動を表す
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MouseMoveEvent {
    /// X座標
    pub x: f64,
    /// Y座標
    pub y: f64,
    /// 前回からのX方向の移動量
    pub delta_x: f64,
    /// 前回からのY方向の移動量
    pub delta_y: f64,
}

/// マウスクリックイベント - マウスのクリックを表す
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MouseClickEvent {
    /// X座標
    pub x: f64,
    /// Y座標
    pub y: f64,
    /// 右クリックかどうか
    pub is_right_click: bool,
    /// 中クリックかどうか
    pub is_middle_click: bool,
    /// ダブルクリックかどうか
    pub is_double_click: bool,
}

/// キーボード入力イベント - キーボードの入力を表す
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyboardEvent {
    /// キーコード
    pub key_code: u32,
    /// キー文字
    pub key: String,
    /// キーが押されたかどうか（falseの場合は離されたことを表す）
    pub is_down: bool,
    /// Ctrlキーが押されているかどうか
    pub ctrl_key: bool,
    /// Shiftキーが押されているかどうか
    pub shift_key: bool,
    /// Altキーが押されているかどうか
    pub alt_key: bool,
}

/// UIクリックイベント - UI要素のクリックを表す
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UIClickEvent {
    /// 要素のID
    pub element_id: String,
    /// アクション
    pub action: Option<String>,
    /// X座標
    pub x: f64,
    /// Y座標
    pub y: f64,
}

/// ホットキーイベント - ショートカットキーの入力を表す
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotkeyEvent {
    /// アクション名
    pub action: String,
}

/// リソース読み込みイベント - リソースの読み込みを表す
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLoadEvent {
    /// リソースタイプ
    pub resource_type: String,
    /// リソースID
    pub resource_id: String,
    /// 読み込み成功したかどうか
    pub is_success: bool,
    /// エラーメッセージ
    pub error_message: Option<String>,
    /// タイムスタンプ
    pub timestamp: u64,
}

/// アプリケーション状態イベント - アプリケーションの状態を表す
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppStateEvent {
    /// 初期化完了したかどうか
    pub is_initialized: bool,
    /// 現在のビュー
    pub current_view: String,
    /// タイムスタンプ
    pub timestamp: u64,
}

// Event実装
impl_event!(MouseMoveEvent, "MouseMove");
impl_event!(MouseClickEvent, "MouseClick");
impl_event!(KeyboardEvent, "KeyboardInput");
impl_event!(UIClickEvent, "UIClick");
impl_event!(HotkeyEvent, "Hotkey");
impl_event!(ResourceLoadEvent, "ResourceLoad");
impl_event!(AppStateEvent, "AppState");

// TypedEvent実装
impl_typed_event!(MouseMoveEvent);
impl_typed_event!(MouseClickEvent);
impl_typed_event!(KeyboardEvent);
impl_typed_event!(UIClickEvent);
impl_typed_event!(HotkeyEvent);
impl_typed_event!(ResourceLoadEvent);
impl_typed_event!(AppStateEvent); 