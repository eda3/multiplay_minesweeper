/**
 * プレイヤー関連のコンポーネント
 * 
 * マルチプレイヤーゲームにおけるプレイヤー情報を表すデータ構造
 */

/// プレイヤーのデータを格納するコンポーネント
#[derive(Debug, Clone)]
pub struct PlayerComponent {
    /// プレイヤーID（一意の識別子）
    pub id: String,
    /// プレイヤーの色（カーソル表示用）
    pub color: String,
    /// 最後の操作時間（ミリ秒）
    pub last_action_time: f64,
    /// ローカルプレイヤーかどうか
    pub is_local: bool,
}

impl PlayerComponent {
    /// 新しいプレイヤーコンポーネントを作成
    pub fn new(id: String, color: String, is_local: bool) -> Self {
        Self {
            id,
            color,
            last_action_time: js_sys::Date::now(),
            is_local,
        }
    }
    
    /// ローカルプレイヤーを作成するショートカット
    pub fn local(id: String) -> Self {
        Self::new(id, "#00FF00".to_string(), true)
    }
    
    /// リモートプレイヤーを作成するショートカット
    pub fn remote(id: String, color: String) -> Self {
        Self::new(id, color, false)
    }
    
    /// 操作時間を更新
    pub fn update_action_time(&mut self) {
        self.last_action_time = js_sys::Date::now();
    }
} 