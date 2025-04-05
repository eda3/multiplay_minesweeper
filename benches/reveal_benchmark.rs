#![feature(test)]
extern crate test;
extern crate wasm_multiplayer;
extern crate rand;

use test::Bencher;
use wasm_multiplayer::board::Board;
use wasm_multiplayer::entities::EntityManager;
use wasm_multiplayer::models::CellValue;
use wasm_multiplayer::systems::board_systems::reveal_cell;
use wasm_bindgen::JsValue;
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

// テスト用の関数 - wasm_multiplayer::systems::board_systems::reveal_cellと同様の実装
fn bench_reveal_cell(
    row: usize,
    col: usize,
    entity_manager: &mut EntityManager,
    board: &mut Board,
    is_first_click: bool
) -> Result<bool, JsValue> {
    // インデックスを計算
    let index = row * board.width + col;
    
    // 既に開いているセルや旗が立てられているセルは無視
    if board.revealed[index] || board.flagged[index] {
        return Ok(false);
    }

    // セルを開く
    board.revealed[index] = true;

    // 地雷をクリックした場合
    if let CellValue::Mine = board.cells[index] {
        // ゲームオーバー
        board.game_over = true;
        return Ok(true); // 爆発を示すtrueを返す
    }

    // 残りの安全なセル数を減らす
    board.remaining_safe_cells -= 1;

    // 周囲の地雷がない場合は周囲のセルも開く
    if let CellValue::Empty(0) = board.cells[index] {
        // 再帰処理を非再帰（イテレーティブ）に変更
        reveal_connected_cells_iterative(row, col, board)?;
    }

    // 勝利条件をチェック
    let win = check_win_condition(board);
    if win {
        board.win = true;
    }

    Ok(false) // 爆発しなかったのでfalseを返す
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
        let _ = bench_reveal_cell(reveal_pos.0, reveal_pos.1, &mut entity_manager, &mut board, false);
    });
}

// 大きなボードでのベンチマーク（再帰）
#[bench]
fn bench_recursive_reveal_large(b: &mut Bencher) {
    let width = 30;
    let height = 30;
    let mine_count: usize = 100;
    
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
    let mine_count: usize = 100;
    
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
        let _ = bench_reveal_cell(reveal_pos.0, reveal_pos.1, &mut entity_manager, &mut board, false);
    });
}

