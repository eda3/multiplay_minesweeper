/**
 * プレイヤー状態リソース
 * 
 * 現在のプレイヤーの状態を管理する
 */
use std::any::Any;
use super::resource_trait::Resource;
use crate::resources::MouseState;

/// プレイヤーの状態を表すリソース
///
/// プレイヤーのID、名前、スコア、マルチプレイヤーステータスなどを管理します。
/// 以前のローカルプレイヤーと他プレイヤー情報の管理を置き換えるものです。
#[derive(Debug, Clone)]
pub struct PlayerStateResource {
    /// プレイヤーID
    pub player_id: String,
    /// プレイヤー名
    pub player_name: String,
    /// スコア
    pub score: u32,
    /// マルチプレイヤーモードかどうか
    pub is_multiplayer: bool,
    /// ホストかどうか
    pub is_host: bool,
    /// 参加済みかどうか
    pub has_joined: bool,
    /// マウスの状態（互換性のため）
    pub mouse_state: MouseState,
    /// 他のプレイヤー情報（互換性のため）
    pub other_players: Vec<Player>,
}

/// プレイヤー情報
#[derive(Debug, Clone)]
pub struct Player {
    /// プレイヤーID
    pub id: String,
    /// プレイヤーのX座標
    pub x: f64,
    /// プレイヤーのY座標
    pub y: f64,
    /// プレイヤーの色
    pub color: String,
}

impl Default for PlayerStateResource {
    fn default() -> Self {
        Self {
            player_id: "local".to_string(),
            player_name: "Player".to_string(),
            score: 0,
            is_multiplayer: false,
            is_host: false,
            has_joined: false,
            mouse_state: MouseState::default(),
            other_players: Vec::new(),
        }
    }
}

impl PlayerStateResource {
    /// 新しいプレイヤー状態を作成
    pub fn new() -> Self {
        Self::default()
    }
    
    /// プレイヤーIDを設定
    pub fn set_player_id(&mut self, id: String) {
        self.player_id = id;
    }
    
    /// プレイヤー名を設定
    pub fn set_player_name(&mut self, name: String) {
        self.player_name = name;
    }
    
    /// スコアを追加
    pub fn add_score(&mut self, points: u32) {
        self.score += points;
    }
    
    /// スコアをリセット
    pub fn reset_score(&mut self) {
        self.score = 0;
    }
    
    /// マルチプレイヤーモードを設定
    pub fn set_multiplayer(&mut self, is_multiplayer: bool) {
        self.is_multiplayer = is_multiplayer;
    }
    
    /// ホスト状態を設定
    pub fn set_host(&mut self, is_host: bool) {
        self.is_host = is_host;
    }
    
    /// 参加状態を設定
    pub fn set_joined(&mut self, has_joined: bool) {
        self.has_joined = has_joined;
    }
    
    // 以下は互換性のためのメソッド
    
    /// ローカルプレイヤーIDを設定（互換性のため）
    pub fn set_local_player_id(&mut self, id: String) {
        self.player_id = id;
    }
    
    /// プレイヤーが存在するかチェック（互換性のため）
    pub fn has_player(&self, id: &str) -> bool {
        if id == &self.player_id {
            return true;
        }
        self.other_players.iter().any(|p| p.id == id)
    }
    
    /// プレイヤーを追加（互換性のため）
    pub fn add_player(&mut self, id: String, x: f64, y: f64, color: String) {
        // 自分自身のIDなら自身を更新
        if id == self.player_id {
            return;
        }
        
        // 既に存在するなら更新
        for player in &mut self.other_players {
            if player.id == id {
                player.x = x;
                player.y = y;
                player.color = color;
                return;
            }
        }
        
        // 新規追加
        self.other_players.push(Player {
            id,
            x,
            y,
            color,
        });
    }
    
    /// プレイヤーを削除（互換性のため）
    pub fn remove_player(&mut self, id: &str) {
        self.other_players.retain(|p| p.id != id);
    }
    
    /// プレイヤーの位置を更新（互換性のため）
    pub fn update_player_position(&mut self, id: &str, x: f64, y: f64) {
        if id == "local" || id == &self.player_id {
            // ローカルプレイヤーの場合はマウス位置も更新
            self.mouse_state.update_position(x, y);
            return;
        }
        
        // 他プレイヤーの場合
        for player in &mut self.other_players {
            if player.id == id {
                player.x = x;
                player.y = y;
                return;
            }
        }
    }
    
    /// マウスの状態を設定（互換性のため）
    pub fn set_mouse_state(&mut self, state: &str) {
        if state == MouseState::LeftDown {
            self.mouse_state.update_buttons(true, false, false);
        } else if state == MouseState::RightDown {
            self.mouse_state.update_buttons(false, true, false);
        } else if state == MouseState::MiddleDown {
            self.mouse_state.update_buttons(false, false, true);
        } else {
            // Up状態の場合はすべてのボタンを解放
            self.mouse_state.update_buttons(false, false, false);
        }
    }
    
    /// すべてのプレイヤーを取得（互換性のため）
    pub fn all_players(&self) -> Vec<&Player> {
        self.other_players.iter().collect()
    }
    
    /// ローカルプレイヤーを取得（互換性のため）
    pub fn local_player(&self) -> Option<Player> {
        Some(Player {
            id: self.player_id.clone(),
            x: self.mouse_state.x,
            y: self.mouse_state.y,
            color: "red".to_string(),
        })
    }
} 