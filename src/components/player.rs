/**
 * プレイヤーコンポーネント
 * 
 * プレイヤーのデータと機能を提供するコンポーネント
 */

use js_sys;
use std::any::Any;
use crate::components::component_trait::Component;

/// プレイヤーのデータを格納するコンポーネント
#[derive(Debug, Clone)]
pub struct Player {
    /// プレイヤーID（一意の識別子）
    pub id: String,
    /// プレイヤーの名前
    pub name: String,
    /// プレイヤーの色（カーソル表示用）
    pub color: String,
    /// 最後にアクションを実行した時間（ミリ秒）
    pub last_action_time: f64,
    /// ローカルプレイヤーかどうか
    pub is_local: bool,
}

impl Player {
    /// 新しいプレイヤーコンポーネントを作成
    pub fn new(id: String, name: String, color: String, is_local: bool) -> Self {
        Self {
            id,
            name,
            color,
            last_action_time: js_sys::Date::now(),
            is_local,
        }
    }
    
    /// ローカルプレイヤーを作成するショートカット
    pub fn local(id: String, name: String) -> Self {
        Self::new(id, name, "#00FF00".to_string(), true)
    }
    
    /// リモートプレイヤーを作成するショートカット
    pub fn remote(id: String, name: String, color: String) -> Self {
        Self::new(id, name, color, false)
    }
    
    /// アクションタイムスタンプを更新
    pub fn update_action_time(&mut self) {
        self.last_action_time = js_sys::Date::now();
    }
}

impl Component for Player {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn clone_box(&self) -> Box<dyn Component> {
        Box::new(self.clone())
    }
} 