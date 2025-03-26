/**
 * リソース定義
 * 
 * システム間で共有するリソースをまとめたモジュール
 */
use std::collections::VecDeque;
use web_sys::{CanvasRenderingContext2d, WebSocket, CloseEvent, MessageEvent};
use wasm_bindgen::{JsCast, JsValue, closure::Closure};

/// ゲーム状態リソース
#[derive(Debug, Clone)]
pub struct GameStateResource {
    /// 現在のゲーム状態（loading, menu, game, game_overなど）
    pub current_state: String,
    /// 勝利状態かどうか
    pub is_victory: bool,
    /// ゲーム開始からの経過時間（秒）
    pub elapsed_time: f64,
    /// プレイヤー数
    pub player_count: usize,
    /// 自分のプレイヤーID
    pub player_id: Option<String>,
}

impl Default for GameStateResource {
    fn default() -> Self {
        Self {
            current_state: "loading".to_string(),
            is_victory: false,
            elapsed_time: 0.0,
            player_count: 1,
            player_id: None,
        }
    }
}

/// プレイヤーリソース
#[derive(Debug, Clone)]
pub struct PlayerResource {
    /// プレイヤーID
    pub player_id: Option<String>,
    /// プレイヤー名
    pub player_name: String,
    /// 自分のカラー
    pub player_color: String,
    /// スコア
    pub score: u32,
}

impl Default for PlayerResource {
    fn default() -> Self {
        Self {
            player_id: None,
            player_name: "Player".to_string(),
            player_color: "#FF0000".to_string(),
            score: 0,
        }
    }
}

/// 入力リソース
#[derive(Debug, Clone)]
pub struct InputResource {
    /// 最後のマウスイベント
    pub last_mouse_event: Option<web_sys::MouseEvent>,
    /// 最後のキーボードイベント
    pub last_key_event: Option<web_sys::KeyboardEvent>,
    /// キーの状態マップ（どのキーが押されているか）
    pub keys_pressed: std::collections::HashMap<String, bool>,
}

impl Default for InputResource {
    fn default() -> Self {
        Self {
            last_mouse_event: None,
            last_key_event: None,
            keys_pressed: std::collections::HashMap::new(),
        }
    }
}

/// 描画リソース
#[derive(Debug, Clone)]
pub struct RenderResource {
    /// キャンバス描画コンテキスト
    pub context: CanvasRenderingContext2d,
    /// キャンバス幅
    pub canvas_width: f64,
    /// キャンバス高さ
    pub canvas_height: f64,
    /// 前回の描画時間
    pub last_render_time: f64,
}

/// ボードリソース
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BoardResource {
    /// ボードの幅（セル数）
    pub width: usize,
    /// ボードの高さ（セル数）
    pub height: usize,
    /// 地雷の数
    pub mine_count: usize,
    /// セルの値（-1: 地雷, 0: 空白, 1-8: 周囲の地雷数）
    pub cells: Vec<i8>,
    /// セルが公開されているかどうか
    pub revealed: Vec<bool>,
    /// セルにフラグが立てられているかどうか
    pub flagged: Vec<bool>,
    /// ゲームがスタートしたかどうか
    pub game_started: bool,
    /// ゲームオーバーフラグ
    pub game_over: bool,
    /// 勝利フラグ
    pub game_won: bool,
    /// セルのサイズ（ピクセル）
    pub cell_size: f64,
    /// 更新フラグ（他のコンポーネントに更新を通知）
    #[serde(skip)]
    pub is_updated: bool,
}

impl BoardResource {
    /// 新しいボードリソースを作成
    pub fn new(width: usize, height: usize, mine_count: usize, cell_size: f64) -> Self {
        let size = width * height;
        Self {
            width,
            height,
            mine_count,
            cells: vec![0; size],
            revealed: vec![false; size],
            flagged: vec![false; size],
            game_started: false,
            game_over: false,
            game_won: false,
            cell_size,
            is_updated: true,
        }
    }
    
    /// ボードを初期化（地雷を設置）
    pub fn initialize(&mut self) {
        let size = self.width * self.height;
        self.cells = vec![0; size];
        self.revealed = vec![false; size];
        self.flagged = vec![false; size];
        self.game_started = false;
        self.game_over = false;
        self.game_won = false;
        self.is_updated = true;
    }
    
    /// 特定のセルを公開
    pub fn reveal_cell(&mut self, x: usize, y: usize) {
        if x >= self.width || y >= self.height {
            return;
        }
        
        let idx = y * self.width + x;
        
        // すでに公開済みまたはフラグが立っていれば何もしない
        if self.revealed[idx] || self.flagged[idx] {
            return;
        }
        
        // まだゲームが開始していなければ、最初のクリックで地雷を配置
        if !self.game_started {
            self.place_mines(x, y);
            self.game_started = true;
        }
        
        // セルを公開
        self.revealed[idx] = true;
        
        // 地雷を踏んだ場合はゲームオーバー
        if self.cells[idx] == -1 {
            self.game_over = true;
            self.is_updated = true;
            return;
        }
        
        // 周囲に地雷がない場合は連鎖的に公開
        if self.cells[idx] == 0 {
            self.reveal_adjacent_cells(x, y);
        }
        
        self.is_updated = true;
    }
    