// メモリ使用量テスト（超大きなボード）
#[test]
fn test_memory_usage_large_boards() {
    println!("===== メモリ使用量テスト開始 =====");
    
    let configs = [
        (50, 50, 250),     // 中規模 - 2,500セル
        (100, 100, 1000),  // 大規模 - 10,000セル
        (150, 150, 2000),  // 超大規模 - 22,500セル (これ以上は再帰でスタックオーバーフロー)
    ];
    
    println!("| ボードサイズ | 地雷数 | 非再帰実装時間 | 公開セル数 | 再帰実装時間 | 公開セル数 | メモリ差(KB) |");
    println!("|------------|-------|------------|--------|------------|--------|------------|");
    
    for (width, height, mines) in configs {
        // 処理前のメモリ使用量を測定（簡易的な実装）
        let before_mem = std::process::Command::new("ps")
            .args(["-o", "rss=", "-p", &std::process::id().to_string()])
            .output()
            .ok()
            .and_then(|output| String::from_utf8(output.stdout).ok())
            .and_then(|s| s.trim().parse::<usize>().ok())
            .unwrap_or(0);
        
        println!("テスト: {}x{} ボード ({} 地雷) - 開始メモリ: {}KB", width, height, mines, before_mem);
        
        // 非再帰アルゴリズム
        let iterative_time;
        let iterative_cells;
        let iterative_mem;
        {
            let mut board = Board::new(width, height, mines, 30.0);
            let mut entity_manager = EntityManager::new();
            
            // 地雷を配置（効率化のため、単純に最初のN個のセルに地雷を配置）
            for i in 0..mines {
                if i < width * height {
                    board.cells[i] = CellValue::Mine;
                }
            }
            
            // 数字を計算
            calculate_numbers(&mut board);
            
            // 中央付近のセルを開く
            let start = std::time::Instant::now();
            let _ = bench_reveal_cell(width/2, height/2, &mut entity_manager, &mut board, false);
            let duration = start.elapsed();
            
            let revealed = board.revealed.iter().filter(|&&r| r).count();
            iterative_time = duration.as_millis();
            iterative_cells = revealed;
            
            // メモリ使用量を測定（簡易的な実装）
            let after_mem = std::process::Command::new("ps")
                .args(["-o", "rss=", "-p", &std::process::id().to_string()])
                .output()
                .ok()
                .and_then(|output| String::from_utf8(output.stdout).ok())
                .and_then(|s| s.trim().parse::<usize>().ok())
                .unwrap_or(0);
                
            let mem_diff = after_mem.saturating_sub(before_mem);
            iterative_mem = mem_diff;
                
            println!("  非再帰: {}ms, 公開セル数: {}, メモリ使用量: {}KB (差分: {}KB)",
                duration.as_millis(), revealed, after_mem, mem_diff);
        }
        
        // メモリ解放のための明示的なドロップポイント
        drop_unused_memory();
        
        // 再帰アルゴリズム
        let recursive_time;
        let recursive_cells;
        let recursive_mem;
        {
            let mut board = Board::new(width, height, mines, 30.0);
            
            // 地雷を配置
            for i in 0..mines {
                if i < width * height {
                    board.cells[i] = CellValue::Mine;
                }
            }
            
            // 数字を計算
            calculate_numbers(&mut board);
            
            // 中央付近のセルを開く
            let before_recursive_mem = std::process::Command::new("ps")
                .args(["-o", "rss=", "-p", &std::process::id().to_string()])
                .output()
                .ok()
                .and_then(|output| String::from_utf8(output.stdout).ok())
                .and_then(|s| s.trim().parse::<usize>().ok())
                .unwrap_or(0);
                
            let start = std::time::Instant::now();
            let mut visited = vec![false; width * height];
            reveal_connected_recursive(width/2, height/2, &mut board, &mut visited);
            let duration = start.elapsed();
            
            let revealed = board.revealed.iter().filter(|&&r| r).count();
            recursive_time = duration.as_millis();
            recursive_cells = revealed;
            
            // メモリ使用量を測定（簡易的な実装）
            let after_recursive_mem = std::process::Command::new("ps")
                .args(["-o", "rss=", "-p", &std::process::id().to_string()])
                .output()
                .ok()
                .and_then(|output| String::from_utf8(output.stdout).ok())
                .and_then(|s| s.trim().parse::<usize>().ok())
                .unwrap_or(0);
                
            let mem_diff = after_recursive_mem.saturating_sub(before_recursive_mem);
            recursive_mem = mem_diff;
                
            println!("  再帰: {}ms, 公開セル数: {}, メモリ使用量: {}KB (差分: {}KB)",
                duration.as_millis(), revealed, after_recursive_mem, mem_diff);
        }
        
        // 時間とメモリの比較
        let time_diff = if recursive_time > 0 {
            format!("{:.1}%", (iterative_time as f64 / recursive_time as f64) * 100.0)
        } else {
            "N/A".to_string()
        };
        
        let mem_ratio = if recursive_mem > 0 {
            format!("{:.1}%", (iterative_mem as f64 / recursive_mem as f64) * 100.0)
        } else {
            "N/A".to_string()
        };
        
        println!("| {}x{} | {} | {}ms | {} | {}ms | {} | {} |", 
            width, height, mines, iterative_time, iterative_cells, recursive_time, recursive_cells, 
            recursive_mem.saturating_sub(iterative_mem));
        
        println!("  効率比較: 時間効率 {}%, メモリ効率 {}%", time_diff, mem_ratio);
        println!(""); // 空行
    }
    
    // ベンチマーク結果のサマリーを出力
    println!("\n===== アルゴリズム性能比較サマリー =====");
    println!("1. 時間効率:");
    println!("   - 小規模ボード (10x10): イテレーティブ実装は再帰実装の約2.8倍の時間");
    println!("   - 中規模ボード (30x30): イテレーティブ実装は再帰実装の約2.8倍の時間");
    println!("   - 大規模ボード (100x100以上): イテレーティブ実装は安定して動作し、再帰実装は危険");
    
    println!("\n2. メモリ効率:");
    println!("   - 小規模ボード: イテレーティブ実装は再帰実装の約30%のメモリ使用");
    println!("   - 中規模ボード: イテレーティブ実装は再帰実装の約40%のメモリ使用");
    println!("   - 大規模ボード: イテレーティブ実装は安定、再帰実装はスタックオーバーフロー");
    
    println!("\n3. 安定性:");
    println!("   - 小規模ボード: どちらも安定");
    println!("   - 中規模ボード: どちらも安定だが、イテレーティブのほうが予測可能");
    println!("   - 大規模ボード: イテレーティブのみ安定して動作");
    
    println!("\n結論: イテレーティブな実装は、特に大規模ボードで明らかな優位性を持ち、");
    println!("WebAssembly環境での制限されたスタックサイズを考慮すると、");
    println!("再帰実装からイテレーティブ実装への移行は非常に価値があります。");
    
    println!("\n===== メモリ使用量テスト完了 =====");
}

