/**
 * プレイヤー状態リソース
 * 
 * ローカルプレイヤーと他プレイヤーの情報を管理するリソース
 */
use std::collections::HashMap;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use js_sys::{Date, JSON, Object, Reflect};
use serde::{Serialize, Deserialize};
use wasm_bindgen::JsValue;
use crate::models::Player as GamePlayer;

/// マウスの状態
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseState {
    /// マウスが押されていない
    Up,
    /// 左ボタンが押されている
    LeftDown,
    /// 右ボタンが押されている
    RightDown,
}

/// ECSリソースとしてのプレイヤー
#[derive(Debug, Clone)]
pub struct Player {
    /// プレイヤーID
    pub id: String,
    /// X座標
    pub x: f64,
    /// Y座標
    pub y: f64,
    /// プレイヤーのカラー（CSS形式）
    pub color: String,
    /// アクティブかどうか
    pub active: bool,
    /// 最終更新時刻
    pub last_update: f64,
}

/// プレイヤー状態リソース
/// マルチプレイヤーゲームにおけるプレイヤーの状態を管理する
#[derive(Debug)]
pub struct PlayerStateResource {
    /// ローカルプレイヤーID
    pub local_player_id: Option<String>,
    /// 全プレイヤーのマップ
    players: HashMap<String, Player>,
    /// マウスX座標
    pub mouse_x: f64,
    /// マウスY座標
    pub mouse_y: f64,
    /// マウスの状態
    pub mouse_state: MouseState,
    /// 最後の位置更新時間
    pub last_position_update: f64,
    /// 最後に押されたキー
    pub last_key_pressed: Option<String>,
    /// アクティブなプレイヤー数
    pub active_player_count: usize,
}

impl Default for PlayerStateResource {
    fn default() -> Self {
        Self::new()
    }
}

impl PlayerStateResource {
    /// 新しいPlayerStateResourceを作成
    pub fn new() -> Self {
        Self {
            local_player_id: None,
            players: HashMap::new(),
            mouse_x: 0.0,
            mouse_y: 0.0,
            mouse_state: MouseState::Up,
            last_position_update: 0.0,
            last_key_pressed: None,
            active_player_count: 0,
        }
    }

    /// ローカルプレイヤーIDを設定
    pub fn set_local_player_id(&mut self, id: String) {
        self.local_player_id = Some(id);
    }

    /// ローカルプレイヤーを取得
    pub fn local_player(&self) -> Option<&Player> {
        if let Some(id) = &self.local_player_id {
            self.players.get(id)
        } else {
            None
        }
    }

    /// ローカルプレイヤーを可変で取得
    pub fn local_player_mut(&mut self) -> Option<&mut Player> {
        if let Some(id) = self.local_player_id.clone() {
            self.players.get_mut(&id)
        } else {
            None
        }
    }

    /// プレイヤーを追加
    pub fn add_player(&mut self, id: String, x: f64, y: f64, color: String) -> &Player {
        let player = Player {
            id: id.clone(),
            x,
            y,
            color,
            active: true,
            last_update: Date::now(),
        };

        self.players.insert(id.clone(), player);
        self.update_active_count();
        self.players.get(&id).unwrap()
    }

    /// プレイヤーを削除
    pub fn remove_player(&mut self, id: &str) {
        self.players.remove(id);
        self.update_active_count();
    }

    /// プレイヤーが存在するかチェック
    pub fn has_player(&self, id: &str) -> bool {
        self.players.contains_key(id)
    }

    /// 全プレイヤーを取得
    pub fn all_players(&self) -> &HashMap<String, Player> {
        &self.players
    }

    /// アクティブなプレイヤー数を更新
    fn update_active_count(&mut self) {
        self.active_player_count = self.players.values().filter(|p| p.active).count();
    }

    /// プレイヤーの位置を更新
    pub fn update_player_position(&mut self, id: &str, x: f64, y: f64) {
        if let Some(player) = self.players.get_mut(id) {
            player.x = x;
            player.y = y;
            player.last_update = Date::now();
        }
    }

    /// ローカルプレイヤーの位置を更新
    pub fn update_local_player_position(&mut self, x: f64, y: f64) {
        if let Some(id) = self.local_player_id.clone() {
            self.update_player_position(&id, x, y);
        }
    }

    /// マウス状態を更新
    pub fn set_mouse_state(&mut self, state: MouseState) {
        self.mouse_state = state;
    }

