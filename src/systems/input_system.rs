/**
 * 入力システム
 * 
 * マウスとキーボードの入力を処理する
 */
use std::rc::Rc;
use std::cell::RefCell;
use std::any::Any;

use crate::resources::InputResource;
use crate::resources::BoardResource;
use crate::resources::{GameStateResource, GamePhase, DifficultyLevel};

/// 入力システム関数
pub fn input_system(resources: &[Rc<RefCell<dyn Any>>]) {
    // InputResourceを探す
    let input_rc_option = resources.iter()
        .find(|r| r.borrow().is::<InputResource>());
    
    if let Some(input_rc) = input_rc_option {
        let input = input_rc.borrow();
        let input = input.downcast_ref::<InputResource>().unwrap();
        
        // BoardResourceのRcを探す
        let board_rc_option = resources.iter()
            .find(|r| r.borrow().is::<BoardResource>());
        
        // GameStateResourceのRcを探す
        let game_rc_option = resources.iter()
            .find(|r| r.borrow().is::<GameStateResource>());
        
        // 必要なリソースが見つからない場合は何もしない
        if board_rc_option.is_none() || game_rc_option.is_none() {
            return;
        }
        
        let board_rc = board_rc_option.unwrap();
        let game_rc = game_rc_option.unwrap();
        
        // 借用して処理
        {
            let mut board = board_rc.borrow_mut();
            let board = board.downcast_mut::<BoardResource>().unwrap();
            
            let mut game = game_rc.borrow_mut();
            let game = game.downcast_mut::<GameStateResource>().unwrap();
            
            // ゲームフェーズに応じた入力処理
            match game.phase {
                GamePhase::StartScreen => process_start_screen_input(input, game),
                GamePhase::Playing => process_playing_input(input, board, game),
                GamePhase::Paused => process_paused_input(input, game),
                GamePhase::GameOver { .. } => process_game_over_input(input, game, board),
                _ => {}
            }
        }
    }
}

/// スタート画面での入力処理
fn process_start_screen_input(input: &InputResource, game: &mut GameStateResource) {
    // スペースキーでゲーム開始
    if input.is_key_pressed("Space") {
        game.start_game();
    }
    
    // 難易度選択
    if input.is_key_pressed("Digit1") {
        game.set_difficulty(DifficultyLevel::Beginner);
    } else if input.is_key_pressed("Digit2") {
        game.set_difficulty(DifficultyLevel::Intermediate);
    } else if input.is_key_pressed("Digit3") {
        game.set_difficulty(DifficultyLevel::Expert);
    }
}

/// ゲームプレイ中の入力処理
fn process_playing_input(input: &InputResource, board: &mut BoardResource, game: &mut GameStateResource) {
    // ESCキーでゲーム一時停止
    if input.is_key_pressed("Escape") {
        game.pause_game();
        return;
    }
    
    // マウス入力処理
    if input.is_mouse_pressed(0) { // 左クリック
        let (mouse_x, mouse_y) = input.get_mouse_position();
        if let Some(cell_index) = board.get_cell_index(mouse_x, mouse_y) {
            // セルを開く
            let exploded = board.reveal_cell(cell_index);
            
            // 爆発した場合はゲームオーバー
            if exploded {
                board.reveal_all_mines();
                game.set_game_over(false);
                return;
            }
            
            // 勝利条件チェック
            if board.check_win_condition() {
                board.reveal_all_mines();
                game.add_score((board.config.width * board.config.height) as u32);
                game.set_game_over(true);
            }
        }
    } else if input.is_mouse_pressed(2) { // 右クリック
        let (mouse_x, mouse_y) = input.get_mouse_position();
        if let Some(cell_index) = board.get_cell_index(mouse_x, mouse_y) {
            // フラグを切り替える
            board.toggle_flag(cell_index);
        }
    }
}

/// 一時停止中の入力処理
fn process_paused_input(input: &InputResource, game: &mut GameStateResource) {
    // ESCキーまたはスペースキーでゲーム再開
    if input.is_key_pressed("Escape") || input.is_key_pressed("Space") {
        game.resume_game();
    }
    
    // Rキーでゲームをリセット
    if input.is_key_pressed("KeyR") {
        game.start_game();
    }
}

/// ゲームオーバー時の入力処理
fn process_game_over_input(input: &InputResource, game: &mut GameStateResource, board: &mut BoardResource) {
    // スペースキーまたはRキーで新しいゲーム
    if input.is_key_pressed("Space") || input.is_key_pressed("KeyR") {
        game.start_game();
        board.reset();
    }
    
    // ESCキーでスタート画面に戻る
    if input.is_key_pressed("Escape") {
        game.phase = GamePhase::StartScreen;
    }
} 