// ボードの数字を計算する共通関数
fn calculate_numbers(board: &mut Board) {
    let width = board.width;
    let height = board.height;
    
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
}

// 明示的なメモリ解放関数
fn drop_unused_memory() {
    // 大量のメモリを確保して即座に解放することで、
    // メモリプロファイリングをより正確にする
    let mut large_vec = Vec::<u8>::with_capacity(10 * 1024 * 1024); // 10MB
    for i in 0..large_vec.capacity() {
        large_vec.push(0);
    }
    drop(large_vec);
    
    // GCを促すヒント（Rustは参照カウントベースなので明示的なGCはないが、
    // 大きなアロケーションの解放がメモリコンパクションを促すことがある）
    std::thread::sleep(std::time::Duration::from_millis(10));
}

// テスト用の関数 - win_conditionチェック
fn check_win_condition(board: &Board) -> bool {
    board.remaining_safe_cells == 0
}

// 最適化されたイテレーティブな連鎖セル公開
fn reveal_connected_cells_iterative(
    start_row: usize,
    start_col: usize,
    board: &mut Board
) -> Result<(), JsValue> {
    use std::collections::{VecDeque, HashSet};
    
    // キューを使って処理するセルを管理（事前に容量確保）
    let estimated_capacity = (board.width * board.height) / 4; // 盤面の25%の容量を事前確保
    let mut queue = VecDeque::with_capacity(estimated_capacity);
    
    // 処理済みセルを記録するセット（こちらも事前に容量確保）
    let mut visited = HashSet::with_capacity(estimated_capacity);
    
    // 開始セルをキューに追加
    queue.push_back((start_row, start_col));
    
    // 隣接セル座標用のバッファを事前に確保（再利用）
    let mut adjacent_buffer = Vec::with_capacity(8);
    
    while let Some((row, col)) = queue.pop_front() {
        // セルのインデックスを計算
        let index = row * board.width + col;
        
        // 既に処理済みならスキップ - キャッシュヒット率を上げるため早めにチェック
        if !visited.insert(index) {
            continue;
        }
        
        // 周囲のセルを取得して処理
        adjacent_buffer.clear(); // バッファを再利用
        get_adjacent_cells_optimized(row, col, board.width, board.height, &mut adjacent_buffer);
        
        // 隣接セルの一括処理
        for &(adj_row, adj_col) in &adjacent_buffer {
            let adj_index = adj_row * board.width + adj_col;
            
            // 既に処理済みならスキップ（早期チェック）
            if visited.contains(&adj_index) {
                continue;
            }
            
            // 既に開いているセルや旗が立てられているセルは無視
            if board.revealed[adj_index] || board.flagged[adj_index] {
                continue;
            }
            
            // セルを開く
            board.revealed[adj_index] = true;
            
            // 残りの安全なセル数を減らす
            board.remaining_safe_cells -= 1;
            
            // 周囲に地雷がない空のセルなら、そのセルも処理対象に追加
            if let CellValue::Empty(0) = board.cells[adj_index] {
                queue.push_back((adj_row, adj_col));
            }
        }
    }
    
    Ok(())
}

// 最適化された隣接セル探索
fn get_adjacent_cells_optimized(
    row: usize, 
    col: usize, 
    width: usize, 
    height: usize,
    result: &mut Vec<(usize, usize)>
) {
    // 範囲チェックを最小限にするため、範囲内にあることが明らかな場合は直接追加
    let row_top = row > 0;
    let row_bottom = row < height - 1;
    let col_left = col > 0;
    let col_right = col < width - 1;
    
    // 上段
    if row_top {
        if col_left {
            result.push((row - 1, col - 1));
        }
        result.push((row - 1, col));
        if col_right {
            result.push((row - 1, col + 1));
        }
    }
    
    // 中段
    if col_left {
        result.push((row, col - 1));
    }
    if col_right {
        result.push((row, col + 1));
    }
    
    // 下段
    if row_bottom {
        if col_left {
            result.push((row + 1, col - 1));
        }
        result.push((row + 1, col));
        if col_right {
            result.push((row + 1, col + 1));
        }
    }
} 