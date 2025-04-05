#![feature(test)]
extern crate test;
extern crate wasm_multiplayer;
extern crate rand;

use test::Bencher;
use wasm_multiplayer::board::Board;
use wasm_multiplayer::entities::EntityManager;
use wasm_multiplayer::models::CellValue;
use wasm_multiplayer::systems::board_systems::cell_reveal_system::reveal_cell;
use rand::Rng;

// 再帰的なアルゴリズムを模倣した計測関数
fn reveal_connected_recursive(
    row: usize,
    col: usize,
    board: &mut Board,
    visited: &mut Vec<bool>
) {
    let width = board.width;
    let height = board.height;
    let index = row * width + col;
    
    // 範囲チェックと既に訪問済みかどうかをチェック
    if row >= height || col >= width || visited[index] {
        return;
    }
    
    // このセルを訪問済みとしてマーク
    visited[index] = true;
    
    // 既に開かれたセルはスキップ
    if board.revealed[index] {
        return;
    }
    
    // セルを開く
    board.revealed[index] = true;
    board.remaining_safe_cells -= 1;
    
    // 地雷の場合はここで終了
    if let CellValue::Mine = board.cells[index] {
        return;
    }
    
    // 数字のセルは連鎖しない
    if let CellValue::Empty(count) = board.cells[index] {
        if count > 0 {
            return;
        }
    }
    
    // 周囲8方向のセルを再帰的に開く
    for dr in -1..=1 {
        for dc in -1..=1 {
            if dr == 0 && dc == 0 {
                continue;
            }
            
            let new_row = row as isize + dr;
            let new_col = col as isize + dc;
            
            if new_row >= 0 && new_row < height as isize && 
               new_col >= 0 && new_col < width as isize {
                reveal_connected_recursive(
                    new_row as usize,
                    new_col as usize,
                    board,
                    visited
                );
            }
        }
    }
}