    /// JSONからプレイヤーを追加
    pub fn add_players_from_json(&mut self, json: &JsValue) -> Result<(), JsValue> {
        let obj = js_sys::Object::from(json.clone());
        
        // プレイヤー情報を取得
        if let Some(players_val) = js_sys::Reflect::get(&obj, &"players".into()).ok() {
            // 文字列形式のJSON
            let players_str = players_val.as_string().ok_or_else(|| {
                JsValue::from_str("Players value is not a string")
            })?;
            
            let players: serde_json::Value = serde_json::from_str(&players_str).map_err(|e| {
                JsValue::from_str(&format!("Failed to parse players JSON: {}", e))
            })?;
            
            if let Some(players_obj) = players.as_object() {
                for (id, player) in players_obj {
                    if let (Some(x), Some(y), Some(color)) = (
                        player.get("x").and_then(|v| v.as_f64()),
                        player.get("y").and_then(|v| v.as_f64()),
                        player.get("color").and_then(|v| v.as_str()),
                    ) {
                        self.add_player(id.clone(), x, y, color.to_string());
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// プレイヤー情報をJSON形式で取得
    pub fn players_to_json(&self) -> String {
        let mut players = serde_json::Map::new();
        
        for (id, player) in &self.players {
            let mut player_obj = serde_json::Map::new();
            player_obj.insert("x".to_string(), serde_json::Value::from(player.x));
            player_obj.insert("y".to_string(), serde_json::Value::from(player.y));
            player_obj.insert("color".to_string(), serde_json::Value::from(player.color.clone()));
            player_obj.insert("active".to_string(), serde_json::Value::from(player.active));
            
            players.insert(id.clone(), serde_json::Value::Object(player_obj));
        }
        
        serde_json::Value::Object(players).to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_add_and_remove_player() {
        let mut player_state = PlayerStateResource::new();
        
        // プレイヤーを追加
        player_state.add_player("p1".to_string(), 10.0, 20.0, "#ff0000".to_string());
        player_state.add_player("p2".to_string(), 30.0, 40.0, "#00ff00".to_string());
        
        // プレイヤー数の確認
        assert_eq!(player_state.active_player_count, 2);
        
        // プレイヤーの存在確認
        assert!(player_state.has_player("p1"));
        assert!(player_state.has_player("p2"));
        
        // プレイヤーを削除
        player_state.remove_player("p1");
        
        // 削除後の確認
        assert_eq!(player_state.active_player_count, 1);
        assert!(!player_state.has_player("p1"));
        assert!(player_state.has_player("p2"));
    }
    
    #[test]
    fn test_update_player_position() {
        let mut player_state = PlayerStateResource::new();
        
        // プレイヤーを追加
        player_state.add_player("p1".to_string(), 10.0, 20.0, "#ff0000".to_string());
        
        // 位置を更新
        player_state.update_player_position("p1", 15.0, 25.0);
        
        // 更新後の位置を確認
        let player = player_state.players.get("p1").unwrap();
        assert_eq!(player.x, 15.0);
        assert_eq!(player.y, 25.0);
    }
    
    #[test]
    fn test_local_player() {
        let mut player_state = PlayerStateResource::new();
        
        // ローカルプレイヤーを設定
        player_state.add_player("local".to_string(), 10.0, 20.0, "#ff0000".to_string());
        player_state.set_local_player_id("local".to_string());
        
        // ローカルプレイヤーの取得
        let local = player_state.local_player().unwrap();
        assert_eq!(local.id, "local");
        
        // ローカルプレイヤーの位置を更新
        player_state.update_local_player_position(15.0, 25.0);
        
        // 更新後の位置を確認
        let local = player_state.local_player().unwrap();
        assert_eq!(local.x, 15.0);
        assert_eq!(local.y, 25.0);
    }
    
    #[test]
    fn test_mouse_state() {
        let mut player_state = PlayerStateResource::new();
        
        // 初期状態の確認
        assert_eq!(player_state.mouse_state, MouseState::Up);
        
        // 状態を変更
        player_state.set_mouse_state(MouseState::LeftDown);
        assert_eq!(player_state.mouse_state, MouseState::LeftDown);
        
        player_state.set_mouse_state(MouseState::RightDown);
        assert_eq!(player_state.mouse_state, MouseState::RightDown);
    }
} 