/**
 * プレイヤー状態リソース
 * 
 * プレイヤーの状態（位置、クリック状態など）を管理するリソース
 */
use super::resource_trait::Resource;
use crate::resources::mouse_state::MouseState;

/// プレイヤーの状態を表すリソース
///
/// プレイヤーのID、名前、スコア、マルチプレイヤーステータスなどを管理します。
/// 以前のローカルプレイヤーと他プレイヤー情報の管理を置き換えるものです。
#[derive(Debug, Clone)]
pub struct PlayerStateResource {
    /// プレイヤー名
    pub name: String,
    /// プレイヤーID（ネットワークプレイ用）
    pub id: Option<String>,
    /// マウス状態
    pub mouse_state: MouseState,
    /// プレイヤーの色
    pub color: String,
    /// 最後にクリックした位置（行、列）
    pub last_click: Option<(usize, usize)>,
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
            name: "Player".to_string(),
            id: None,
            mouse_state: MouseState::default(),
            color: "#3498db".to_string(),
            last_click: None,
        }
    }
}

impl PlayerStateResource {
    /// 新しいプレイヤー状態を作成
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            id: None,
            mouse_state: MouseState::default(),
            color: "#3498db".to_string(),
            last_click: None,
        }
    }
    
    /// マウス位置を更新
    pub fn update_mouse_position(&mut self, x: f64, y: f64) {
        self.mouse_state.x = x;
        self.mouse_state.y = y;
    }
    
    /// マウスボタンの状態を更新
    pub fn update_mouse_button(&mut self, button: u16, pressed: bool) {
        match button {
            0 => self.mouse_state.left_button = pressed,
            2 => self.mouse_state.right_button = pressed,
            _ => {}
        }
    }
    
    /// クリック位置を記録
    pub fn set_last_click(&mut self, row: usize, col: usize) {
        self.last_click = Some((row, col));
    }
    
    /// ネットワークIDを設定
    pub fn set_network_id(&mut self, id: String) {
        self.id = Some(id);
    }
    
    /// プレイヤーIDを設定
    pub fn set_player_id(&mut self, id: String) {
        self.id = Some(id);
    }
    
    /// プレイヤー名を設定
    pub fn set_player_name(&mut self, name: String) {
        self.name = name;
    }
    
    /// マルチプレイヤーモードを設定
    pub fn set_multiplayer(&mut self, is_multiplayer: bool) {
        // このリソースはマルチプレイヤーを考慮していないため、このメソッドは実装されていません。
    }
    
    /// ホスト状態を設定
    pub fn set_host(&mut self, is_host: bool) {
        // このリソースはマルチプレイヤーを考慮していないため、このメソッドは実装されていません。
    }
    
    /// 参加状態を設定
    pub fn set_joined(&mut self, has_joined: bool) {
        // このリソースはマルチプレイヤーを考慮していないため、このメソッドは実装されていません。
    }
    
    // 以下は互換性のためのメソッド
    
    /// ローカルプレイヤーIDを設定（互換性のため）
    pub fn set_local_player_id(&mut self, id: String) {
        self.id = Some(id);
    }
    
    /// プレイヤーが存在するかチェック（互換性のため）
    pub fn has_player(&self, id: &str) -> bool {
        self.id.as_ref().map_or(false, |i| i == id)
    }
    
    /// プレイヤーを追加（互換性のため）
    pub fn add_player(&mut self, id: String, x: f64, y: f64, color: String) {
        // 自分自身のIDなら自身を更新
        if id == *self.id.as_ref().unwrap_or(&"local".to_string()) {
            return;
        }
        
        // 既に存在するなら更新
        self.id = Some(id);
        self.mouse_state.x = x;
        self.mouse_state.y = y;
        self.color = color;
    }
    
    /// プレイヤーを削除（互換性のため）
    pub fn remove_player(&mut self) {
        self.id = None;
    }
    
    /// プレイヤーの位置を更新（互換性のため）
    pub fn update_player_position(&mut self, x: f64, y: f64) {
        self.mouse_state.x = x;
        self.mouse_state.y = y;
    }
    
    /// マウスの状態を設定（互換性のため）
    pub fn set_mouse_state(&mut self, state: &str) {
        if state == MouseState::LeftDown {
            self.mouse_state.left_button = true;
        } else if state == MouseState::RightDown {
            self.mouse_state.right_button = true;
        } else if state == MouseState::MiddleDown {
            self.mouse_state.middle_button = true;
        } else {
            // Up状態の場合はすべてのボタンを解放
            self.mouse_state.left_button = false;
            self.mouse_state.right_button = false;
            self.mouse_state.middle_button = false;
        }
    }
    
    /// すべてのプレイヤーを取得（互換性のため）
    pub fn all_players(&self) -> Vec<Player> {
        // Playerインスタンスを作成して所有権ごと返す
                let player = Player {
            id: self.id.as_ref().unwrap_or(&"local".to_string()).clone(),
            x: self.mouse_state.x,
            y: self.mouse_state.y,
            color: self.color.clone(),
        };
        vec![player]
    }
    
    /// ローカルプレイヤーを取得（互換性のため）
    pub fn local_player(&self) -> Option<Player> {
        Some(Player {
            id: self.id.as_ref().unwrap_or(&"local".to_string()).clone(),
            x: self.mouse_state.x,
            y: self.mouse_state.y,
            color: self.color.clone(),
        })
    }
} 