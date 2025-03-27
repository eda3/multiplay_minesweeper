use wasm_multiplayer::resources::TimeResource;

#[test]
fn test_new_resource_has_default_values() {
    let resource = TimeResource::new();
    
    assert_eq!(resource.delta_time, 0.0);
    assert_eq!(resource.total_time, 0.0);
    assert_eq!(resource.frame_count, 0);
    assert_eq!(resource.fps, 0.0);
    assert_eq!(resource.last_fps_update, 0.0);
}

#[test]
fn test_update_advances_time() {
    let mut resource = TimeResource::new();
    
    // 最初のフレーム
    resource.update(16.7); // 16.7ミリ秒経過
    
    assert_eq!(resource.delta_time, 16.7);
    assert_eq!(resource.total_time, 16.7);
    assert_eq!(resource.frame_count, 1);
    
    // 2フレーム目
    resource.update(16.7); // さらに16.7ミリ秒
    
    assert_eq!(resource.delta_time, 16.7);
    assert_eq!(resource.total_time, 33.4);
    assert_eq!(resource.frame_count, 2);
}

#[test]
fn test_fps_calculation() {
    let mut resource = TimeResource::new();
    
    // FPS計算の間隔は通常1秒
    // シミュレーションのために複数のフレームを短い間隔で更新
    for _ in 0..10 {
        resource.update(100.0); // 100ミリ秒ごと = 10FPS
    }
    
    // 10 フレーム、合計 1000 ミリ秒経過したため、FPSは約10になる
    assert!(resource.fps > 9.0 && resource.fps < 11.0);
}

#[test]
fn test_reset() {
    let mut resource = TimeResource::new();
    
    // いくつかのフレームを進める
    resource.update(16.7);
    resource.update(16.7);
    
    // リセット
    resource.reset();
    
    // 値がリセットされている
    assert_eq!(resource.delta_time, 0.0);
    assert_eq!(resource.total_time, 0.0);
    assert_eq!(resource.frame_count, 0);
}

#[test]
fn test_time_scale() {
    let mut resource = TimeResource::new();
    
    // 時間スケールを0.5に設定（半分の速度）
    resource.set_time_scale(0.5);
    
    // 16.7ミリ秒が経過
    resource.update(16.7);
    
    // 実際の経過時間は16.7 * 0.5 = 8.35ミリ秒になる
    assert!(resource.delta_time < 8.4 && resource.delta_time > 8.3);
    assert!(resource.total_time < 8.4 && resource.total_time > 8.3);
    
    // 時間スケールを2.0に設定（2倍の速度）
    resource.set_time_scale(2.0);
    
    // リセット
    resource.reset();
    
    // 16.7ミリ秒が経過
    resource.update(16.7);
    
    // 実際の経過時間は16.7 * 2.0 = 33.4ミリ秒になる
    assert!(resource.delta_time < 33.5 && resource.delta_time > 33.3);
    assert!(resource.total_time < 33.5 && resource.total_time > 33.3);
} 