use wasm_multiplayer::resources::{CoreGameResource, GamePhase};

#[test]
fn test_new_resource_has_default_values() {
    let resource = CoreGameResource::new();
    
    assert_eq!(resource.phase, GamePhase::Setup);
    assert_eq!(resource.score, 0);
    assert_eq!(resource.elapsed_time, 0.0);
    assert_eq!(resource.remaining_mines, 0);
    assert!(!resource.win);
}

#[test]
fn test_initialize_sets_correct_values() {
    let mut resource = CoreGameResource::new();
    resource.initialize(50);
    
    assert_eq!(resource.phase, GamePhase::Setup);
    assert_eq!(resource.score, 0);
    assert_eq!(resource.elapsed_time, 0.0);
    assert_eq!(resource.remaining_mines, 50);
}

#[test]
fn test_start_game_changes_phase() {
    let mut resource = CoreGameResource::new();
    resource.start_game();
    
    assert_eq!(resource.phase, GamePhase::Playing);
}

#[test]
fn test_pause_game_works() {
    let mut resource = CoreGameResource::new();
    resource.start_game();
    resource.pause_game();
    
    assert_eq!(resource.phase, GamePhase::Paused);
}

#[test]
fn test_resume_game_works() {
    let mut resource = CoreGameResource::new();
    resource.start_game();
    resource.pause_game();
    resource.resume_game();
    
    assert_eq!(resource.phase, GamePhase::Playing);
}

#[test]
fn test_end_game_sets_win_status() {
    let mut resource = CoreGameResource::new();
    resource.start_game();
    
    // 勝利パターン
    resource.end_game(true);
    assert_eq!(resource.phase, GamePhase::GameOver);
    assert!(resource.win);
    
    // 敗北パターン
    let mut resource = CoreGameResource::new();
    resource.start_game();
    resource.end_game(false);
    assert_eq!(resource.phase, GamePhase::GameOver);
    assert!(!resource.win);
}

#[test]
fn test_is_playing_returns_correct_value() {
    let mut resource = CoreGameResource::new();
    assert!(!resource.is_playing());
    
    resource.start_game();
    assert!(resource.is_playing());
    
    resource.pause_game();
    assert!(!resource.is_playing());
    
    resource.resume_game();
    assert!(resource.is_playing());
    
    resource.end_game(true);
    assert!(!resource.is_playing());
}

#[test]
fn test_update_elapsed_time_increases_time() {
    let mut resource = CoreGameResource::new();
    resource.start_game();
    
    resource.update_elapsed_time(16.7); // 16.7ミリ秒経過
    assert!(resource.elapsed_time > 0.0);
    
    let previous_time = resource.elapsed_time;
    resource.update_elapsed_time(33.3); // さらに33.3ミリ秒経過
    assert!(resource.elapsed_time > previous_time);
}

#[test]
fn test_add_score_updates_score() {
    let mut resource = CoreGameResource::new();
    
    resource.add_score(100);
    assert_eq!(resource.score, 100);
    
    resource.add_score(50);
    assert_eq!(resource.score, 150);
}

#[test]
fn test_remaining_mines_updates_correctly() {
    let mut resource = CoreGameResource::new();
    resource.initialize(40);
    
    assert_eq!(resource.remaining_mines, 40);
    
    resource.decrement_mines(5);
    assert_eq!(resource.remaining_mines, 35);
    
    resource.decrement_mines(10);
    assert_eq!(resource.remaining_mines, 25);
    
    // 0未満にはならない
    resource.decrement_mines(30);
    assert_eq!(resource.remaining_mines, 0);
} 