    /// 地雷を配置
    fn place_mines(&mut self, first_x: usize, first_y: usize) {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let size = self.width * self.height;
        
        // 最初にクリックしたセルとその周囲には地雷を置かない
        let mut safe_cells = Vec::new();
        for dy in -1..=1 {
            for dx in -1..=1 {
                let nx = first_x as isize + dx;
                let ny = first_y as isize + dy;
                
                if nx >= 0 && nx < self.width as isize && ny >= 0 && ny < self.height as isize {
                    safe_cells.push(ny as usize * self.width + nx as usize);
                }
            }
        }
        
        // 地雷を配置
        let mut mines_placed = 0;
        while mines_placed < self.mine_count {
            let idx = rng.gen_range(0..size);
            
            // 安全なセルには地雷を置かない
            if !safe_cells.contains(&idx) && self.cells[idx] != -1 {
                self.cells[idx] = -1;
                mines_placed += 1;
                
                // 周囲のセルの数字を更新
                let x = idx % self.width;
                let y = idx / self.width;
                
                for dy in -1..=1 {
                    for dx in -1..=1 {
                        if dx == 0 && dy == 0 {
                            continue;
                        }
                        
                        let nx = x as isize + dx;
                        let ny = y as isize + dy;
                        
                        if nx >= 0 && nx < self.width as isize && ny >= 0 && ny < self.height as isize {
                            let nidx = ny as usize * self.width + nx as usize;
                            if self.cells[nidx] != -1 {
                                self.cells[nidx] += 1;
                            }
                        }
                    }
                }
            }
        }
    }
    
    /// 周囲のセルを連鎖的に公開
    fn reveal_adjacent_cells(&mut self, x: usize, y: usize) {
        for dy in -1..=1 {
            for dx in -1..=1 {
                if dx == 0 && dy == 0 {
                    continue;
                }
                
                let nx = x as isize + dx;
                let ny = y as isize + dy;
                
                if nx >= 0 && nx < self.width as isize && ny >= 0 && ny < self.height as isize {
                    let nidx = ny as usize * self.width + nx as usize;
                    
                    // まだ公開されておらず、フラグが立っていなければ公開
                    if !self.revealed[nidx] && !self.flagged[nidx] {
                        self.revealed[nidx] = true;
                        
                        // 周囲に地雷がなければさらに連鎖的に公開
                        if self.cells[nidx] == 0 {
                            self.reveal_adjacent_cells(nx as usize, ny as usize);
                        }
                    }
                }
            }
        }
    }
    
    /// フラグを切り替え
    pub fn toggle_flag(&mut self, x: usize, y: usize) {
        if x >= self.width || y >= self.height {
            return;
        }
        
        let idx = y * self.width + x;
        
        // すでに公開済みなら何もしない
        if self.revealed[idx] {
            return;
        }
        
        // フラグの切り替え
        self.flagged[idx] = !self.flagged[idx];
        self.is_updated = true;
    }
}

/// ネットワークリソース
#[derive(Debug, Clone)]
pub struct NetworkResource {
    /// WebSocket接続
    #[allow(dead_code)]
    socket: Option<WebSocket>,
    /// 接続状態
    pub connected: bool,
    /// メッセージキュー
    pub message_queue: VecDeque<String>,
    /// 接続クロージャ参照（GCから守るため）
    #[allow(dead_code)]
    socket_open_callback: Option<Closure<dyn FnMut(JsValue)>>,
    /// メッセージ受信クロージャ参照
    #[allow(dead_code)]
    socket_message_callback: Option<Closure<dyn FnMut(MessageEvent)>>,
    /// エラークロージャ参照
    #[allow(dead_code)]
    socket_error_callback: Option<Closure<dyn FnMut(JsValue)>>,
    /// 切断クロージャ参照
    #[allow(dead_code)]
    socket_close_callback: Option<Closure<dyn FnMut(CloseEvent)>>,
}

impl NetworkResource {
    /// 新しいネットワークリソースを作成
    pub fn new() -> Self {
        Self {
            socket: None,
            connected: false,
            message_queue: VecDeque::new(),
            socket_open_callback: None,
            socket_message_callback: None,
            socket_error_callback: None,
            socket_close_callback: None,
        }
    }
    
