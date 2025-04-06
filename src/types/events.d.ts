/**
 * マルチプレイマインスイーパーのイベント型定義
 * @packageDocumentation
 */

import { Coordinate } from './coordinate';

/**
 * イベントの優先度
 */
export enum EventPriority {
    Low = 0,
    Normal = 1,
    High = 2,
    Background = 30
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

/**
 * UIクリックイベント
 */
export interface UIClickEvent extends BaseEvent {
    type: 'UIClick';
    element_id: string;
    x: number;
    y: number;
}

/**
 * ホットキーイベント
 */
export interface HotkeyEvent extends BaseEvent {
    type: 'Hotkey';
    key_combination: string;
    action: string;
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
 * セル状態変更イベント
 */
export interface CellStateChangeEvent extends BaseEvent {
    type: 'CellStateChange';
    coord: Coordinate;
    new_state: string;
    previous_state: string;
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
 * フラグ設置イベント
 */
export interface FlagPlacedEvent extends BaseEvent {
    type: 'FlagPlaced';
    coord: Coordinate;
    is_flagged: boolean;
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
 * ボード初期化イベント
 */
export interface BoardInitializedEvent extends BaseEvent {
    type: 'BoardInitialized';
    width: number;
    height: number;
    mine_count: number;
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
 * ゲーム開始イベント
 */
export interface GameStartEvent extends BaseEvent {
    type: 'GameStart';
    difficulty: string;
    width: number;
    height: number;
    mine_count: number;
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

/**
 * 難易度変更イベント
 */
export interface DifficultyChangeEvent extends BaseEvent {
    type: 'DifficultyChange';
    difficulty: string;
    width: number;
    height: number;
    mine_count: number;
}

/**
 * タイマーイベント
 */
export interface TimerEvent extends BaseEvent {
    type: 'Timer';
    elapsed_time: number;
    is_running: boolean;
}

/**
 * スコア更新イベント
 */
export interface ScoreUpdateEvent extends BaseEvent {
    type: 'ScoreUpdate';
    score: number;
    time_bonus: number;
    reveal_bonus: number;
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
 * ネットワーク接続イベント
 */
export interface NetworkConnectEvent extends BaseEvent {
    type: 'NetworkConnect';
    url: string;
    session_id?: string;
}

/**
 * ネットワーク切断イベント
 */
export interface NetworkDisconnectEvent extends BaseEvent {
    type: 'NetworkDisconnect';
    reason: string;
    was_clean: boolean;
}

/**
 * ネットワークエラーイベント
 */
export interface NetworkErrorEvent extends BaseEvent {
    type: 'NetworkError';
    error_code: number;
    error_message: string;
}

/**
 * データ受信イベント
 */
export interface DataReceivedEvent extends BaseEvent {
    type: 'DataReceived';
    data_type: string;
    size: number;
}

/**
 * データ送信イベント
 */
export interface DataSentEvent extends BaseEvent {
    type: 'DataSent';
    data_type: string;
    size: number;
    is_reliable: boolean;
}

/**
 * セッション参加イベント
 */
export interface SessionJoinEvent extends BaseEvent {
    type: 'SessionJoin';
    session_id: string;
    player_count: number;
}

/**
 * セッション退出イベント
 */
export interface SessionLeaveEvent extends BaseEvent {
    type: 'SessionLeave';
    session_id: string;
    reason: string;
}

/**
 * 遅延測定イベント
 */
export interface LagMeasurementEvent extends BaseEvent {
    type: 'LagMeasurement';
    ping_ms: number;
    jitter_ms: number;
    packet_loss: number;
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
    'UIClick': UIClickEvent;
    'Hotkey': HotkeyEvent;
    'CellRevealed': CellRevealedEvent;
    'MultipleCellsRevealed': MultipleCellsRevealedEvent;
    'CellStateChange': CellStateChangeEvent;
    'BulkCellStateChange': BulkCellStateChangeEvent;
    'FlagPlaced': FlagPlacedEvent;
    'MineExploded': MineExplodedEvent;
    'BoardInitialized': BoardInitializedEvent;
    'GameProgress': GameProgressEvent;
    'GameStart': GameStartEvent;
    'GameStateChange': GameStateChangeEvent;
    'GameEnd': GameEndEvent;
    'DifficultyChange': DifficultyChangeEvent;
    'Timer': TimerEvent;
    'ScoreUpdate': ScoreUpdateEvent;
    'PlayerJoined': PlayerJoinedEvent;
    'PlayerLeft': PlayerLeftEvent;
    'PlayerScoreUpdate': PlayerScoreUpdateEvent;
    'ChatMessage': ChatMessageEvent;
    'RoomState': RoomStateEvent;
    'NetworkConnect': NetworkConnectEvent;
    'NetworkDisconnect': NetworkDisconnectEvent;
    'NetworkError': NetworkErrorEvent;
    'DataReceived': DataReceivedEvent;
    'DataSent': DataSentEvent;
    'SessionJoin': SessionJoinEvent;
    'SessionLeave': SessionLeaveEvent;
    'LagMeasurement': LagMeasurementEvent;
    'ConnectionState': ConnectionStateEvent;
    'ResourceLoad': ResourceLoadEvent;
    'AppState': AppStateEvent;
};

/**
 * 型安全なイベントバスクラスの型定義
 */
export interface TypedEventBus {
    /**
     * イベントを購読する
     * @param eventType イベントの種類
     * @param handler イベントハンドラー関数
     * @param handlerId 任意のハンドラーID
     * @returns ハンドラーID（unsubscribeで使用）
     */
    subscribe<K extends keyof EventMap>(
        eventType: K,
        handler: (event: EventMap[K]) => void,
        handlerId?: string
    ): string;

    /**
     * イベント購読を解除する
     * @param handlerId 購読解除するハンドラーID
     * @returns 解除が成功したかどうか
     */
    unsubscribe(handlerId: string): boolean;

    /**
     * イベントを発行する（標準優先度）
     * @param event 発行するイベント
     */
    publish<K extends keyof EventMap>(
        event: EventMap[K]
    ): void;

    /**
     * イベントを発行する（優先度指定）
     * @param event 発行するイベント
     * @param priority イベントの優先度
     */
    publishWithPriority<K extends keyof EventMap>(
        event: EventMap[K],
        priority: EventPriority
    ): void;

    /**
     * バッチモードを開始する
     * 複数のイベントをまとめて処理するためのモード
     */
    startBatchMode(): void;

    /**
     * バッチモードを終了する
     * まとめられたイベントを処理する
     */
    endBatchMode(): void;

    /**
     * 特定のイベントタイプのハンドラ数を取得する
     * @param eventType イベントの種類
     * @returns ハンドラの数
     */
    handlerCount<K extends keyof EventMap>(eventType: K): number;

    /**
     * すべてのハンドラをクリアする
     */
    clearHandlers(): void;
}; 