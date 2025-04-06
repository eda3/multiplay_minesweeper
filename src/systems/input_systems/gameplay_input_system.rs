/**
 * ゲームプレイ入力システム
 *
 * 入力リソースからのイベントをゲームプレイアクションに変換するシステム
 */
use crate::ecs::system::{System, SystemResult};
use crate::entities::EntityManager;
use crate::resources::{ResourceManager, InputResource, EventQueueResource, BoardResource};
use crate::systems::input_systems::{InputEvent, InputEventType, MouseButton};
use crate::resources::resource_trait::Resource;

/// ゲームプレイ入力システム - 入力イベントをゲームアクションに変換
pub struct GameplayInputSystem {
    /// デバッグモード
    debug_mode: bool,
    /// 前回のマウス位置（セル座標）
    last_cell_position: Option<(usize, usize)>,
    /// ドラッグ中フラグ
    dragging: bool,
}

impl Default for GameplayInputSystem {
    fn default() -> Self {
        Self {
            debug_mode: false,
            last_cell_position: None,
            dragging: false,
        }
    }
}

impl GameplayInputSystem {
    /// 新しいゲームプレイ入力システムを作成
    pub fn new() -> Self {
        Self::default()
    }

    /// デバッグモードを設定
    pub fn with_debug(mut self, debug: bool) -> Self {
        self.debug_mode = debug;
        self
    }

    /// マウスイベント処理
    fn process_mouse_event(&mut self, event: &InputEvent, resources: &ResourceManager) -> bool {
        match &event.event_type {
            InputEventType::MouseClick(x, y, button) => {
                self.handle_mouse_click(*x, *y, *button, resources);
                true
            },
            InputEventType::MouseDown(x, y, button) => {
                if *button == MouseButton::Left {
                    self.dragging = true;
                }
                true
            },
            InputEventType::MouseUp(x, y, button) => {
                if *button == MouseButton::Left {
                    self.dragging = false;
                }
                true
            },
            InputEventType::MouseMove(x, y) => {
                if self.dragging {
                    self.handle_drag(*x, *y, resources);
                } else {
                    self.handle_mouse_hover(*x, *y, resources);
                }
                true
            },
            _ => false,
        }
    }

    /// マウスクリック処理
    fn handle_mouse_click(&mut self, x: f64, y: f64, button: MouseButton, resources: &ResourceManager) {
        // ボード上のセル位置を計算
        if let Some((row, col)) = self.calculate_cell_position(x, y, resources) {
            // イベントキューにゲームイベントを追加
            if let Ok(event_queue_rc) = resources.get::<EventQueueResource>() {
                let mut event_queue = event_queue_rc.borrow_mut();
                if let Some(queue) = event_queue.downcast_mut::<EventQueueResource>() {
                    match button {
                        MouseButton::Left => {
                            // 左クリック: セル公開イベント
                            queue.push_event(crate::resources::event_queue_resource::GameEvent::CellReveal(row, col));
                            if self.debug_mode {
                                println!("セル公開: ({}, {})", row, col);
                            }
                        },
                        MouseButton::Right => {
                            // 右クリック: フラグトグルイベント
                            queue.push_event(crate::resources::event_queue_resource::GameEvent::FlagToggle(row, col));
                            if self.debug_mode {
                                println!("フラグトグル: ({}, {})", row, col);
                            }
                        },
                        _ => {},
                    }
                }
            }
        }
    }

    /// マウスドラッグ処理
    fn handle_drag(&mut self, x: f64, y: f64, resources: &ResourceManager) {
        // ドラッグ時は何もしない（将来的に実装する場合のプレースホルダー）
        if self.debug_mode {
            if let Some((row, col)) = self.calculate_cell_position(x, y, resources) {
                println!("ドラッグ中: ({}, {})", row, col);
            }
        }
    }

    /// マウスホバー処理
    fn handle_mouse_hover(&mut self, x: f64, y: f64, resources: &ResourceManager) {
        let cell_pos = self.calculate_cell_position(x, y, resources);
        
        // 前回と異なるセルにホバーした場合
        if cell_pos != self.last_cell_position {
            // 前回のホバーセルのハイライトを解除
            if let Some((prev_row, prev_col)) = self.last_cell_position {
                self.update_cell_highlight(prev_row, prev_col, false, resources);
            }
            
            // 新しいセルをハイライト
            if let Some((row, col)) = cell_pos {
                self.update_cell_highlight(row, col, true, resources);
            }
            
            self.last_cell_position = cell_pos;
        }
    }

    /// セルのハイライト状態を更新
    fn update_cell_highlight(&self, row: usize, col: usize, highlighted: bool, resources: &ResourceManager) {
        // ボードリソースを使用してセルのハイライト状態を更新
        // 現状はデバッグ出力のみ
        if self.debug_mode {
            if highlighted {
                println!("セルハイライト: ({}, {})", row, col);
            } else {
                println!("セルハイライト解除: ({}, {})", row, col);
            }
        }
    }

    /// マウス座標からセル位置を計算
    fn calculate_cell_position(&self, x: f64, y: f64, resources: &ResourceManager) -> Option<(usize, usize)> {
        // ボードリソースを取得
        if let Ok(board_rc) = resources.get::<BoardResource>() {
            let board = board_rc.borrow();
            if let Some(board) = board.downcast_ref::<BoardResource>() {
                // ボードのセルサイズを取得
                let cell_size = board.cell_size as f64;
                
                // セル座標を計算
                let col = (x / cell_size).floor() as usize;
                let row = (y / cell_size).floor() as usize;
                
                // ボード範囲内かチェック
                if row < board.height && col < board.width {
                    return Some((row, col));
                }
            }
        }
        
        None
    }
}

impl System for GameplayInputSystem {
    fn name(&self) -> &str {
        "GameplayInputSystem"
    }

    fn initialize(&mut self, _entity_manager: &mut EntityManager, _resources: &mut ResourceManager) -> SystemResult {
        SystemResult::Ok
    }

    fn update(&mut self, entity_manager: &mut EntityManager, resources: &mut ResourceManager) -> SystemResult {
        // UIInputSystemの依存関係を確認（実際のシステムでは依存関係は別の方法で管理されるはず）
        
        // 入力リソースを取得
        if let Ok(input_rc) = resources.get::<InputResource>() {
            let mut input = input_rc.borrow_mut();
            if let Some(input) = input.downcast_mut::<InputResource>() {
                // 入力イベントキューからイベントを処理
                let mut event_indices_to_mark = Vec::new();
                
                // イベントキューをコピーして参照の問題を回避
                let events: Vec<InputEvent> = input.event_queue.iter().cloned().collect();
                
                for (i, event) in events.iter().enumerate() {
                    if !event.handled {
                        let processed = self.process_mouse_event(event, resources);
                        if processed {
                            event_indices_to_mark.push(i);
                        }
                    }
                }
                
                // 処理済みイベントをマーク
                for i in event_indices_to_mark {
                    if i < input.event_queue.len() {
                        if let Some(event) = input.event_queue.get_mut(i) {
                            event.mark_handled();
                        }
                    }
                }
            }
        }
        
        SystemResult::Ok
    }

    fn cleanup(&mut self, _entity_manager: &mut EntityManager, _resources: &mut ResourceManager) -> SystemResult {
        // クリーンアップ処理（必要に応じて）
        self.last_cell_position = None;
        self.dragging = false;
        SystemResult::Ok
    }
} 