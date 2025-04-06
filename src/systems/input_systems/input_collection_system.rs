/**
 * 入力収集システム
 * 
 * DOMイベントをキャプチャし、InputResourceにイベントとして格納する
 */

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{
    window, Document, Event, EventTarget, MouseEvent, 
    KeyboardEvent, WheelEvent, TouchEvent, Touch,
};
use std::cell::RefCell;
use std::rc::Rc;

use crate::resources::input_resource::InputResource;
use crate::systems::input_systems::{InputEvent, InputEventType, MouseButton};
use crate::ecs::system::{System, SystemResult};
use crate::resources::ResourceManager;
use crate::entities::EntityManager;

// WASMで使用するためのマーカー属性
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn error(s: &str);
}

/// DOM入力イベントを収集してInputResourceに変換するシステム
#[derive(Default)]
pub struct InputCollectionSystem {
    /// システムが初期化されたかどうか
    initialized: bool,
    /// イベントリスナーのクリーンアップ用クロージャ保存リスト
    cleanup_indices: Vec<usize>,
    /// ドキュメント参照（キャッシュ用）
    document_set: bool,
    /// システムが有効かどうか
    enabled: bool,
}

// グローバルなクロージャ保存用のコンテナ（WASMでのデータ所有権問題回避用）
#[cfg(target_arch = "wasm32")]
thread_local! {
    static GLOBAL_CLOSURES: RefCell<Vec<Closure<dyn FnMut(Event)>>> = RefCell::new(Vec::new());
}

impl InputCollectionSystem {
    /// 新しい入力収集システムを作成
    pub fn new() -> Self {
        Self {
            initialized: false,
            cleanup_indices: Vec::new(),
            document_set: false,
            enabled: true,
        }
    }
    
    /// イベントリスナーを設定
    fn setup_event_listeners(&mut self, input_resource: &mut InputResource) -> Result<(), String> {
        // すでに初期化済みなら何もしない
        if self.initialized {
            return Ok(());
        }
        
        // ドキュメントを取得
        let window = match window() {
            Some(win) => win,
            None => return Err("ウィンドウが見つかりません".to_string()),
        };
        
        let document = match window.document() {
            Some(doc) => doc,
            None => return Err("ドキュメントが見つかりません".to_string()),
        };
        
        self.document_set = true;
        
        // キーボードイベントリスナーを設定
        self.add_keydown_listener(&document, input_resource)?;
        self.add_keyup_listener(&document, input_resource)?;
        
        // マウスイベントリスナーを設定
        self.add_mousemove_listener(&document, input_resource)?;
        self.add_mousedown_listener(&document, input_resource)?;
        self.add_mouseup_listener(&document, input_resource)?;
        self.add_wheel_listener(&document, input_resource)?;
        
        // タッチイベントリスナーを設定
        self.add_touchstart_listener(&document, input_resource)?;
        self.add_touchmove_listener(&document, input_resource)?;
        self.add_touchend_listener(&document, input_resource)?;
        
        self.initialized = true;
        Ok(())
    }
    
    /// キーダウンイベントリスナーを追加
    fn add_keydown_listener(&mut self, target: &EventTarget, input_resource: &mut InputResource) -> Result<(), String> {
        // 同じ入力リソースへの参照をJavaScriptのクロージャから安全に使用するため、グローバルに保存
        let resource_ptr: *mut InputResource = input_resource;
        
        #[cfg(target_arch = "wasm32")]
        let closure = Closure::wrap(Box::new(move |event: Event| {
            // イベントのデフォルト動作を防止
            event.prevent_default();
            
            // 安全でない参照を使用して入力リソースを取得（WASMでは単一スレッドなので安全）
            let input_resource = unsafe { &mut *resource_ptr };
            
            if let Some(_) = event.dyn_ref::<KeyboardEvent>() {
                input_resource.handle_key_down(&event);
            }
        }) as Box<dyn FnMut(Event)>);
        
        // イベントリスナーを追加
        #[cfg(target_arch = "wasm32")]
        if let Err(e) = target.add_event_listener_with_callback(
            "keydown",
            closure.as_ref().unchecked_ref()
        ) {
            return Err(format!("キーダウンリスナーの追加に失敗: {:?}", e));
        }
        
        // クロージャを保存して参照をキープ
        #[cfg(target_arch = "wasm32")]
        GLOBAL_CLOSURES.with(|closures| {
            let index = closures.borrow().len();
            closures.borrow_mut().push(closure);
            self.cleanup_indices.push(index);
        });
        
        Ok(())
    }
    
