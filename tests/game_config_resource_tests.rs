use wasm_multiplayer::resources::{GameConfigResource, Difficulty, BoardConfig};

#[test]
fn test_new_resource_has_default_values() {
    let resource = GameConfigResource::new();
    
    // デフォルトは Medium 難易度
    assert_eq!(resource.difficulty, Difficulty::Medium);
    
    // 標準的なボード設定のチェック
    assert_eq!(resource.board_config.width, 16);
    assert_eq!(resource.board_config.height, 16);
    assert_eq!(resource.board_config.mine_count, 40);
    
    // デフォルトのルール
    assert!(resource.safe_first_click);
    assert!(resource.auto_flag_when_complete);
    assert!(!resource.auto_reveal_on_flagging);
}

#[test]
fn test_set_difficulty_easy() {
    let mut resource = GameConfigResource::new();
    resource.set_difficulty(Difficulty::Easy);
    
    assert_eq!(resource.difficulty, Difficulty::Easy);
    assert_eq!(resource.board_config.width, 9);
    assert_eq!(resource.board_config.height, 9);
    assert_eq!(resource.board_config.mine_count, 10);
}

#[test]
fn test_set_difficulty_medium() {
    let mut resource = GameConfigResource::new();
    resource.set_difficulty(Difficulty::Medium);
    
    assert_eq!(resource.difficulty, Difficulty::Medium);
    assert_eq!(resource.board_config.width, 16);
    assert_eq!(resource.board_config.height, 16);
    assert_eq!(resource.board_config.mine_count, 40);
}

#[test]
fn test_set_difficulty_hard() {
    let mut resource = GameConfigResource::new();
    resource.set_difficulty(Difficulty::Hard);
    
    assert_eq!(resource.difficulty, Difficulty::Hard);
    assert_eq!(resource.board_config.width, 30);
    assert_eq!(resource.board_config.height, 16);
    assert_eq!(resource.board_config.mine_count, 99);
}

#[test]
fn test_set_custom_board() {
    let mut resource = GameConfigResource::new();
    resource.set_custom_board(20, 20, 50);
    
    assert_eq!(resource.difficulty, Difficulty::Custom);
    assert_eq!(resource.board_config.width, 20);
    assert_eq!(resource.board_config.height, 20);
    assert_eq!(resource.board_config.mine_count, 50);
}

#[test]
fn test_update_cell_size() {
    let mut resource = GameConfigResource::new();
    
    // 中サイズのボード (16x16) に対して、320x320のキャンバスでセルサイズをテスト
    // 理論的にはセルサイズは約20になるはず (320 / 16 = 20)
    resource.update_cell_size(320.0, 320.0);
    
    // 自動計算によるセルサイズをチェック (余白を考慮)
    assert!(resource.board_config.cell_size > 19.0 && resource.board_config.cell_size < 21.0);
    
    // 小さいキャンバスでのテスト
    resource.update_cell_size(160.0, 160.0);
    
    // 小さいサイズでは最小値に制限される
    assert!(resource.board_config.cell_size >= 10.0);
}

#[test]
fn test_toggle_safe_first_click() {
    let mut resource = GameConfigResource::new();
    
    // デフォルトはtrue
    assert!(resource.safe_first_click);
    
    // トグル
    resource.toggle_safe_first_click();
    assert!(!resource.safe_first_click);
    
    // もう一度トグル
    resource.toggle_safe_first_click();
    assert!(resource.safe_first_click);
}

#[test]
fn test_toggle_auto_flag() {
    let mut resource = GameConfigResource::new();
    
    // デフォルトはtrue
    assert!(resource.auto_flag_when_complete);
    
    // トグル
    resource.toggle_auto_flag();
    assert!(!resource.auto_flag_when_complete);
    
    // もう一度トグル
    resource.toggle_auto_flag();
    assert!(resource.auto_flag_when_complete);
}

#[test]
fn test_toggle_auto_reveal() {
    let mut resource = GameConfigResource::new();
    
    // デフォルトはfalse
    assert!(!resource.auto_reveal_on_flagging);
    
    // トグル
    resource.toggle_auto_reveal();
    assert!(resource.auto_reveal_on_flagging);
    
    // もう一度トグル
    resource.toggle_auto_reveal();
    assert!(!resource.auto_reveal_on_flagging);
}

#[test]
fn test_get_random_seed() {
    let resource = GameConfigResource::new();
    
    // 異なるシード値が生成されることを確認
    let seed1 = resource.get_random_seed();
    let seed2 = resource.get_random_seed();
    
    assert_ne!(seed1, seed2);
}

#[test]
fn test_calculate_score() {
    let mut resource = GameConfigResource::new();
    
    // Easyモードでのスコア計算 (低難易度)
    resource.set_difficulty(Difficulty::Easy);
    let easy_score = resource.calculate_score(100.0, 0.5); // 時間100秒、進捗50%
    
    // Hardモードでのスコア計算 (高難易度)
    resource.set_difficulty(Difficulty::Hard);
    let hard_score = resource.calculate_score(100.0, 0.5); // 同じ条件
    
    // 難易度が高いほどスコアが高くなることを確認
    assert!(hard_score > easy_score);
    
    // 同じ難易度でも、時間が短いほどスコアが高くなることを確認
    let fast_score = resource.calculate_score(50.0, 0.5); // 時間50秒、進捗50%
    assert!(fast_score > hard_score);
    
    // 同じ難易度と時間でも、進捗が多いほどスコアが高くなることを確認
    let high_progress_score = resource.calculate_score(100.0, 0.8); // 時間100秒、進捗80%
    assert!(high_progress_score > hard_score);
} 