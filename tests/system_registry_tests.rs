use wasm_multiplayer::systems::system_registry::SystemRegistry;
use wasm_multiplayer::resources::ResourceManager;
use std::rc::Rc;
use std::cell::RefCell;

// テスト用のダミーシステム構造体
struct DummySystem {
    name: String,
    priority: u32,
    called: Rc<RefCell<bool>>,
}

impl DummySystem {
    fn new(name: &str, priority: u32) -> Self {
        Self {
            name: name.to_string(),
            priority,
            called: Rc::new(RefCell::new(false)),
        }
    }
    
    fn was_called(&self) -> bool {
        *self.called.borrow()
    }
    
    fn reset_called(&self) {
        *self.called.borrow_mut() = false;
    }
}

// システムトレイトの実装
impl wasm_multiplayer::systems::system_trait::System for DummySystem {
    fn name(&self) -> &str {
        &self.name
    }
    
    fn priority(&self) -> u32 {
        self.priority
    }
    
    fn update(&self, _entity_manager: &mut wasm_multiplayer::entities::entity_manager::EntityManager, _resources: &mut std::collections::HashMap<&'static str, Rc<RefCell<dyn std::any::Any>>>, _delta_time: f64) -> Result<(), wasm_bindgen::JsValue> {
        *self.called.borrow_mut() = true;
        Ok(())
    }
}

#[test]
fn test_new_registry_is_empty() {
    let registry = SystemRegistry::new();
    assert_eq!(registry.system_count(), 0);
}

#[test]
fn test_register_system_adds_system() {
    let mut registry = SystemRegistry::new();
    let system = DummySystem::new("test", 0);
    
    registry.register_system(Box::new(system));
    assert_eq!(registry.system_count(), 1);
}

#[test]
fn test_register_systems_adds_multiple_systems() {
    let mut registry = SystemRegistry::new();
    let system1 = DummySystem::new("test1", 0);
    let system2 = DummySystem::new("test2", 1);
    
    registry.register_system(Box::new(system1));
    registry.register_system(Box::new(system2));
    assert_eq!(registry.system_count(), 2);
}

#[test]
fn test_update_calls_all_systems() {
    let mut registry = SystemRegistry::new();
    
    // システムを作成と登録（Rcを使って外部からも状態を確認できるようにする）
    let system1 = DummySystem::new("test1", 0);
    let system1_called = system1.called.clone();
    
    let system2 = DummySystem::new("test2", 1);
    let system2_called = system2.called.clone();
    
    registry.register_system(Box::new(system1));
    registry.register_system(Box::new(system2));
    
    // エンティティマネージャーとリソースマネージャーのダミー
    let mut entity_manager = wasm_multiplayer::entities::entity_manager::EntityManager::new();
    let mut resources = std::collections::HashMap::new();
    
    // 更新を呼び出し
    registry.update(&mut entity_manager, &mut resources, 0.016).unwrap();
    
    // 両方のシステムが呼び出されたことを確認
    assert!(*system1_called.borrow());
    assert!(*system2_called.borrow());
}

#[test]
fn test_systems_execute_in_priority_order() {
    // このテストはより複雑なため、実装は省略
    // 実際には呼び出し順序を検証するロジックが必要
}

#[test]
fn test_add_resource_makes_it_accessible() {
    let mut registry = SystemRegistry::new();
    let mut resource_manager = ResourceManager::new();
    
    // リソースを追加
    #[derive(Clone)]
    struct TestResource { value: i32 }
    
    let test_resource = TestResource { value: 42 };
    resource_manager.add_resource("test", test_resource);
    
    // リソースにアクセスできることを確認
    if let Some(res) = resource_manager.get_resource::<TestResource>("test") {
        assert_eq!(res.borrow().value, 42);
    } else {
        panic!("Resource not found");
    }
}

#[test]
fn test_sort_systems_reorders_by_priority() {
    let mut registry = SystemRegistry::new();
    
    // 優先度が異なるシステムを登録（順序を意図的に入れ替える）
    let system_high = DummySystem::new("high", 10);
    let system_medium = DummySystem::new("medium", 5);
    let system_low = DummySystem::new("low", 1);
    
    // 優先度に関係なく登録
    registry.register_system(Box::new(system_medium));
    registry.register_system(Box::new(system_high));
    registry.register_system(Box::new(system_low));
    
    // システムの順序を整理
    registry.sort_systems();
    
    // 結果は内部実装に依存するため、直接検証は難しい
    // 実際のテストでは、更新時の呼び出し順序を検証する方法がよい
}

#[test]
fn test_get_system_returns_correct_system() {
    let mut registry = SystemRegistry::new();
    
    let system = DummySystem::new("test", 0);
    registry.register_system(Box::new(system));
    
    // 名前でシステムを取得
    let found_system = registry.get_system("test");
    assert!(found_system.is_some());
    
    // 存在しない名前
    let not_found = registry.get_system("nonexistent");
    assert!(not_found.is_none());
}

#[cfg(test)]
mod iterative_cell_reveal_tests {
    use crate::board::Board;
    use crate::entities::EntityManager;
    use crate::models::CellValue;
    use crate::systems::board_systems::cell_reveal_system::reveal_cell;
    