    /// キーアップイベントリスナーを追加
    fn add_keyup_listener(&mut self, target: &EventTarget, input_resource: &mut InputResource) -> Result<(), String> {
        // 同じ入力リソースへの参照をJavaScriptのクロージャから安全に使用するため、グローバルに保存
        let resource_ptr: *mut InputResource = input_resource;
        
        #[cfg(target_arch = "wasm32")]
        let closure = Closure::wrap(Box::new(move |event: Event| {
            // イベントのデフォルト動作を防止
            event.prevent_default();
            
            // 安全でない参照を使用して入力リソースを取得（WASMでは単一スレッドなので安全）
            let input_resource = unsafe { &mut *resource_ptr };
            
            if let Some(_) = event.dyn_ref::<KeyboardEvent>() {
                input_resource.handle_key_up(&event);
            }
        }) as Box<dyn FnMut(Event)>);
        
        // イベントリスナーを追加
        #[cfg(target_arch = "wasm32")]
        if let Err(e) = target.add_event_listener_with_callback(
            "keyup",
            closure.as_ref().unchecked_ref()
        ) {
            return Err(format!("キーアップリスナーの追加に失敗: {:?}", e));
        }
        
        // クロージャを保存して参照をキープ
        #[cfg(target_arch = "wasm32")]
        GLOBAL_CLOSURES.with(|closures| {
            let index = closures.borrow().len();
            closures.borrow_mut().push(closure);
            self.cleanup_indices.push(index);
        });
        
        Ok(())
    }
    
    /// マウス移動イベントリスナーを追加
    fn add_mousemove_listener(&mut self, target: &EventTarget, input_resource: &mut InputResource) -> Result<(), String> {
        // 同じ入力リソースへの参照をJavaScriptのクロージャから安全に使用するため、グローバルに保存
        let resource_ptr: *mut InputResource = input_resource;
        
        #[cfg(target_arch = "wasm32")]
        let closure = Closure::wrap(Box::new(move |event: Event| {
            // 安全でない参照を使用して入力リソースを取得（WASMでは単一スレッドなので安全）
            let input_resource = unsafe { &mut *resource_ptr };
            
            if let Some(_) = event.dyn_ref::<MouseEvent>() {
                input_resource.handle_mouse_move(&event);
            }
        }) as Box<dyn FnMut(Event)>);
        
        // イベントリスナーを追加
        #[cfg(target_arch = "wasm32")]
        if let Err(e) = target.add_event_listener_with_callback(
            "mousemove",
            closure.as_ref().unchecked_ref()
        ) {
            return Err(format!("マウス移動リスナーの追加に失敗: {:?}", e));
        }
        
        // クロージャを保存して参照をキープ
        #[cfg(target_arch = "wasm32")]
        GLOBAL_CLOSURES.with(|closures| {
            let index = closures.borrow().len();
            closures.borrow_mut().push(closure);
            self.cleanup_indices.push(index);
        });
        
        Ok(())
    }
    
