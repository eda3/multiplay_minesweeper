/**
 * 入力収集システム
 * 
 * DOMイベントをキャプチャし、InputResourceにイベントとして格納する
 */

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;
#[cfg(target_arch = "wasm32")]
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
    /// JavaScriptイベントリスナーで使用するクロージャを格納するためのグローバルコンテナ
    /// 
    /// # WASM互換性に関する注意
    /// 
    /// - WASMでは`Closure`はJavaScriptに渡した後も有効である必要がある
    /// - `thread_local!`を使用してリソースの所有権問題を回避
    /// - これにより、Rustの所有権システムとJavaScriptのコールバックモデルの互換性を確保
    static GLOBAL_CLOSURES: RefCell<Vec<Closure<dyn FnMut(Event)>>> = RefCell::new(Vec::new());
}

/// JavaScriptイベントリスナーのメモリ管理に関するヘルパー関数
#[cfg(target_arch = "wasm32")]
fn register_closure(closure: Closure<dyn FnMut(Event)>) -> usize {
    // クロージャをグローバルコンテナに保存し、インデックスを返す
    let index = GLOBAL_CLOSURES.with(|closures| {
        let index = closures.borrow().len();
        closures.borrow_mut().push(closure);
        index
    });
    index
}

/// ネイティブ環境用の空のヘルパー関数（コンパイル互換性のため）
#[cfg(not(target_arch = "wasm32"))]
fn register_empty_closure() -> usize {
    0 // ネイティブ環境では何もしない
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
    #[cfg(target_arch = "wasm32")]
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
    
    /// ネイティブ環境用の空のセットアップ関数
    #[cfg(not(target_arch = "wasm32"))]
    fn setup_event_listeners(&mut self, _input_resource: &mut InputResource) -> Result<(), String> {
        // ネイティブ環境では何もしない
        self.initialized = true;
        Ok(())
    }
    
    /// キーダウンイベントリスナーを追加
    #[cfg(target_arch = "wasm32")]
    fn add_keydown_listener(&mut self, target: &EventTarget, input_resource: &mut InputResource) -> Result<(), String> {
        // 同じ入力リソースへの参照をJavaScriptのクロージャから安全に使用するため、グローバルに保存
        let resource_ptr: *mut InputResource = input_resource;
        
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
        if let Err(e) = target.add_event_listener_with_callback(
            "keydown",
            closure.as_ref().unchecked_ref()
        ) {
            return Err(format!("キーダウンリスナーの追加に失敗: {:?}", e));
        }
        
        // クロージャを保存して参照をキープ
        let index = register_closure(closure);
        self.cleanup_indices.push(index);
        
        Ok(())
    }
    
    /// ネイティブ環境用の空の実装
    #[cfg(not(target_arch = "wasm32"))]
    fn add_keydown_listener(&mut self, _target: &(), _input_resource: &mut InputResource) -> Result<(), String> {
        Ok(())
    }
    
    /// キーアップイベントリスナーを追加
    #[cfg(target_arch = "wasm32")]
    fn add_keyup_listener(&mut self, target: &EventTarget, input_resource: &mut InputResource) -> Result<(), String> {
        // 同じ入力リソースへの参照をJavaScriptのクロージャから安全に使用するため、グローバルに保存
        let resource_ptr: *mut InputResource = input_resource;
        
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
        if let Err(e) = target.add_event_listener_with_callback(
            "keyup",
            closure.as_ref().unchecked_ref()
        ) {
            return Err(format!("キーアップリスナーの追加に失敗: {:?}", e));
        }
        
        // クロージャを保存して参照をキープ
        let index = register_closure(closure);
        self.cleanup_indices.push(index);
        
        Ok(())
    }
    
    /// ネイティブ環境用の空の実装
    #[cfg(not(target_arch = "wasm32"))]
    fn add_keyup_listener(&mut self, _target: &(), _input_resource: &mut InputResource) -> Result<(), String> {
        Ok(())
    }
    
    /// マウス移動イベントリスナーを追加
    #[cfg(target_arch = "wasm32")]
    fn add_mousemove_listener(&mut self, target: &EventTarget, input_resource: &mut InputResource) -> Result<(), String> {
        // 同じ入力リソースへの参照をJavaScriptのクロージャから安全に使用するため、グローバルに保存
        let resource_ptr: *mut InputResource = input_resource;
        
        let closure = Closure::wrap(Box::new(move |event: Event| {
            // 安全でない参照を使用して入力リソースを取得（WASMでは単一スレッドなので安全）
            let input_resource = unsafe { &mut *resource_ptr };
            
            if let Some(_) = event.dyn_ref::<MouseEvent>() {
                input_resource.handle_mouse_move(&event);
            }
        }) as Box<dyn FnMut(Event)>);
        
        // イベントリスナーを追加
        if let Err(e) = target.add_event_listener_with_callback(
            "mousemove",
            closure.as_ref().unchecked_ref()
        ) {
            return Err(format!("マウス移動リスナーの追加に失敗: {:?}", e));
        }
        
        // クロージャを保存して参照をキープ
        let index = register_closure(closure);
        self.cleanup_indices.push(index);
        
        Ok(())
    }
    
    /// ネイティブ環境用の空の実装
    #[cfg(not(target_arch = "wasm32"))]
    fn add_mousemove_listener(&mut self, _target: &(), _input_resource: &mut InputResource) -> Result<(), String> {
        Ok(())
    }
    
    /// マウスダウンイベントリスナーを追加
    #[cfg(target_arch = "wasm32")]
    fn add_mousedown_listener(&mut self, target: &EventTarget, input_resource: &mut InputResource) -> Result<(), String> {
        // 同じ入力リソースへの参照をJavaScriptのクロージャから安全に使用するため、グローバルに保存
        let resource_ptr: *mut InputResource = input_resource;
        
        let closure = Closure::wrap(Box::new(move |event: Event| {
            // 安全でない参照を使用して入力リソースを取得（WASMでは単一スレッドなので安全）
            let input_resource = unsafe { &mut *resource_ptr };
            
            if let Some(_) = event.dyn_ref::<MouseEvent>() {
                input_resource.handle_mouse_down(&event);
            }
        }) as Box<dyn FnMut(Event)>);
        
        // イベントリスナーを追加
        if let Err(e) = target.add_event_listener_with_callback(
            "mousedown",
            closure.as_ref().unchecked_ref()
        ) {
            return Err(format!("マウスダウンリスナーの追加に失敗: {:?}", e));
        }
        
        // クロージャを保存して参照をキープ
        let index = register_closure(closure);
        self.cleanup_indices.push(index);
        
        Ok(())
    }
    
    /// ネイティブ環境用の空の実装
    #[cfg(not(target_arch = "wasm32"))]
    fn add_mousedown_listener(&mut self, _target: &(), _input_resource: &mut InputResource) -> Result<(), String> {
        Ok(())
    }
    
    /// マウスアップイベントリスナーを追加
    #[cfg(target_arch = "wasm32")]
    fn add_mouseup_listener(&mut self, target: &EventTarget, input_resource: &mut InputResource) -> Result<(), String> {
        // 同じ入力リソースへの参照をJavaScriptのクロージャから安全に使用するため、グローバルに保存
        let resource_ptr: *mut InputResource = input_resource;
        
        let closure = Closure::wrap(Box::new(move |event: Event| {
            // 安全でない参照を使用して入力リソースを取得（WASMでは単一スレッドなので安全）
            let input_resource = unsafe { &mut *resource_ptr };
            
            if let Some(_) = event.dyn_ref::<MouseEvent>() {
                input_resource.handle_mouse_up(&event);
            }
        }) as Box<dyn FnMut(Event)>);
        
        // イベントリスナーを追加
        if let Err(e) = target.add_event_listener_with_callback(
            "mouseup",
            closure.as_ref().unchecked_ref()
        ) {
            return Err(format!("マウスアップリスナーの追加に失敗: {:?}", e));
        }
        
        // クロージャを保存して参照をキープ
        let index = register_closure(closure);
        self.cleanup_indices.push(index);
        
        Ok(())
    }
    
    /// ネイティブ環境用の空の実装
    #[cfg(not(target_arch = "wasm32"))]
    fn add_mouseup_listener(&mut self, _target: &(), _input_resource: &mut InputResource) -> Result<(), String> {
        Ok(())
    }
    
    /// ホイールイベントリスナーを追加
    #[cfg(target_arch = "wasm32")]
    fn add_wheel_listener(&mut self, target: &EventTarget, input_resource: &mut InputResource) -> Result<(), String> {
        // 同じ入力リソースへの参照をJavaScriptのクロージャから安全に使用するため、グローバルに保存
        let resource_ptr: *mut InputResource = input_resource;
        
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
        if let Err(e) = target.add_event_listener_with_callback(
            "wheel",
            closure.as_ref().unchecked_ref()
        ) {
            return Err(format!("ホイールリスナーの追加に失敗: {:?}", e));
        }
        
        // クロージャを保存して参照をキープ
        let index = register_closure(closure);
        self.cleanup_indices.push(index);
        
        Ok(())
    }
    
    /// ネイティブ環境用の空の実装
    #[cfg(not(target_arch = "wasm32"))]
    fn add_wheel_listener(&mut self, _target: &(), _input_resource: &mut InputResource) -> Result<(), String> {
        Ok(())
    }
    
    /// タッチスタートイベントリスナーを追加
    #[cfg(target_arch = "wasm32")]
    fn add_touchstart_listener(&mut self, target: &EventTarget, input_resource: &mut InputResource) -> Result<(), String> {
        // 同じ入力リソースへの参照をJavaScriptのクロージャから安全に使用するため、グローバルに保存
        let resource_ptr: *mut InputResource = input_resource;
        
        let closure = Closure::wrap(Box::new(move |event: Event| {
            // 安全でない参照を使用して入力リソースを取得（WASMでは単一スレッドなので安全）
            let input_resource = unsafe { &mut *resource_ptr };
            
            if let Some(_) = event.dyn_ref::<TouchEvent>() {
                input_resource.handle_touch_start(&event);
            }
        }) as Box<dyn FnMut(Event)>);
        
        // イベントリスナーを追加
        if let Err(e) = target.add_event_listener_with_callback(
            "touchstart",
            closure.as_ref().unchecked_ref()
        ) {
            return Err(format!("タッチスタートリスナーの追加に失敗: {:?}", e));
        }
        
        // クロージャを保存して参照をキープ
        let index = register_closure(closure);
        self.cleanup_indices.push(index);
        
        Ok(())
    }
    
    /// ネイティブ環境用の空の実装
    #[cfg(not(target_arch = "wasm32"))]
    fn add_touchstart_listener(&mut self, _target: &(), _input_resource: &mut InputResource) -> Result<(), String> {
        Ok(())
    }
    
    /// タッチ移動イベントリスナーを追加
    #[cfg(target_arch = "wasm32")]
    fn add_touchmove_listener(&mut self, target: &EventTarget, input_resource: &mut InputResource) -> Result<(), String> {
        // 同じ入力リソースへの参照をJavaScriptのクロージャから安全に使用するため、グローバルに保存
        let resource_ptr: *mut InputResource = input_resource;
        
        let closure = Closure::wrap(Box::new(move |event: Event| {
            // 安全でない参照を使用して入力リソースを取得（WASMでは単一スレッドなので安全）
            let input_resource = unsafe { &mut *resource_ptr };
            
            if let Some(_) = event.dyn_ref::<TouchEvent>() {
                input_resource.handle_touch_move(&event);
            }
        }) as Box<dyn FnMut(Event)>);
        
        // イベントリスナーを追加
        if let Err(e) = target.add_event_listener_with_callback(
            "touchmove",
            closure.as_ref().unchecked_ref()
        ) {
            return Err(format!("タッチ移動リスナーの追加に失敗: {:?}", e));
        }
        
        // クロージャを保存して参照をキープ
        let index = register_closure(closure);
        self.cleanup_indices.push(index);
        
        Ok(())
    }
    
    /// ネイティブ環境用の空の実装
    #[cfg(not(target_arch = "wasm32"))]
    fn add_touchmove_listener(&mut self, _target: &(), _input_resource: &mut InputResource) -> Result<(), String> {
        Ok(())
    }
    
    /// タッチエンドイベントリスナーを追加
    #[cfg(target_arch = "wasm32")]
    fn add_touchend_listener(&mut self, target: &EventTarget, input_resource: &mut InputResource) -> Result<(), String> {
        // 同じ入力リソースへの参照をJavaScriptのクロージャから安全に使用するため、グローバルに保存
        let resource_ptr: *mut InputResource = input_resource;
        
        let closure = Closure::wrap(Box::new(move |event: Event| {
            // 安全でない参照を使用して入力リソースを取得（WASMでは単一スレッドなので安全）
            let input_resource = unsafe { &mut *resource_ptr };
            
            if let Some(_) = event.dyn_ref::<TouchEvent>() {
                input_resource.handle_touch_end(&event);
            }
        }) as Box<dyn FnMut(Event)>);
        
        // イベントリスナーを追加
        if let Err(e) = target.add_event_listener_with_callback(
            "touchend",
            closure.as_ref().unchecked_ref()
        ) {
            return Err(format!("タッチエンドリスナーの追加に失敗: {:?}", e));
        }
        
        // クロージャを保存して参照をキープ
        let index = register_closure(closure);
        self.cleanup_indices.push(index);
        
        Ok(())
    }
    
    /// ネイティブ環境用の空の実装
    #[cfg(not(target_arch = "wasm32"))]
    fn add_touchend_listener(&mut self, _target: &(), _input_resource: &mut InputResource) -> Result<(), String> {
        Ok(())
    }
    
    /// イベントリスナーのクリーンアップ処理
    fn cleanup_event_listeners(&mut self) -> Result<(), String> {
        #[cfg(target_arch = "wasm32")]
        {
            // 明示的なクロージャの解放はJavaScriptのガベージコレクタに任せる
            // インデックスのクリアのみを行う
            self.cleanup_indices.clear();
            
            // 注意: 実際のDOM側のイベントリスナーを削除するには、
            // removeEventListener() を呼び出す必要があるが、
            // WASM環境ではページ遷移時に自動的にクリーンアップされる
        }
        
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
                        #[cfg(target_arch = "wasm32")]
                        web_sys::console::error_1(&format!("イベントリスナーの設定に失敗: {}", e).into());
                        #[cfg(not(target_arch = "wasm32"))]
                        println!("イベントリスナーの設定に失敗: {}", e);
                        return SystemResult::Error;
                    }
                }
            } else {
                #[cfg(target_arch = "wasm32")]
                web_sys::console::error_1(&"InputResourceが見つかりません".into());
                #[cfg(not(target_arch = "wasm32"))]
                println!("InputResourceが見つかりません");
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
                    #[cfg(target_arch = "wasm32")]
                    web_sys::console::error_1(&format!("イベントリスナーの設定に失敗: {}", e).into());
                    #[cfg(not(target_arch = "wasm32"))]
                    println!("イベントリスナーの設定に失敗: {}", e);
                    SystemResult::Error
                }
            }
        } else {
            #[cfg(target_arch = "wasm32")]
            web_sys::console::error_1(&"InputResourceが見つかりません".into());
            #[cfg(not(target_arch = "wasm32"))]
            println!("InputResourceが見つかりません");
            SystemResult::Error
        }
    }
    
    fn cleanup(&mut self, _entity_manager: &mut EntityManager, _resources: &mut ResourceManager) -> SystemResult {
        match self.cleanup_event_listeners() {
            Ok(_) => SystemResult::Ok,
            Err(e) => {
                #[cfg(target_arch = "wasm32")]
                web_sys::console::error_1(&format!("イベントリスナーのクリーンアップに失敗: {}", e).into());
                #[cfg(not(target_arch = "wasm32"))]
                println!("イベントリスナーのクリーンアップに失敗: {}", e);
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
    
    // モックテスト用の実装（WebシステムのテストはWASMテスト環境が必要）
    #[cfg(test)]
    fn test_mock_keydown(&mut self, key_code: &str) -> SystemResult {
        // モックキーダウンイベントをシミュレート
        SystemResult::Ok
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