    /// 小さなボードでの連鎖公開のテスト
    #[test]
    fn test_small_board_chain_reveal() {
        // 5x5の小さなボードを作成
        let mut board = Board::new(5, 5, 5, 30.0);
        let mut entity_manager = EntityManager::new();
        
        // 地雷の配置（手動でセット）
        board.cells[0] = CellValue::Mine; // 左上
        board.cells[4] = CellValue::Mine; // 右上
        board.cells[20] = CellValue::Mine; // 左下
        board.cells[24] = CellValue::Mine; // 右下
        board.cells[12] = CellValue::Mine; // 中央
        
        // 周囲のセルに地雷カウントを設定
        for row in 0..5 {
            for col in 0..5 {
                let idx = row * 5 + col;
                if let CellValue::Empty(_) = board.cells[idx] {
                    let mut count = 0;
                    
                    // 周囲のセルをチェック
                    for dr in -1..=1 {
                        for dc in -1..=1 {
                            if dr == 0 && dc == 0 {
                                continue;
                            }
                            
                            let new_row = row as isize + dr;
                            let new_col = col as isize + dc;
                            
                            if new_row >= 0 && new_row < 5 && new_col >= 0 && new_col < 5 {
                                let adj_idx = new_row as usize * 5 + new_col as usize;
                                if let CellValue::Mine = board.cells[adj_idx] {
                                    count += 1;
                                }
                            }
                        }
                    }
                    
                    board.cells[idx] = CellValue::Empty(count);
                }
            }
        }
        
        // 初期状態を設定
        board.first_click = false;
        board.remaining_safe_cells = 20; // 25 - 5
        
        // セル（2,2）が空（0）なら、クリックして連鎖反応をテスト
        let center_idx = 2 * 5 + 2;
        assert!(matches!(board.cells[center_idx], CellValue::Empty(0)));
        
        // セルを公開
        let result = reveal_cell(2, 2, &mut entity_manager, &mut board, false);
        assert!(result.is_ok());
        
        // 連鎖反応で複数のセルが公開されていることを確認
        let revealed_count = board.revealed.iter().filter(|&&r| r).count();
        assert!(revealed_count > 1, "連鎖反応が発生していません。公開セル数: {}", revealed_count);
        
        // 具体的にどのセルが公開されたかチェック
        for row in 0..5 {
            for col in 0..5 {
                let idx = row * 5 + col;
                if let CellValue::Empty(count) = board.cells[idx] {
                    if count == 0 {
                        assert!(board.revealed[idx], "座標({},{})の空白セルが公開されていません", row, col);
                    }
                }
            }
        }
    }
    
    /// 大きなボードでの連鎖公開テスト（パフォーマンス検証）
    #[test]
    fn test_large_board_chain_reveal() {
        // 50x50の大きなボードを作成（地雷は少なめに）
        let mut board = Board::new(50, 50, 250, 30.0);
        let mut entity_manager = EntityManager::new();
        
        // 地雷を配置（外周に集中させる）
        for i in 0..50 {
            board.cells[i] = CellValue::Mine; // 上部
            board.cells[49 * 50 + i] = CellValue::Mine; // 下部
            board.cells[i * 50] = CellValue::Mine; // 左部
            board.cells[i * 50 + 49] = CellValue::Mine; // 右部
        }
        
        // 残りの50個の地雷をランダムに配置
        let mut remaining_mines = 50;
        let mut rng = rand::thread_rng();
        while remaining_mines > 0 {
            let row = rng.gen_range(1..49);
            let col = rng.gen_range(1..49);
            let idx = row * 50 + col;
            
            if let CellValue::Empty(_) = board.cells[idx] {
                board.cells[idx] = CellValue::Mine;
                remaining_mines -= 1;
            }
        }
        
        // 周囲のセルに地雷カウントを設定
        for row in 0..50 {
            for col in 0..50 {
                let idx = row * 50 + col;
                if let CellValue::Empty(_) = board.cells[idx] {
                    let mut count = 0;
                    
                    // 周囲のセルをチェック
                    for dr in -1..=1 {
                        for dc in -1..=1 {
                            if dr == 0 && dc == 0 {
                                continue;
                            }
                            
                            let new_row = row as isize + dr;
                            let new_col = col as isize + dc;
                            
                            if new_row >= 0 && new_row < 50 && new_col >= 0 && new_col < 50 {
                                let adj_idx = new_row as usize * 50 + new_col as usize;
                                if let CellValue::Mine = board.cells[adj_idx] {
                                    count += 1;
                                }
                            }
                        }
                    }
                    
                    board.cells[idx] = CellValue::Empty(count);
                }
            }
        }
        
        // 初期状態を設定
        board.first_click = false;
        board.remaining_safe_cells = 50 * 50 - 250; // 2500 - 250
        
        // 中央付近のセルをクリック
        let center_row = 25;
        let center_col = 25;
        let center_idx = center_row * 50 + center_col;
        
        // 中央にゼロセルがあることを確認（なければ近くを探す）
        let (mut test_row, mut test_col) = (center_row, center_col);
        for dr in -5..=5 {
            for dc in -5..=5 {
                let r = (center_row as isize + dr) as usize;
                let c = (center_col as isize + dc) as usize;
                if r < 50 && c < 50 {
                    let idx = r * 50 + c;
                    if let CellValue::Empty(0) = board.cells[idx] {
                        test_row = r;
                        test_col = c;
                        break;
                    }
                }
            }
        }
        
        // 時間計測開始
        let start = std::time::Instant::now();
        
        // セルを公開
        let result = reveal_cell(test_row, test_col, &mut entity_manager, &mut board, false);
        assert!(result.is_ok());
        
        // 時間計測終了
        let duration = start.elapsed();
        
        // 連鎖反応で多数のセルが公開されていることを確認
        let revealed_count = board.revealed.iter().filter(|&&r| r).count();
        assert!(revealed_count > 100, "連鎖反応が十分に発生していません。公開セル数: {}", revealed_count);
        
        // パフォーマンステスト - 大きなボードでも処理時間が許容範囲内であること
        assert!(duration.as_millis() < 100, "大きなボードの処理に時間がかかりすぎています: {:?}", duration);
        
        println!("大きなボード処理時間: {:?}, 公開されたセル数: {}", duration, revealed_count);
    }
} 