    /// マウスダウンイベントリスナーを追加
    fn add_mousedown_listener(&mut self, target: &EventTarget, input_resource: &mut InputResource) -> Result<(), String> {
        // 同じ入力リソースへの参照をJavaScriptのクロージャから安全に使用するため、グローバルに保存
        let resource_ptr: *mut InputResource = input_resource;
        
        #[cfg(target_arch = "wasm32")]
        let closure = Closure::wrap(Box::new(move |event: Event| {
            // 安全でない参照を使用して入力リソースを取得（WASMでは単一スレッドなので安全）
            let input_resource = unsafe { &mut *resource_ptr };
            
            if let Some(_) = event.dyn_ref::<MouseEvent>() {
                input_resource.handle_mouse_down(&event);
            }
        }) as Box<dyn FnMut(Event)>);
        
        // イベントリスナーを追加
        #[cfg(target_arch = "wasm32")]
        if let Err(e) = target.add_event_listener_with_callback(
            "mousedown",
            closure.as_ref().unchecked_ref()
        ) {
            return Err(format!("マウスダウンリスナーの追加に失敗: {:?}", e));
        }
        
        // クロージャを保存して参照をキープ
        #[cfg(target_arch = "wasm32")]
        GLOBAL_CLOSURES.with(|closures| {
            let index = closures.borrow().len();
            closures.borrow_mut().push(closure);
            self.cleanup_indices.push(index);
        });
        
        Ok(())
    }
    
    /// マウスアップイベントリスナーを追加
    fn add_mouseup_listener(&mut self, target: &EventTarget, input_resource: &mut InputResource) -> Result<(), String> {
        // 同じ入力リソースへの参照をJavaScriptのクロージャから安全に使用するため、グローバルに保存
        let resource_ptr: *mut InputResource = input_resource;
        
        #[cfg(target_arch = "wasm32")]
        let closure = Closure::wrap(Box::new(move |event: Event| {
            // 安全でない参照を使用して入力リソースを取得（WASMでは単一スレッドなので安全）
            let input_resource = unsafe { &mut *resource_ptr };
            
            if let Some(_) = event.dyn_ref::<MouseEvent>() {
                input_resource.handle_mouse_up(&event);
            }
        }) as Box<dyn FnMut(Event)>);
        
        // イベントリスナーを追加
        #[cfg(target_arch = "wasm32")]
        if let Err(e) = target.add_event_listener_with_callback(
            "mouseup",
            closure.as_ref().unchecked_ref()
        ) {
            return Err(format!("マウスアップリスナーの追加に失敗: {:?}", e));
        }
        
        // クロージャを保存して参照をキープ
        #[cfg(target_arch = "wasm32")]
        GLOBAL_CLOSURES.with(|closures| {
            let index = closures.borrow().len();
            closures.borrow_mut().push(closure);
            self.cleanup_indices.push(index);
        });
        
        Ok(())
    }
    
    /// ホイールイベントリスナーを追加
    fn add_wheel_listener(&mut self, target: &EventTarget, input_resource: &mut InputResource) -> Result<(), String> {
        // 同じ入力リソースへの参照をJavaScriptのクロージャから安全に使用するため、グローバルに保存
        let resource_ptr: *mut InputResource = input_resource;
        
        #[cfg(target_arch = "wasm32")]
        let closure = Closure::wrap(Box::new(move |event: Event| {
            // イベントのデフォルト動作を防止
            event.prevent_default();
            
            // 安全でない参照を使用して入力リソースを取得（WASMでは単一スレッドなので安全）
            let input_resource = unsafe { &mut *resource_ptr };
            
            if let Some(_) = event.dyn_ref::<WheelEvent>() {
                input_resource.handle_wheel(&event);
            }
        }) as Box<dyn FnMut(Event)>);
        
        // イベントリスナーを追加
        #[cfg(target_arch = "wasm32")]
        if let Err(e) = target.add_event_listener_with_callback(
            "wheel",
            closure.as_ref().unchecked_ref()
        ) {
            return Err(format!("ホイールリスナーの追加に失敗: {:?}", e));
        }
        
        // クロージャを保存して参照をキープ
        #[cfg(target_arch = "wasm32")]
        GLOBAL_CLOSURES.with(|closures| {
            let index = closures.borrow().len();
            closures.borrow_mut().push(closure);
            self.cleanup_indices.push(index);
        });
        
        Ok(())
    }
    
