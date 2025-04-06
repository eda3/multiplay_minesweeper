/**
 * ボード関連のイベント
 * 
 * ボードの状態やセルの操作に関連するイベント型を定義
 */
use std::fmt::Debug;
use serde::{Serialize, Deserialize};
use crate::events::event_trait::Event;
use crate::events::typed_event::TypedEvent;
use crate::events::EventData;
use crate::impl_event;
use crate::impl_typed_event;
use crate::impl_typed_event_with_conversion;
use crate::models::cell::{CellState, CellValue};
use crate::models::coordinate::Coordinate;

/// セル状態変更イベント - セルの状態が変更されたことを表す
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CellStateChangeEvent {
    /// セルの座標
    pub coord: Coordinate,
    /// 新しい状態
    pub new_state: CellState,
    /// 前の状態
    pub old_state: CellState,
    /// セルの値
    pub value: CellValue,
}

/// セル一括状態変更イベント - 複数のセルの状態が同時に変更されたことを表す
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BulkCellStateChangeEvent {
    /// 変更されたセルの座標と新しい状態のリスト
    pub changes: Vec<(Coordinate, CellState, CellValue)>,
}

/// フラグ設置イベント - セルにフラグが設置されたことを表す
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlagPlacedEvent {
    /// セルの座標
    pub coord: Coordinate,
    /// フラグが設置されたかどうか（falseの場合は除去を表す）
    pub is_placed: bool,
    /// 残りのフラグ数
    pub remaining_flags: u32,
}

/// セル開示イベント - セルが開示されたことを表す
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CellRevealedEvent {
    /// セルの座標
    pub coord: Coordinate,
    /// セルの値
    pub value: CellValue,
    /// チェーン反応で開いたかどうか
    pub is_chain: bool,
}

/// 複数セル開示イベント - 複数のセルが同時に開示されたことを表す
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultipleCellsRevealedEvent {
    /// 開示されたセルの座標と値のリスト
    pub revealed_cells: Vec<(Coordinate, CellValue)>,
    /// チェーン反応で開いたかどうか
    pub is_chain: bool,
}

/// 地雷爆発イベント - 地雷が爆発したことを表す
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MineExplodedEvent {
    /// 爆発した地雷の座標
    pub coord: Coordinate,
}

/// ボード初期化イベント - ボードが初期化されたことを表す
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoardInitializedEvent {
    /// ボードの幅
    pub width: u32,
    /// ボードの高さ
    pub height: u32,
    /// 地雷の数
    pub mine_count: u32,
    /// 初回クリックされた座標（地雷が配置されていない）
    pub first_click: Option<Coordinate>,
}

/// ゲーム進行状況イベント - ゲームの進行状況を表す
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameProgressEvent {
    /// 開示されたセルの数
    pub revealed_count: u32,
    /// 残りのセルの数（地雷以外）
    pub remaining_non_mine_cells: u32,
    /// 完了率（0.0〜1.0）
    pub completion_rate: f32,
}

// Event実装
impl_event!(CellStateChangeEvent, "CellStateChange");
impl_event!(BulkCellStateChangeEvent, "BulkCellStateChange");
impl_event!(FlagPlacedEvent, "FlagPlaced");
impl_event!(CellRevealedEvent, "CellRevealed");
impl_event!(MultipleCellsRevealedEvent, "MultipleCellsRevealed");
impl_event!(MineExplodedEvent, "MineExploded");
impl_event!(BoardInitializedEvent, "BoardInitialized");
impl_event!(GameProgressEvent, "GameProgress");

// TypedEvent実装（個別実装）
impl_typed_event_with_conversion!(CellRevealedEvent, |event: &CellRevealedEvent| Some(EventData::CellRevealed(event.clone())));
impl_typed_event_with_conversion!(MineExplodedEvent, |event: &MineExplodedEvent| Some(EventData::MineExploded(event.clone())));
impl_typed_event_with_conversion!(BoardInitializedEvent, |event: &BoardInitializedEvent| Some(EventData::BoardInitialized(event.clone())));
impl_typed_event_with_conversion!(MultipleCellsRevealedEvent, |event: &MultipleCellsRevealedEvent| Some(EventData::MultipleCellsRevealed(event.clone())));
impl_typed_event_with_conversion!(GameProgressEvent, |event: &GameProgressEvent| Some(EventData::GameProgress(event.clone())));

// その他のイベント用にデフォルト実装
impl_typed_event!(CellStateChangeEvent);
impl_typed_event!(BulkCellStateChangeEvent);
impl_typed_event!(FlagPlacedEvent); 