// 再帰実装を使用したベンチマーク
#[bench]
fn bench_recursive_reveal_small(b: &mut Bencher) {
    let width = 10;
    let height = 10;
    let mine_count = 10;
    
    b.iter(|| {
        // 新しいボードとエンティティマネージャー
        let mut board = Board::new(width, height, mine_count, 30.0);
        let mut entity_manager = EntityManager::new();
        
        // 地雷を配置
        for i in 0..mine_count {
            board.cells[i] = CellValue::Mine;
        }
        
        // 数字を計算
        for row in 0..height {
            for col in 0..width {
                let idx = row * width + col;
                if let CellValue::Empty(_) = board.cells[idx] {
                    let mut count = 0;
                    
                    for dr in -1..=1 {
                        for dc in -1..=1 {
                            if dr == 0 && dc == 0 {
                                continue;
                            }
                            
                            let new_row = row as isize + dr;
                            let new_col = col as isize + dc;
                            
                            if new_row >= 0 && new_row < height as isize &&
                               new_col >= 0 && new_col < width as isize {
                                let adj_idx = new_row as usize * width + new_col as usize;
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
        
        // 連鎖反応が起きる位置を探す
        let mut reveal_pos = (5, 5);
        for row in 0..height {
            for col in 0..width {
                let idx = row * width + col;
                if let CellValue::Empty(0) = board.cells[idx] {
                    reveal_pos = (row, col);
                    break;
                }
            }
        }
        
        // 再帰的に開く
        let mut visited = vec![false; width * height];
        reveal_connected_recursive(reveal_pos.0, reveal_pos.1, &mut board, &mut visited);
    });
}

// 非再帰（イテレーティブ）実装を使用したベンチマーク
#[bench]
fn bench_iterative_reveal_small(b: &mut Bencher) {
    let width = 10;
    let height = 10;
    let mine_count = 10;
    
    b.iter(|| {
        // 新しいボードとエンティティマネージャー
        let mut board = Board::new(width, height, mine_count, 30.0);
        let mut entity_manager = EntityManager::new();
        
        // 地雷を配置
        for i in 0..mine_count {
            board.cells[i] = CellValue::Mine;
        }
        
        // 数字を計算
        for row in 0..height {
            for col in 0..width {
                let idx = row * width + col;
                if let CellValue::Empty(_) = board.cells[idx] {
                    let mut count = 0;
                    
                    for dr in -1..=1 {
                        for dc in -1..=1 {
                            if dr == 0 && dc == 0 {
                                continue;
                            }
                            
                            let new_row = row as isize + dr;
                            let new_col = col as isize + dc;
                            
                            if new_row >= 0 && new_row < height as isize &&
                               new_col >= 0 && new_col < width as isize {
                                let adj_idx = new_row as usize * width + new_col as usize;
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
        
        // 連鎖反応が起きる位置を探す
        let mut reveal_pos = (5, 5);
        for row in 0..height {
            for col in 0..width {
                let idx = row * width + col;
                if let CellValue::Empty(0) = board.cells[idx] {
                    reveal_pos = (row, col);
                    break;
                }
            }
        }
        
        // 非再帰的に開く
        let _ = reveal_cell(reveal_pos.0, reveal_pos.1, &mut entity_manager, &mut board, false);
    });
}

// 大きなボードでのベンチマーク（再帰）
#[bench]
fn bench_recursive_reveal_large(b: &mut Bencher) {
    let width = 30;
    let height = 30;
    let mine_count = 100;
    
    b.iter(|| {
        // 新しいボードとエンティティマネージャー
        let mut board = Board::new(width, height, mine_count, 30.0);
        let mut entity_manager = EntityManager::new();
        
        // 地雷を配置（周囲に集中）
        for i in 0..width {
            board.cells[i] = CellValue::Mine; // 上部
            board.cells[(height-1) * width + i] = CellValue::Mine; // 下部
        }
        
        for i in 0..height {
            board.cells[i * width] = CellValue::Mine; // 左部
            board.cells[i * width + (width-1)] = CellValue::Mine; // 右部
        }
        
        // 残りの地雷をランダムに配置
        let edge_mines = (width + height) * 2 - 4; // 重複を除く
        let mut remaining_mines = mine_count.saturating_sub(edge_mines);
        let mut rng = rand::thread_rng();
        
        while remaining_mines > 0 {
            let row = rng.gen_range(1..(height-1));
            let col = rng.gen_range(1..(width-1));
            let idx = row * width + col;
            
            if let CellValue::Empty(_) = board.cells[idx] {
                board.cells[idx] = CellValue::Mine;
                remaining_mines -= 1;
            }
        }
        
        // 数字を計算
        for row in 0..height {
            for col in 0..width {
                let idx = row * width + col;
                if let CellValue::Empty(_) = board.cells[idx] {
                    let mut count = 0;
                    
                    for dr in -1..=1 {
                        for dc in -1..=1 {
                            if dr == 0 && dc == 0 {
                                continue;
                            }
                            
                            let new_row = row as isize + dr;
                            let new_col = col as isize + dc;
                            
                            if new_row >= 0 && new_row < height as isize &&
                               new_col >= 0 && new_col < width as isize {
                                let adj_idx = new_row as usize * width + new_col as usize;
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
        
        // 中央付近で連鎖反応が起きる位置を探す
        let center_row = height / 2;
        let center_col = width / 2;
        let mut reveal_pos = (center_row, center_col);
        
        'search: for dr in -3..=3 {
            for dc in -3..=3 {
                let row = (center_row as isize + dr) as usize;
                let col = (center_col as isize + dc) as usize;
                
                if row < height && col < width {
                    let idx = row * width + col;
                    if let CellValue::Empty(0) = board.cells[idx] {
                        reveal_pos = (row, col);
                        break 'search;
                    }
                }
            }
        }
        
        // 再帰的に開く
        let mut visited = vec![false; width * height];
        reveal_connected_recursive(reveal_pos.0, reveal_pos.1, &mut board, &mut visited);
    });
}

// 大きなボードでのベンチマーク（非再帰）
#[bench]
fn bench_iterative_reveal_large(b: &mut Bencher) {
    let width = 30;
    let height = 30;
    let mine_count = 100;
    
    b.iter(|| {
        // 新しいボードとエンティティマネージャー
        let mut board = Board::new(width, height, mine_count, 30.0);
        let mut entity_manager = EntityManager::new();
        
        // 地雷を配置（周囲に集中）
        for i in 0..width {
            board.cells[i] = CellValue::Mine; // 上部
            board.cells[(height-1) * width + i] = CellValue::Mine; // 下部
        }
        
        for i in 0..height {
            board.cells[i * width] = CellValue::Mine; // 左部
            board.cells[i * width + (width-1)] = CellValue::Mine; // 右部
        }
        
        // 残りの地雷をランダムに配置
        let edge_mines = (width + height) * 2 - 4; // 重複を除く
        let mut remaining_mines = mine_count.saturating_sub(edge_mines);
        let mut rng = rand::thread_rng();
        
        while remaining_mines > 0 {
            let row = rng.gen_range(1..(height-1));
            let col = rng.gen_range(1..(width-1));
            let idx = row * width + col;
            
            if let CellValue::Empty(_) = board.cells[idx] {
                board.cells[idx] = CellValue::Mine;
                remaining_mines -= 1;
            }
        }
        
        // 数字を計算
        for row in 0..height {
            for col in 0..width {
                let idx = row * width + col;
                if let CellValue::Empty(_) = board.cells[idx] {
                    let mut count = 0;
                    
                    for dr in -1..=1 {
                        for dc in -1..=1 {
                            if dr == 0 && dc == 0 {
                                continue;
                            }
                            
                            let new_row = row as isize + dr;
                            let new_col = col as isize + dc;
                            
                            if new_row >= 0 && new_row < height as isize &&
                               new_col >= 0 && new_col < width as isize {
                                let adj_idx = new_row as usize * width + new_col as usize;
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
        
        // 中央付近で連鎖反応が起きる位置を探す
        let center_row = height / 2;
        let center_col = width / 2;
        let mut reveal_pos = (center_row, center_col);
        
        'search: for dr in -3..=3 {
            for dc in -3..=3 {
                let row = (center_row as isize + dr) as usize;
                let col = (center_col as isize + dc) as usize;
                
                if row < height && col < width {
                    let idx = row * width + col;
                    if let CellValue::Empty(0) = board.cells[idx] {
                        reveal_pos = (row, col);
                        break 'search;
                    }
                }
            }
        }
        
        // 非再帰的に開く
        let _ = reveal_cell(reveal_pos.0, reveal_pos.1, &mut entity_manager, &mut board, false);
    });
}

// メモリ使用量テスト（超大きなボード）
#[test]
fn test_memory_usage_large_boards() {
    let configs = [
        (50, 50, 250),    // 中規模 - 2,500セル
        (100, 100, 1000), // 大規模 - 10,000セル
    ];
    
    for (width, height, mines) in configs {
        println!("テスト: {}x{} ボード ({} 地雷)", width, height, mines);
        
        // 非再帰アルゴリズム
        {
            let mut board = Board::new(width, height, mines, 30.0);
            let mut entity_manager = EntityManager::new();
            
            // 地雷を配置
            for i in 0..mines {
                if i < width * height {
                    board.cells[i] = CellValue::Mine;
                }
            }
            
            // 中央付近のセルを開く
            let start = std::time::Instant::now();
            let _ = reveal_cell(width/2, height/2, &mut entity_manager, &mut board, false);
            let duration = start.elapsed();
            
            let revealed = board.revealed.iter().filter(|&&r| r).count();
            println!("  非再帰: {}ms, 公開セル数: {}", duration.as_millis(), revealed);
        }
        
        // 再帰アルゴリズム
        {
            let mut board = Board::new(width, height, mines, 30.0);
            
            // 地雷を配置
            for i in 0..mines {
                if i < width * height {
                    board.cells[i] = CellValue::Mine;
                }
            }
            
            // 中央付近のセルを開く
            let start = std::time::Instant::now();
            let mut visited = vec![false; width * height];
            reveal_connected_recursive(width/2, height/2, &mut board, &mut visited);
            let duration = start.elapsed();
            
            let revealed = board.revealed.iter().filter(|&&r| r).count();
            println!("  再帰: {}ms, 公開セル数: {}", duration.as_millis(), revealed);
        }
        
        println!(""); // 空行
    }
} 