    /// タッチスタートイベントリスナーを追加
    fn add_touchstart_listener(&mut self, target: &EventTarget, input_resource: &mut InputResource) -> Result<(), String> {
        // 同じ入力リソースへの参照をJavaScriptのクロージャから安全に使用するため、グローバルに保存
        let resource_ptr: *mut InputResource = input_resource;
        
        #[cfg(target_arch = "wasm32")]
        let closure = Closure::wrap(Box::new(move |event: Event| {
            // 安全でない参照を使用して入力リソースを取得（WASMでは単一スレッドなので安全）
            let input_resource = unsafe { &mut *resource_ptr };
            
            if let Some(_) = event.dyn_ref::<TouchEvent>() {
                input_resource.handle_touch_start(&event);
            }
        }) as Box<dyn FnMut(Event)>);
        
        // イベントリスナーを追加
        #[cfg(target_arch = "wasm32")]
        if let Err(e) = target.add_event_listener_with_callback(
            "touchstart",
            closure.as_ref().unchecked_ref()
        ) {
            return Err(format!("タッチスタートリスナーの追加に失敗: {:?}", e));
        }
        
        // クロージャを保存して参照をキープ
        #[cfg(target_arch = "wasm32")]
        GLOBAL_CLOSURES.with(|closures| {
            let index = closures.borrow().len();
            closures.borrow_mut().push(closure);
            self.cleanup_indices.push(index);
        });
        
        Ok(())
    }
    
    /// タッチ移動イベントリスナーを追加
    fn add_touchmove_listener(&mut self, target: &EventTarget, input_resource: &mut InputResource) -> Result<(), String> {
        // 同じ入力リソースへの参照をJavaScriptのクロージャから安全に使用するため、グローバルに保存
        let resource_ptr: *mut InputResource = input_resource;
        
        #[cfg(target_arch = "wasm32")]
        let closure = Closure::wrap(Box::new(move |event: Event| {
            // 安全でない参照を使用して入力リソースを取得（WASMでは単一スレッドなので安全）
            let input_resource = unsafe { &mut *resource_ptr };
            
            if let Some(_) = event.dyn_ref::<TouchEvent>() {
                input_resource.handle_touch_move(&event);
            }
        }) as Box<dyn FnMut(Event)>);
        
        // イベントリスナーを追加
        #[cfg(target_arch = "wasm32")]
        if let Err(e) = target.add_event_listener_with_callback(
            "touchmove",
            closure.as_ref().unchecked_ref()
        ) {
            return Err(format!("タッチ移動リスナーの追加に失敗: {:?}", e));
        }
        
        // クロージャを保存して参照をキープ
        #[cfg(target_arch = "wasm32")]
        GLOBAL_CLOSURES.with(|closures| {
            let index = closures.borrow().len();
            closures.borrow_mut().push(closure);
            self.cleanup_indices.push(index);
        });
        
        Ok(())
    }
    
    /// タッチエンドイベントリスナーを追加
    fn add_touchend_listener(&mut self, target: &EventTarget, input_resource: &mut InputResource) -> Result<(), String> {
        // 同じ入力リソースへの参照をJavaScriptのクロージャから安全に使用するため、グローバルに保存
        let resource_ptr: *mut InputResource = input_resource;
        
        #[cfg(target_arch = "wasm32")]
        let closure = Closure::wrap(Box::new(move |event: Event| {
            // 安全でない参照を使用して入力リソースを取得（WASMでは単一スレッドなので安全）
            let input_resource = unsafe { &mut *resource_ptr };
            
            if let Some(_) = event.dyn_ref::<TouchEvent>() {
                input_resource.handle_touch_end(&event);
            }
        }) as Box<dyn FnMut(Event)>);
        
        // イベントリスナーを追加
        #[cfg(target_arch = "wasm32")]
        if let Err(e) = target.add_event_listener_with_callback(
            "touchend",
            closure.as_ref().unchecked_ref()
        ) {
            return Err(format!("タッチエンドリスナーの追加に失敗: {:?}", e));
        }
        
        // クロージャを保存して参照をキープ
        #[cfg(target_arch = "wasm32")]
        GLOBAL_CLOSURES.with(|closures| {
            let index = closures.borrow().len();
            closures.borrow_mut().push(closure);
            self.cleanup_indices.push(index);
        });
        
        Ok(())
    }
    
