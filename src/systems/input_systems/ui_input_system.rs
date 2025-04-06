/**
 * UI入力システム
 *
 * 入力リソースからのイベントをUIコンポーネントに転送するシステム
 */
use crate::ecs::system::{System, SystemResult};
use crate::entities::EntityManager;
use crate::resources::{ResourceManager, InputResource, EventQueueResource};
use crate::systems::input_systems::{InputEvent, InputEventType, MouseButton};
use crate::resources::resource_trait::Resource;

/// UI入力システム - 入力イベントをUI要素との相互作用に変換
pub struct UIInputSystem {
    /// 前回のフレームでのマウス位置
    previous_mouse_pos: (f64, f64),
    /// UI要素のホバー状態
    hovering_elements: Vec<String>,
    /// 現在アクティブなUI要素
    active_element: Option<String>,
    /// デバッグモード
    debug_mode: bool,
}

impl Default for UIInputSystem {
    fn default() -> Self {
        Self {
            previous_mouse_pos: (0.0, 0.0),
            hovering_elements: Vec::new(),
            active_element: None,
            debug_mode: false,
        }
    }
}

impl UIInputSystem {
    /// 新しいUI入力システムを作成
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
            InputEventType::MouseMove(x, y) => {
                self.handle_mouse_move(*x, *y, resources);
                true
            },
            InputEventType::MouseClick(x, y, button) => {
                self.handle_mouse_click(*x, *y, *button, resources);
                true
            },
            InputEventType::MouseDown(x, y, button) => {
                self.handle_mouse_down(*x, *y, *button, resources);
                true
            },
            InputEventType::MouseUp(x, y, button) => {
                self.handle_mouse_up(*x, *y, *button, resources);
                true
            },
            _ => false,
        }
    }

    /// マウス移動処理
    fn handle_mouse_move(&mut self, x: f64, y: f64, resources: &ResourceManager) {
        // UI要素検出のためのヒットテスト
        let new_hovering = self.perform_hit_test(x, y, resources);
        
        // ホバー状態が変わった要素を検出
        for element_id in &new_hovering {
            if !self.hovering_elements.contains(element_id) {
                // 新しくホバーになった要素にイベント発行
                self.publish_ui_event("hover_enter", element_id, x, y, resources);
            }
        }
        
        // ホバーから外れた要素を検出
        for element_id in &self.hovering_elements {
            if !new_hovering.contains(element_id) {
                // ホバーから外れた要素にイベント発行
                self.publish_ui_event("hover_exit", element_id, x, y, resources);
            }
        }
        
        // ホバー要素リストを更新
        self.hovering_elements = new_hovering;
        self.previous_mouse_pos = (x, y);
    }

    /// マウスクリック処理
    fn handle_mouse_click(&mut self, x: f64, y: f64, button: MouseButton, resources: &ResourceManager) {
        if button != MouseButton::Left {
            return; // 左クリックのみ処理
        }
        
        // ヒットテストを行い、最前面の要素を特定
        let hit_elements = self.perform_hit_test(x, y, resources);
        
        if let Some(element_id) = hit_elements.first() {
            // イベントキューにUIクリックイベントを追加
            if let Ok(event_queue_rc) = resources.get::<EventQueueResource>() {
                let mut event_queue = event_queue_rc.borrow_mut();
                if let Some(queue) = event_queue.downcast_mut::<EventQueueResource>() {
                    queue.push_event(crate::resources::event_queue_resource::GameEvent::UIClick(element_id.clone()));
                    
                    if self.debug_mode {
                        println!("UI要素クリック: {}", element_id);
                    }
                }
            }
        }
    }

    /// マウスボタン押下処理
    fn handle_mouse_down(&mut self, x: f64, y: f64, button: MouseButton, resources: &ResourceManager) {
        if button != MouseButton::Left {
            return; // 左ボタンのみ処理
        }
        
        // ヒットテストを行い、最前面の要素を特定
        let hit_elements = self.perform_hit_test(x, y, resources);
        
        if let Some(element_id) = hit_elements.first() {
            // アクティブ要素を設定
            self.active_element = Some(element_id.clone());
            
            // イベント発行
            self.publish_ui_event("active", element_id, x, y, resources);
        }
    }

    /// マウスボタン解放処理
    fn handle_mouse_up(&mut self, x: f64, y: f64, button: MouseButton, resources: &ResourceManager) {
        if button != MouseButton::Left {
            return; // 左ボタンのみ処理
        }
        
        // アクティブ要素があれば解放イベントを発行
        if let Some(element_id) = &self.active_element {
            self.publish_ui_event("inactive", element_id, x, y, resources);
            self.active_element = None;
        }
    }

    /// ヒットテスト - 指定座標に存在するUI要素をZオーダーで取得
    fn perform_hit_test(&self, x: f64, y: f64, resources: &ResourceManager) -> Vec<String> {
        // TODO: 実際のUI要素とのヒットテストを実装
        // 簡易的な実装として、ハードコードした範囲チェックを行う
        let mut hit_elements = Vec::new();
        
        // 例: リセットボタンの範囲
        if x >= 10.0 && x <= 110.0 && y >= 10.0 && y <= 50.0 {
            hit_elements.push("reset_button".to_string());
        }
        
        // 例: 難易度選択の範囲
        if x >= 120.0 && x <= 220.0 && y >= 10.0 && y <= 50.0 {
            hit_elements.push("difficulty_selector".to_string());
        }
        
        hit_elements
    }

    /// UI要素に対するイベントを発行
    fn publish_ui_event(&self, event_type: &str, element_id: &str, x: f64, y: f64, resources: &ResourceManager) {
        // 実際のアプリケーションでは、より詳細なUIイベントを発行する
        if self.debug_mode {
            println!("UIイベント: {} - 要素: {} - 座標: ({}, {})", event_type, element_id, x, y);
        }
    }
}

impl System for UIInputSystem {
    fn name(&self) -> &str {
        "UIInputSystem"
    }

    fn initialize(&mut self, _entity_manager: &mut EntityManager, _resources: &mut ResourceManager) -> SystemResult {
        SystemResult::Ok
    }

    fn update(&mut self, entity_manager: &mut EntityManager, resources: &mut ResourceManager) -> SystemResult {
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
        self.hovering_elements.clear();
        self.active_element = None;
        SystemResult::Ok
    }
} 