    /// WebSocket接続を確立
    pub fn connect(&mut self, url: &str) -> Result<(), JsValue> {
        // WebSocketを作成
        let socket = WebSocket::new(url)?;
        
        // バイナリ形式を指定
        socket.set_binary_type(web_sys::BinaryType::Arraybuffer);
        
        // オープンコールバック
        let open_callback = {
            let socket_clone = socket.clone();
            Closure::wrap(Box::new(move |_| {
                web_sys::console::log_1(&"WebSocket connection established".into());
                // 接続成功したら自分のIDをリクエスト
                let _ = socket_clone.send_with_str("join");
            }) as Box<dyn FnMut(JsValue)>)
        };
        socket.set_onopen(Some(open_callback.as_ref().unchecked_ref()));
        
        // メッセージ受信コールバック
        let message_queue = std::rc::Rc::new(std::cell::RefCell::new(self.message_queue.clone()));
        let message_callback = {
            let message_queue = message_queue.clone();
            Closure::wrap(Box::new(move |event: MessageEvent| {
                if let Ok(txt) = event.data().dyn_into::<js_sys::JsString>() {
                    let message = String::from(txt);
                    message_queue.borrow_mut().push_back(message);
                }
            }) as Box<dyn FnMut(MessageEvent)>)
        };
        socket.set_onmessage(Some(message_callback.as_ref().unchecked_ref()));
        
        // エラーコールバック
        let error_callback = Closure::wrap(Box::new(move |e| {
            web_sys::console::error_1(&format!("WebSocket error: {:?}", e).into());
        }) as Box<dyn FnMut(JsValue)>);
        socket.set_onerror(Some(error_callback.as_ref().unchecked_ref()));
        
        // 切断コールバック
        let close_callback = Closure::wrap(Box::new(move |e: CloseEvent| {
            web_sys::console::log_1(&format!("WebSocket closed: {}", e.reason()).into());
        }) as Box<dyn FnMut(CloseEvent)>);
        socket.set_onclose(Some(close_callback.as_ref().unchecked_ref()));
        
        // 状態を更新
        self.socket = Some(socket);
        self.socket_open_callback = Some(open_callback);
        self.socket_message_callback = Some(message_callback);
        self.socket_error_callback = Some(error_callback);
        self.socket_close_callback = Some(close_callback);
        
        Ok(())
    }
    
    /// メッセージを送信
    pub fn send_message(&self, message: &str) -> Result<(), JsValue> {
        if let Some(socket) = &self.socket {
            if socket.ready_state() == web_sys::WebSocket::OPEN {
                return socket.send_with_str(message);
            }
        }
        
        Err(JsValue::from_str("WebSocket not connected"))
    }
    
    /// 新しいメッセージがあるかチェック
    pub fn has_messages(&self) -> bool {
        !self.message_queue.is_empty()
    }
    
    /// 次のメッセージを取得
    pub fn next_message(&mut self) -> Option<String> {
        self.message_queue.pop_front()
    }
}

/// タイマーリソース
#[derive(Debug, Clone)]
pub struct TimerResource {
    /// タイマーID
    pub id: String,
    /// 現在の時間
    pub current_time: f64,
    /// タイマー期間（0.0の場合は無制限）
    pub duration: f64,
    /// 実行中フラグ
    pub is_running: bool,
    /// 完了フラグ
    pub is_completed: bool,
}

impl TimerResource {
    /// 新しいタイマーを作成
    pub fn new(id: &str, duration: f64) -> Self {
        Self {
            id: id.to_string(),
            current_time: 0.0,
            duration,
            is_running: false,
            is_completed: false,
        }
    }
    
    /// タイマーを開始
    pub fn start(&mut self) {
        self.is_running = true;
        self.is_completed = false;
    }
    
    /// タイマーを停止
    pub fn stop(&mut self) {
        self.is_running = false;
    }
    
    /// タイマーをリセット
    pub fn reset(&mut self) {
        self.current_time = 0.0;
        self.is_completed = false;
    }
}

/// UIリソース
#[derive(Debug, Clone)]
pub struct UiResource {
    /// ロード画面の進捗（0.0〜1.0）
    pub loading_progress: f64,
    /// 選択された難易度（0: Easy, 1: Medium, 2: Hard）
    pub selected_difficulty: usize,
    /// メニューボタン
    pub menu_buttons: Vec<UiButton>,
    /// UIメッセージ
    pub messages: Vec<UiMessage>,
}

/// UIボタン
#[derive(Debug, Clone)]
pub struct UiButton {
    /// ボタンテキスト
    pub text: String,
    /// ボタンID
    pub id: String,
    /// ホバー状態かどうか
    pub is_hovered: bool,
    /// クリック状態かどうか
    pub is_clicked: bool,
}

/// UIメッセージ
#[derive(Debug, Clone)]
pub struct UiMessage {
    /// メッセージテキスト
    pub text: String,
    /// 表示時間（秒）
    pub duration: f64,
    /// 経過時間
    pub elapsed: f64,
    /// メッセージタイプ（info, error, warningなど）
    pub message_type: String,
} 