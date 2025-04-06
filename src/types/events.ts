/**
 * マルチプレイマインスイーパーのイベント型定義
 * 
 * Rustとの型の一貫性を保ちつつ、TypeScriptでの型安全性を確保するためのイベント型定義
 */

import { Coordinate } from './coordinate';

/**
 * イベントの優先度
 */
export enum EventPriority {
    Low = 0,
    Normal = 1,
    High = 2,
}

/**
 * 基本イベントインターフェース
 * すべてのイベントはこれを実装する
 */
export interface BaseEvent {
    type: string;
    timestamp: number;
}

// ----- 入力関連イベント -----

/**
 * マウスクリックイベント
 */
export interface MouseClickEvent extends BaseEvent {
    type: 'MouseClick';
    x: number;
    y: number;
    is_right_click: boolean;
}

/**
 * キーボード入力イベント
 */
export interface KeyboardInputEvent extends BaseEvent {
    type: 'KeyboardInput';
    key_code: string;
    is_pressed: boolean;
}

// ----- ボード関連イベント -----

/**
 * セル公開イベント
 */
export interface CellRevealedEvent extends BaseEvent {
    type: 'CellRevealed';
    coord: Coordinate;
    is_chain: boolean;
}

/**
 * 複数セル公開イベント（チェイン反応用）
 */
export interface MultipleCellsRevealedEvent extends BaseEvent {
    type: 'MultipleCellsRevealed';
    coords: Coordinate[];
}

/**
 * 一括セル状態変更イベント
 */
export interface BulkCellStateChangeEvent extends BaseEvent {
    type: 'BulkCellStateChange';
    revealed_coords: Coordinate[];
    flagged_coords: Coordinate[];
}

/**
 * 地雷爆発イベント
 */
export interface MineExplodedEvent extends BaseEvent {
    type: 'MineExploded';
    coord: Coordinate;
    is_game_over: boolean;
}

/**
 * ゲーム進行イベント
 */
export interface GameProgressEvent extends BaseEvent {
    type: 'GameProgress';
    revealed_count: number;
    flagged_count: number;
    total_cells: number;
    remaining_mines: number;
    elapsed_time: number;
}

// ----- ゲーム状態関連イベント -----

/**
 * ゲーム状態
 */
export enum GameState {
    Waiting = 'Waiting',
    InProgress = 'InProgress',
    Victory = 'Victory',
    GameOver = 'GameOver',
    Paused = 'Paused',
}

/**
 * ゲーム状態変更イベント
 */
export interface GameStateChangeEvent extends BaseEvent {
    type: 'GameStateChange';
    previous_state: GameState;
    new_state: GameState;
}

/**
 * ゲーム終了イベント
 */
export interface GameEndEvent extends BaseEvent {
    type: 'GameEnd';
    is_win: boolean;
    play_time: number;
    revealed_cells: number;
    flagged_cells: number;
    score: number;
}

// ----- マルチプレイ関連イベント -----

/**
 * プレイヤー状態
 */
export interface PlayerState {
    id: string;
    name: string;
    color: string;
    score: number;
    is_active: boolean;
}

/**
 * プレイヤー参加イベント
 */
export interface PlayerJoinedEvent extends BaseEvent {
    type: 'PlayerJoined';
    player: PlayerState;
}

/**
 * プレイヤー退出イベント
 */
export interface PlayerLeftEvent extends BaseEvent {
    type: 'PlayerLeft';
    player_id: string;
}

/**
 * プレイヤースコア更新イベント
 */
export interface PlayerScoreUpdateEvent extends BaseEvent {
    type: 'PlayerScoreUpdate';
    player_id: string;
    new_score: number;
}

/**
 * チャットメッセージイベント
 */
export interface ChatMessageEvent extends BaseEvent {
    type: 'ChatMessage';
    player_id: string;
    message: string;
}

/**
 * ルーム状態イベント
 */
export interface RoomStateEvent extends BaseEvent {
    type: 'RoomState';
    room_id: string;
    players: PlayerState[];
    game_state: GameState;
}

// ----- ネットワーク関連イベント -----

/**
 * 接続状態
 */
export enum ConnectionState {
    Connecting = 'Connecting',
    Connected = 'Connected',
    Disconnected = 'Disconnected',
    Error = 'Error',
}

/**
 * 接続状態イベント
 */
export interface ConnectionStateEvent extends BaseEvent {
    type: 'ConnectionState';
    state: ConnectionState;
    error_message?: string;
}

// ----- システム関連イベント -----

/**
 * リソース読み込みイベント
 */
export interface ResourceLoadEvent extends BaseEvent {
    type: 'ResourceLoad';
    resource_type: string;
    resource_id: string;
    is_success: boolean;
    error_message?: string;
}

/**
 * アプリケーション状態イベント
 */
export interface AppStateEvent extends BaseEvent {
    type: 'AppState';
    is_initialized: boolean;
    current_view: string;
}

// ----- イベントの型マップ（型安全なイベントハンドリングのため）-----

export type EventMap = {
    'MouseClick': MouseClickEvent;
    'KeyboardInput': KeyboardInputEvent;
    'CellRevealed': CellRevealedEvent;
    'MultipleCellsRevealed': MultipleCellsRevealedEvent;
    'BulkCellStateChange': BulkCellStateChangeEvent;
    'MineExploded': MineExplodedEvent;
    'GameProgress': GameProgressEvent;
    'GameStateChange': GameStateChangeEvent;
    'GameEnd': GameEndEvent;
    'PlayerJoined': PlayerJoinedEvent;
    'PlayerLeft': PlayerLeftEvent;
    'PlayerScoreUpdate': PlayerScoreUpdateEvent;
    'ChatMessage': ChatMessageEvent;
    'RoomState': RoomStateEvent;
    'ConnectionState': ConnectionStateEvent;
    'ResourceLoad': ResourceLoadEvent;
    'AppState': AppStateEvent;
}

/**
 * 型安全なイベントバスクラスの型定義
 */
export interface TypedEventBus {
    subscribe<K extends keyof EventMap>(
        eventType: K,
        handler: (event: EventMap[K]) => void,
        handlerId?: string
    ): string;

    unsubscribe(handlerId: string): boolean;

    publish<K extends keyof EventMap>(
        event: EventMap[K],
        priority?: EventPriority
    ): void;

    publishWithPriority<K extends keyof EventMap>(
        event: EventMap[K],
        priority: EventPriority
    ): void;
} 