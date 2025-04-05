/**
 * 入力収集システム
 * 
 * DOMイベントを収集し、InputResourceに変換して追加する
 */
use std::rc::Rc;
use std::cell::RefCell;
use std::any::Any;

use crate::system::System;
use crate::system::system_registry::SystemPhase;
use crate::resources::{ResourceManager, InputResource, EventQueueResource, GameEvent};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{Event, MouseEvent, Document, Window};

/// 入力収集システム
pub struct InputCollectionSystem {
    /// システム名
    name: String,
    /// 初期化済みかどうか
    initialized: bool,
    /// イベントリスナーのコールバック
    _event_callbacks: Vec<Closure<dyn FnMut(Event)>>,
}

impl InputCollectionSystem {
    /// 新しい入力収集システムを作成
    pub fn new() -> Self {
        Self {
            name: "InputCollectionSystem".to_string(),
            initialized: false,
            _event_callbacks: Vec::new(),
        }
    }

    /// DOMイベントリスナーを設定
    fn setup_event_listeners(&mut self) -> Result<(), JsValue> {
        // windowオブジェクトを取得
        let window = web_sys::window().expect("ウィンドウが見つかりません");
        let document = window.document().expect("ドキュメントが見つかりません");
        
        // キーボードイベントリスナー
        self.setup_keyboard_listeners(&window)?;
        
        // マウスイベントリスナー
        self.setup_mouse_listeners(&document)?;
        
        Ok(())
    }

    /// キーボードイベントリスナーをセットアップ
    fn setup_keyboard_listeners(&mut self, window: &Window) -> Result<(), JsValue> {
        // キーダウンイベント
        let keydown_callback = Closure::<dyn FnMut(_)>::new(move |event: Event| {
            // InputResourceに後でアクセスして更新する
            // 現時点ではDOMイベントをキャプチャするだけ
            let code = js_sys::Reflect::get(&event, &JsValue::from_str("code"))
                .unwrap_or(JsValue::from_str(""))
                .as_string()
                .unwrap_or_default();
            
            // 今はログだけ出す
            web_sys::console::log_1(&JsValue::from_str(&format!("Key down: {}", code)));
            
            // デフォルト動作を防止
            event.prevent_default();
        });
        
        window.add_event_listener_with_callback("keydown", keydown_callback.as_ref().unchecked_ref())?;
        self._event_callbacks.push(keydown_callback);
        
        // キーアップイベント
        let keyup_callback = Closure::<dyn FnMut(_)>::new(move |event: Event| {
            let code = js_sys::Reflect::get(&event, &JsValue::from_str("code"))
                .unwrap_or(JsValue::from_str(""))
                .as_string()
                .unwrap_or_default();
            
            web_sys::console::log_1(&JsValue::from_str(&format!("Key up: {}", code)));
        });
        
        window.add_event_listener_with_callback("keyup", keyup_callback.as_ref().unchecked_ref())?;
        self._event_callbacks.push(keyup_callback);
        
        Ok(())
    }

    /// マウスイベントリスナーをセットアップ
    fn setup_mouse_listeners(&mut self, document: &Document) -> Result<(), JsValue> {
        // マウス移動イベント
        let mousemove_callback = Closure::<dyn FnMut(_)>::new(move |event: Event| {
            let mouse_event = event.dyn_ref::<MouseEvent>().unwrap();
            let x = mouse_event.client_x();
            let y = mouse_event.client_y();
            
            web_sys::console::log_1(&JsValue::from_str(&format!("Mouse move: ({}, {})", x, y)));
        });
        
        document.add_event_listener_with_callback("mousemove", mousemove_callback.as_ref().unchecked_ref())?;
        self._event_callbacks.push(mousemove_callback);
        
        // マウスダウンイベント
        let mousedown_callback = Closure::<dyn FnMut(_)>::new(move |event: Event| {
            let mouse_event = event.dyn_ref::<MouseEvent>().unwrap();
            let button = mouse_event.button();
            
            web_sys::console::log_1(&JsValue::from_str(&format!("Mouse down: button {}", button)));
        });
        
        document.add_event_listener_with_callback("mousedown", mousedown_callback.as_ref().unchecked_ref())?;
        self._event_callbacks.push(mousedown_callback);
        
        // マウスアップイベント
        let mouseup_callback = Closure::<dyn FnMut(_)>::new(move |event: Event| {
            let mouse_event = event.dyn_ref::<MouseEvent>().unwrap();
            let button = mouse_event.button();
            
            web_sys::console::log_1(&JsValue::from_str(&format!("Mouse up: button {}", button)));
        });
        
        document.add_event_listener_with_callback("mouseup", mouseup_callback.as_ref().unchecked_ref())?;
        self._event_callbacks.push(mouseup_callback);
        
        // コンテキストメニュー防止
        let contextmenu_callback = Closure::<dyn FnMut(_)>::new(move |event: Event| {
            event.prevent_default();
        });
        
        document.add_event_listener_with_callback("contextmenu", contextmenu_callback.as_ref().unchecked_ref())?;
        self._event_callbacks.push(contextmenu_callback);
        
        Ok(())
    }
}

impl System for InputCollectionSystem {
    fn name(&self) -> &str {
        &self.name
    }
    
    fn phase(&self) -> SystemPhase {
        SystemPhase::Input
    }
    
    fn run(&mut self, resources: &mut ResourceManager) {
        // 初期化されていなければイベントリスナーをセットアップ
        if !self.initialized {
            if let Err(e) = self.setup_event_listeners() {
                web_sys::console::error_1(&JsValue::from_str(&format!("イベントリスナーのセットアップに失敗しました: {:?}", e)));
            } else {
                self.initialized = true;
            }
        }
        
        // InputResourceを取得して更新
        if let Ok(input_resource_rc) = resources.get_mut::<InputResource>() {
            let mut input_resource = input_resource_rc.borrow_mut();
            let input_resource = input_resource.downcast_mut::<InputResource>().unwrap();
            
            // フレーム更新
            input_resource.update();
        }
        
        // このフレームで収集した入力イベントをEventQueueResourceに追加
        // 現在は実際のイベント収集はDOMイベントリスナーでまだ行われていない
        // 将来的にはここでEventQueueResourceにイベントを追加する
        if let Ok(event_queue_rc) = resources.get_mut::<EventQueueResource>() {
            // ここでイベントキューにイベントを追加する
            let mut event_queue = event_queue_rc.borrow_mut();
            let event_queue = event_queue.downcast_mut::<EventQueueResource>().unwrap();
            
            // まだコールバックからイベントを取得できないため、
            // ここでは簡単な例としてマウス移動イベントを生成
            
            // 例：毎フレームダミーのマウス移動イベントを追加
            // 実際のコードでは、DOM側で収集したイベントを追加する
            if self.initialized {
                // ダミーイベント（デモ用）
                // event_queue.push_event(GameEvent::MouseMove(0.0, 0.0));
            }
        }
    }
} 