    /// すべてのイベントリスナーのクリーンアップ
    fn cleanup_event_listeners(&mut self) -> Result<(), String> {
        #[cfg(target_arch = "wasm32")]
        GLOBAL_CLOSURES.with(|closures| {
            // クリーンアップ用にクロージャを明示的に解放
            // 実際のDOMイベントリスナーの削除はブラウザのガベージコレクションに任せる
            for index in self.cleanup_indices.drain(..) {
                if index < closures.borrow().len() {
                    // 特に何もしない（クロージャは保持したまま）
                    // WASMの終了時にこれらは自動的に解放される
                }
            }
        });
        
        self.initialized = false;
        self.document_set = false;
        Ok(())
    }
}

impl System for InputCollectionSystem {
    fn update(&mut self, _entity_manager: &mut EntityManager, resources: &mut ResourceManager) -> SystemResult {
        if !self.enabled {
            return SystemResult::Ok;
        }
        
        // 初期化されていなければ初期化を試みる
        if !self.initialized {
            if let Some(input_resource) = resources.get_resource_mut::<InputResource>("") {
                // イベントリスナーを設定
                match self.setup_event_listeners(input_resource) {
                    Ok(_) => {},
                    Err(e) => {
                        web_sys::console::error_1(&format!("イベントリスナーの設定に失敗: {}", e).into());
                        return SystemResult::Error;
                    }
                }
            } else {
                web_sys::console::error_1(&"InputResourceが見つかりません".into());
                return SystemResult::Error;
            }
        }
        
        // すでにすべてのイベントはDOMイベントリスナーによって処理されているため、
        // このメソッドでは特に何もする必要がない
        SystemResult::Ok
    }
    
    fn initialize(&mut self, _entity_manager: &mut EntityManager, resources: &mut ResourceManager) -> SystemResult {
        // 初期化処理
        if let Some(input_resource) = resources.get_resource_mut::<InputResource>("") {
            match self.setup_event_listeners(input_resource) {
                Ok(_) => SystemResult::Ok,
                Err(e) => {
                    web_sys::console::error_1(&format!("イベントリスナーの設定に失敗: {}", e).into());
                    SystemResult::Error
                }
            }
        } else {
            web_sys::console::error_1(&"InputResourceが見つかりません".into());
            SystemResult::Error
        }
    }
    
    fn cleanup(&mut self, _entity_manager: &mut EntityManager, _resources: &mut ResourceManager) -> SystemResult {
        // イベントリスナーをクリーンアップ
        match self.cleanup_event_listeners() {
            Ok(_) => SystemResult::Ok,
            Err(e) => {
                web_sys::console::error_1(&format!("イベントリスナーのクリーンアップに失敗: {}", e).into());
                SystemResult::Error
            }
        }
    }
    
    fn name(&self) -> &str {
        "InputCollectionSystem"
    }
    
    fn enabled(&self) -> bool {
        self.enabled
    }
    
    fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    // モックテスト用の実装（WebシステムのテストはWASMテスト環境が必要）
    #[test]
    fn test_system_creation() {
        let system = InputCollectionSystem::new();
        assert_eq!(system.name(), "InputCollectionSystem");
        assert!(!system.initialized);
        assert!(system.enabled());
    }
} 