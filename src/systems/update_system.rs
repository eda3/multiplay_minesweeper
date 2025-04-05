/**
 * 更新システム
 * 
 * ゲームの状態を更新するシステム
 */
use std::rc::Rc;
use std::cell::RefCell;
use std::any::Any;

use crate::resources::TimeResource;
use crate::resources::InputResource;
use crate::resources::{GameStateResource, GamePhase};

/// 更新システム関数
pub fn update_system(resources: &[Rc<RefCell<dyn Any>>]) {
    // TimeResourceのRcを探す
    let time_rc_option = resources.iter()
        .find(|r| r.borrow().is::<TimeResource>());
    
    // InputResourceのRcを探す
    let input_rc_option = resources.iter()
        .find(|r| r.borrow().is::<InputResource>());
    
    // GameStateResourceのRcを探す
    let game_rc_option = resources.iter()
        .find(|r| r.borrow().is::<GameStateResource>());
    
    // 必要なリソースが見つからない場合は何もしない
    if time_rc_option.is_none() || input_rc_option.is_none() || game_rc_option.is_none() {
        return;
    }
    
    // 借用して処理
    {
        let mut time = time_rc_option.unwrap().borrow_mut();
        let time = time.downcast_mut::<TimeResource>().unwrap();
        
        let mut input = input_rc_option.unwrap().borrow_mut();
        let input = input.downcast_mut::<InputResource>().unwrap();
        
        let mut game = game_rc_option.unwrap().borrow_mut();
        let game = game.downcast_mut::<GameStateResource>().unwrap();
        
        // タイムリソースの更新
        time.update();
        
        // 入力リソースの更新
        input.update();
        
        // ゲームフェーズに応じた更新処理
        match game.phase {
            GamePhase::Playing => {
                // プレイ中の時間更新
                game.update_elapsed_time();
            },
            GamePhase::Paused => {
                // 一時停止中の処理
                // 特に何もしない
            },
            GamePhase::GameOver { .. } => {
                // ゲームオーバー時の処理
                // 特に何もしない
            },
            _ => {}
        }
    }
}

/// ゲームティック関数 - 一定間隔で呼び出される定期更新
pub fn game_tick_system(resources: &[Rc<RefCell<dyn Any>>]) {
    // TimeResourceのRcを探す
    let time_rc_option = resources.iter()
        .find(|r| r.borrow().is::<TimeResource>());
    
    // GameStateResourceのRcを探す
    let game_rc_option = resources.iter()
        .find(|r| r.borrow().is::<GameStateResource>());
    
    // 必要なリソースが見つからない場合は何もしない
    if time_rc_option.is_none() || game_rc_option.is_none() {
        return;
    }
    
    // 借用して処理
    {
        let time = time_rc_option.unwrap().borrow();
        let time = time.downcast_ref::<TimeResource>().unwrap();
        
        let mut game = game_rc_option.unwrap().borrow_mut();
        let game = game.downcast_mut::<GameStateResource>().unwrap();
        
        // ゲームがプレイ中の場合のみ処理
        if game.is_playing() {
            // 定期的な更新処理（例：1秒ごとのスコア更新など）
            let tick_interval = 1000.0; // 1秒
            let elapsed = time.total_time * 1000.0; // ミリ秒に変換
            
            // 現在の時間を1000で割った余りが前フレームより小さい場合、1秒経過
            if (elapsed % tick_interval) < time.get_delta_time() * 1000.0 {
                // 1秒ごとのゲーム内処理
                // 例：タイムアタックモードのスコア減少など
            }
        }
    }
} 