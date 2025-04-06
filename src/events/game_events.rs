/**
 * ゲーム状態関連のイベント
 * 
 * ゲームの開始、終了、状態変更などに関連するイベント型を定義
 */
use std::fmt::Debug;
use serde::{Serialize, Deserialize};
use crate::events::event_trait::Event;
use crate::events::typed_event::TypedEvent;
use crate::events::EventData;
use crate::impl_event;
use crate::impl_typed_event;
use crate::models::difficulty::Difficulty;

/// ゲーム開始イベント - ゲームの開始を表す
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameStartEvent {
    /// 難易度
    pub difficulty: Difficulty,
    /// カスタムモードの場合の幅
    pub custom_width: Option<u32>,
    /// カスタムモードの場合の高さ
    pub custom_height: Option<u32>,
    /// カスタムモードの場合の地雷数
    pub custom_mines: Option<u32>,
    /// マルチプレイヤーかどうか
    pub is_multiplayer: bool,
    /// セッションID（マルチプレイヤー時のみ）
    pub session_id: Option<String>,
}

/// ゲーム終了イベント - ゲームの終了を表す
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameEndEvent {
    /// 勝利したかどうか
    pub is_win: bool,
    /// プレイ時間（秒）
    pub play_time: u32,
    /// 開いたセル数
    pub revealed_cells: u32,
    /// フラグを立てたセル数
    pub flagged_cells: u32,
    /// スコア
    pub score: u32,
}

/// ゲーム状態変更イベント - ゲームの状態変更を表す
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameStateChangeEvent {
    /// 新しい状態
    pub new_state: GameState,
    /// 前の状態
    pub old_state: GameState,
}

/// ゲームの状態
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GameState {
    /// メニュー画面
    Menu,
    /// ゲーム初期化中
    Initializing,
    /// ゲームプレイ中
    Playing,
    /// 一時停止中
    Paused,
    /// ゲーム終了
    GameOver,
    /// 勝利
    Victory,
    /// 結果画面
    Results,
}

/// タイマーイベント - ゲーム内時間の更新を表す
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimerEvent {
    /// 現在の経過時間（秒）
    pub time: u32,
}

/// 難易度変更イベント - 難易度の変更を表す
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DifficultyChangeEvent {
    /// 新しい難易度
    pub new_difficulty: Difficulty,
    /// カスタムモードの場合の幅
    pub custom_width: Option<u32>,
    /// カスタムモードの場合の高さ
    pub custom_height: Option<u32>,
    /// カスタムモードの場合の地雷数
    pub custom_mines: Option<u32>,
}

/// スコア更新イベント - スコアの更新を表す
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoreUpdateEvent {
    /// 新しいスコア
    pub new_score: u32,
    /// スコア変化量
    pub delta: i32,
}

// Event実装
impl_event!(GameStartEvent, "GameStart");
impl_event!(GameEndEvent, "GameEnd");
impl_event!(GameStateChangeEvent, "GameStateChange");
impl_event!(TimerEvent, "Timer");
impl_event!(DifficultyChangeEvent, "DifficultyChange");
impl_event!(ScoreUpdateEvent, "ScoreUpdate");

// TypedEvent実装
impl_typed_event!(GameStartEvent);
impl_typed_event!(GameEndEvent);
impl_typed_event!(GameStateChangeEvent);
impl_typed_event!(TimerEvent);
impl_typed_event!(DifficultyChangeEvent);
impl_typed_event!(ScoreUpdateEvent); 