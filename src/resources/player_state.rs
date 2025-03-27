/**
 * プレイヤー状態リソース
 * 
 * ローカルプレイヤーと他プレイヤーの情報を管理するリソース
 */
use std::collections::HashMap;
use wasm_bindgen::JsValue;
use wasm_bindgen::JsCast;
use js_sys::{JSON, Object, Reflect};

use crate::models::Player;

/// プレイヤー状態リソース
#[derive(Debug, Clone)]
pub struct PlayerStateResource {
    /// ローカルプレイヤーID
    pub local_player_id: Option<String>,
    /// 全プレイヤーの情報
    pub players: HashMap<String, Player>,
    /// マウスX座標
    pub mouse_x: f64,
    /// マウスY座標
    pub mouse_y: f64,
    /// マウスボタン押下状態
    pub is_mouse_down: bool,
    /// 最後に位置情報を送信した時間
    pub last_position_update: f64,
    /// 最後に入力されたキー
    pub last_key: Option<String>,
    /// アクティブなプレイヤー数
    pub active_player_count: usize,
}

impl Default for PlayerStateResource {
    fn default() -> Self {
        Self {
            local_player_id: None,
            players: HashMap::new(),
            mouse_x: 0.0,
            mouse_y: 0.0,
            is_mouse_down: false,
            last_position_update: 0.0,
            last_key: None,
            active_player_count: 0,
        }
    }
}

impl PlayerStateResource {
    /// 新しいプレイヤー状態リソースを作成
    pub fn new() -> Self {
        Self::default()
    }
    
    /// ローカルプレイヤーのIDを設定
    pub fn set_local_player_id(&mut self, id: String) {
        self.local_player_id = Some(id);
    }
    
    /// ローカルプレイヤーの情報を取得
    pub fn get_local_player(&self) -> Option<&Player> {
        self.local_player_id.as_ref()
            .and_then(|id| self.players.get(id))
    }
    
    /// ローカルプレイヤーの情報を取得（可変）
    pub fn get_local_player_mut(&mut self) -> Option<&mut Player> {
        if let Some(id) = &self.local_player_id {
            self.players.get_mut(id)
        } else {
            None
        }
    }
    
    /// プレイヤーを追加
    pub fn add_player(&mut self, player: Player) {
        let id = player.id.clone();
        self.players.insert(id, player);
        self.update_player_count();
    }
    
    /// IDを指定してプレイヤーを追加
    pub fn add_player_with_id(&mut self, id: String, x: f64, y: f64, color: String) {
        let player = Player {
            id: id.clone(),
            name: format!("プレイヤー_{}", id),
            color,
            score: 0,
            x,
            y,
            is_local: self.local_player_id.as_ref().map_or(false, |local_id| *local_id == id),
            is_host: false,
            is_alive: true,
            cells_revealed: 0,
        };
        self.players.insert(id, player);
        self.update_player_count();
    }
    
    /// 複数のプレイヤーを追加
    pub fn add_players_from_json(&mut self, json: &JsValue) -> Result<(), JsValue> {
        let players_array = js_sys::Array::from(json);
        let len = players_array.length();
        for i in 0..len {
            if let Some(player_obj) = players_array.get(i).dyn_into::<Object>().ok() {
                let id = Reflect::get(&player_obj, &"id".into())?
                    .as_string()
                    .unwrap_or_default();
                
                let x = Reflect::get(&player_obj, &"x".into())?
                    .as_f64()
                    .unwrap_or(0.0);
                
                let y = Reflect::get(&player_obj, &"y".into())?
                    .as_f64()
                    .unwrap_or(0.0);
                
                let color = Reflect::get(&player_obj, &"color".into())?
                    .as_string()
                    .unwrap_or("#FF0000".to_string());
                
                self.add_player_with_id(id, x, y, color);
            }
        }
        
        self.update_player_count();
        Ok(())
    }
    
    /// プレイヤーを削除
    pub fn remove_player(&mut self, id: &str) {
        self.players.remove(id);
        self.update_player_count();
    }
    
