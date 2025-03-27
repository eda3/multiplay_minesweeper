use wasm_multiplayer::resources::{PlayerStateResource, MouseState};

#[test]
fn test_new_resource_has_default_values() {
    let resource = PlayerStateResource::new();
    
    assert_eq!(resource.all_players().len(), 0);
    assert!(resource.local_player().is_none());
    assert!(resource.local_player_id().is_none());
    assert_eq!(resource.mouse_state(), MouseState::Up);
}

#[test]
fn test_set_local_player_id() {
    let mut resource = PlayerStateResource::new();
    
    resource.set_local_player_id("player1".to_string());
    
    assert_eq!(resource.local_player_id(), Some(&"player1".to_string()));
    assert!(resource.local_player().is_none()); // プレイヤーはまだ追加されていない
}

#[test]
fn test_add_player() {
    let mut resource = PlayerStateResource::new();
    
    // プレイヤーを追加
    resource.add_player("player1".to_string(), 100.0, 200.0, "#FF0000".to_string());
    
    // プレイヤーが追加されたか確認
    assert_eq!(resource.all_players().len(), 1);
    assert!(resource.has_player(&"player1".to_string()));
    
    // プレイヤーの情報が正しいか確認
    if let Some(player) = resource.get_player(&"player1".to_string()) {
        assert_eq!(player.id, "player1");
        assert_eq!(player.x, 100.0);
        assert_eq!(player.y, 200.0);
        assert_eq!(player.color, "#FF0000");
    } else {
        panic!("Player not found");
    }
    
    // ローカルプレイヤーが設定されているか確認
    resource.set_local_player_id("player1".to_string());
    assert!(resource.local_player().is_some());
}

#[test]
fn test_update_player_position() {
    let mut resource = PlayerStateResource::new();
    
    // プレイヤーを追加
    resource.add_player("player1".to_string(), 100.0, 200.0, "#FF0000".to_string());
    
    // プレイヤーの位置を更新
    resource.update_player_position("player1", 150.0, 250.0);
    
    // 更新された位置を確認
    if let Some(player) = resource.get_player(&"player1".to_string()) {
        assert_eq!(player.x, 150.0);
        assert_eq!(player.y, 250.0);
    } else {
        panic!("Player not found");
    }
    
    // 存在しないプレイヤーの位置を更新してもエラーにならない
    resource.update_player_position("nonexistent", 300.0, 300.0);
}

#[test]
fn test_remove_player() {
    let mut resource = PlayerStateResource::new();
    
    // プレイヤーを追加
    resource.add_player("player1".to_string(), 100.0, 200.0, "#FF0000".to_string());
    resource.add_player("player2".to_string(), 300.0, 400.0, "#00FF00".to_string());
    
    assert_eq!(resource.all_players().len(), 2);
    
    // プレイヤーを削除
    resource.remove_player("player1");
    
    assert_eq!(resource.all_players().len(), 1);
    assert!(!resource.has_player(&"player1".to_string()));
    assert!(resource.has_player(&"player2".to_string()));
    
    // 存在しないプレイヤーを削除してもエラーにならない
    resource.remove_player("nonexistent");
}

#[test]
fn test_set_mouse_state() {
    let mut resource = PlayerStateResource::new();
    
    // デフォルトはUp
    assert_eq!(resource.mouse_state(), MouseState::Up);
    
    // マウス状態を変更
    resource.set_mouse_state(MouseState::LeftDown);
    assert_eq!(resource.mouse_state(), MouseState::LeftDown);
    
    resource.set_mouse_state(MouseState::RightDown);
    assert_eq!(resource.mouse_state(), MouseState::RightDown);
    
    resource.set_mouse_state(MouseState::Up);
    assert_eq!(resource.mouse_state(), MouseState::Up);
}

#[test]
fn test_to_json() {
    let mut resource = PlayerStateResource::new();
    
    // プレイヤーを追加
    resource.add_player("player1".to_string(), 100.0, 200.0, "#FF0000".to_string());
    resource.add_player("player2".to_string(), 300.0, 400.0, "#00FF00".to_string());
    
    // JSONに変換
    let json = resource.to_json();
    
    // JSONの構造を確認
    assert!(json.is_object());
    assert!(json.as_object().unwrap().contains_key("players"));
    
    let players = json.as_object().unwrap().get("players").unwrap();
    assert!(players.is_object());
    assert!(players.as_object().unwrap().contains_key("player1"));
    assert!(players.as_object().unwrap().contains_key("player2"));
    
    // プレイヤー1の情報を確認
    let player1 = players.as_object().unwrap().get("player1").unwrap();
    assert_eq!(player1.as_object().unwrap().get("x").unwrap().as_f64().unwrap(), 100.0);
    assert_eq!(player1.as_object().unwrap().get("y").unwrap().as_f64().unwrap(), 200.0);
    assert_eq!(player1.as_object().unwrap().get("color").unwrap().as_str().unwrap(), "#FF0000");
} 