    /// プレイヤーの位置を更新
    pub fn update_player_position(&mut self, id: &str, x: f64, y: f64) {
        if let Some(player) = self.players.get_mut(id) {
            player.x = x;
            player.y = y;
        }
    }
    
    /// ローカルプレイヤーの位置を更新
    pub fn update_local_position(&mut self, x: f64, y: f64) {
        self.mouse_x = x;
        self.mouse_y = y;
        
        if let Some(player) = self.get_local_player_mut() {
            player.x = x;
            player.y = y;
        }
    }
    
    /// マウスの状態を更新
    pub fn update_mouse_state(&mut self, x: f64, y: f64, is_down: bool) {
        self.mouse_x = x;
        self.mouse_y = y;
        self.is_mouse_down = is_down;
    }
    
    /// プレイヤー数を更新
    fn update_player_count(&mut self) {
        self.active_player_count = self.players.len();
    }
    
    /// プレイヤー情報をJSONに変換
    pub fn get_player_as_json(&self, id: &str) -> Option<JsValue> {
        self.players.get(id).map(|player| {
            let obj = Object::new();
            let _ = Reflect::set(&obj, &"id".into(), &player.id.clone().into());
            let _ = Reflect::set(&obj, &"x".into(), &player.x.into());
            let _ = Reflect::set(&obj, &"y".into(), &player.y.into());
            let _ = Reflect::set(&obj, &"color".into(), &player.color.clone().into());
            obj.into()
        })
    }

    pub fn players_from_json(&mut self, json: JsValue) -> Result<(), String> {
        self.players.clear();
        
        // JSONから配列を取得
        let players_array = js_sys::Array::from(&json);
        for i in 0..players_array.length() {
            if let Ok(player_json) = players_array.get(i).dyn_into::<js_sys::Object>() {
                // JsValueをPlayerに変換する（ここではシンプルな方法を使用）
                let id = js_sys::Reflect::get(&player_json, &"id".into())
                    .map_err(|e| format!("ID取得エラー: {:?}", e))?
                    .as_string()
                    .unwrap_or_default();
                    
                let name = js_sys::Reflect::get(&player_json, &"name".into())
                    .map_err(|e| format!("名前取得エラー: {:?}", e))?
                    .as_string()
                    .unwrap_or_else(|| format!("プレイヤー_{}", id));
                    
                let color = js_sys::Reflect::get(&player_json, &"color".into())
                    .map_err(|e| format!("色取得エラー: {:?}", e))?
                    .as_string()
                    .unwrap_or("#FF0000".to_string());
                    
                let score = js_sys::Reflect::get(&player_json, &"score".into())
                    .map(|v| v.as_f64().unwrap_or(0.0) as u32)
                    .unwrap_or(0);
                    
                let x = js_sys::Reflect::get(&player_json, &"x".into())
                    .map_err(|e| format!("X座標取得エラー: {:?}", e))?
                    .as_f64()
                    .unwrap_or(0.0);
                    
                let y = js_sys::Reflect::get(&player_json, &"y".into())
                    .map_err(|e| format!("Y座標取得エラー: {:?}", e))?
                    .as_f64()
                    .unwrap_or(0.0);
                    
                let is_local = js_sys::Reflect::get(&player_json, &"is_local".into())
                    .map(|v| v.as_bool().unwrap_or(false))
                    .unwrap_or(false);
                    
                let is_host = js_sys::Reflect::get(&player_json, &"is_host".into())
                    .map(|v| v.as_bool().unwrap_or(false))
                    .unwrap_or(false);
                    
                let is_alive = js_sys::Reflect::get(&player_json, &"is_alive".into())
                    .map(|v| v.as_bool().unwrap_or(true))
                    .unwrap_or(true);
                    
                let cells_revealed = js_sys::Reflect::get(&player_json, &"cells_revealed".into())
                    .map(|v| v.as_f64().unwrap_or(0.0) as usize)
                    .unwrap_or(0);
                    
                let player = Player {
                    id: id.clone(),
                    name,
                    color,
                    score,
                    x,
                    y,
                    is_local,
                    is_host,
                    is_alive,
                    cells_revealed,
                };
                
                self.players.insert(id, player);
            }
        }
        
        self.update_player_count();
        Ok(())
    }
} 