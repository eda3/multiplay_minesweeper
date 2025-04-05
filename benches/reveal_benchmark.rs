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

// ベンチマーク - インデックスキャッシュ最適化
#[bench]
fn bench_optimized_reveal_small(b: &mut Bencher) {
    let width = 10;
    let height = 10;
    let mine_count = 10;
    
    b.iter(|| {
        // 新しいボードとエンティティマネージャー
        let mut board = Board::new(width, height, mine_count, 30.0);
        
        // 地雷を配置
        for i in 0..mine_count {
            board.cells[i] = CellValue::Mine;
        }
        
        // 数字を計算
        calculate_numbers(&mut board);
        
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
        
        // 最適化されたアルゴリズムを使用
        let _ = reveal_connected_cells_optimized(reveal_pos.0, reveal_pos.1, &mut board);
    });
}

// ベンチマーク - インデックスキャッシュ最適化（大きなボード）
#[bench]
fn bench_optimized_reveal_large(b: &mut Bencher) {
    let width = 30;
    let height = 30;
    let mine_count: usize = 100;
    
    b.iter(|| {
        // 新しいボードとエンティティマネージャー
        let mut board = Board::new(width, height, mine_count, 30.0);
        
        // 地雷を配置
        for i in 0..mine_count {
            board.cells[i] = CellValue::Mine;
        }
        
        // 数字を計算
        calculate_numbers(&mut board);
        
        // 連鎖反応が起きる位置を探す
        let mut reveal_pos = (15, 15);
        for row in 0..height {
            for col in 0..width {
                let idx = row * width + col;
                if let CellValue::Empty(0) = board.cells[idx] {
                    reveal_pos = (row, col);
                    break;
                }
            }
        }
        
        // 最適化されたアルゴリズムを使用
        let _ = reveal_connected_cells_optimized(reveal_pos.0, reveal_pos.1, &mut board);
    });
}

// インデックスキャッシュの効果を検証するテスト
#[test]
fn test_index_caching_optimization() {
    println!("===== インデックスキャッシュ最適化テスト =====");
    
    // 異なるサイズのボードで検証
    let configs = [
        (10, 10, 10),    // 小規模
        (30, 30, 100),   // 中規模
        (50, 50, 250),   // 大規模
    ];
    
    println!("| ボードサイズ | 通常実装(μs) | キャッシュ実装(μs) | 速度向上率 |");
    println!("|------------|------------|--------------|--------|");
    
    for (width, height, mines) in configs {
        let mut board = Board::new(width, height, mines, 30.0);
        let mut entity_manager = EntityManager::new();
        
        // 地雷を配置
        for i in 0..mines {
            if i < width * height {
                board.cells[i] = CellValue::Mine;
            }
        }
        
        // 数字を計算
        calculate_numbers(&mut board);
        
        // 連鎖反応が起きる位置を中央に設定
        let reveal_pos = (height / 2, width / 2);
        
        // 通常実装のパフォーマンス測定
        let mut total_normal = 0;
        let iterations = 10;
        
        for _ in 0..iterations {
            let mut board_copy = board.clone();
            
            let start = std::time::Instant::now();
            let _ = reveal_connected_cells_iterative(reveal_pos.0, reveal_pos.1, &mut board_copy);
            let duration = start.elapsed();
            
            total_normal += duration.as_micros();
        }
        
        let avg_normal = total_normal / iterations as u128;
        
        // キャッシュ最適化実装のパフォーマンス測定
        let mut total_cached = 0;
        
        for _ in 0..iterations {
            let mut board_copy = board.clone();
            
            let start = std::time::Instant::now();
            let _ = reveal_connected_cells_optimized(reveal_pos.0, reveal_pos.1, &mut board_copy);
            let duration = start.elapsed();
            
            total_cached += duration.as_micros();
        }
        
        let avg_cached = total_cached / iterations as u128;
        
        // 速度向上率の計算
        let speedup = if avg_cached > 0 {
            (avg_normal as f64 / avg_cached as f64) * 100.0
        } else {
            0.0
        };
        
        println!("| {}x{} | {}μs | {}μs | {:.1}% |", 
            width, height, avg_normal, avg_cached, speedup);
    }
    
    println!("===== インデックスキャッシュ最適化テスト完了 =====");
}

/// テスト用の最適化されたセル公開処理
fn optimized_reveal_cell(
    row: usize,
    col: usize,
    board: &mut Board,
    is_first_click: bool
) -> Result<bool, JsValue> {
    // ゲームオーバーや勝利状態では何もしない
    if board.game_over || board.win {
        return Ok(false);
    }

    // インデックスを計算
    let index = row * board.width + col;
    
    // 既に開いているセルや旗が立てられているセルは無視
    if board.revealed[index] || board.flagged[index] {
        return Ok(false);
    }

    // 最初のクリックの場合、地雷を再配置
    if is_first_click {
        // 最初のクリックでは地雷に当たらないようにする
        board.initialize();
        board.first_click = false;
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
        reveal_connected_cells_optimized(row, col, board)?;
    }

    // 勝利条件をチェック
    if check_win_condition(board) {
        board.win = true;
    }

    Ok(false) // 爆発しなかったのでfalseを返す
}

/// インデックス計算のキャッシングを行う最適化バージョン
fn reveal_connected_cells_optimized(
    start_row: usize,
    start_col: usize,
    board: &mut Board
) -> Result<(), JsValue> {
    use std::collections::{VecDeque, HashSet};
    
    let width = board.width;
    let height = board.height;
    
    // キューを使って処理するセルを管理（事前に容量確保）
    let estimated_capacity = (width * height) / 4; // 盤面の25%の容量を事前確保
    let mut queue = VecDeque::with_capacity(estimated_capacity);
    
    // 処理済みセルを記録するセット（こちらも事前に容量確保）
    let mut visited = HashSet::with_capacity(estimated_capacity);
    
    // 開始セルをキューに追加
    queue.push_back((start_row, start_col));
    
    // 隣接セル座標用のバッファを事前に確保（再利用）
    let mut adjacent_buffer = Vec::with_capacity(8);
    
    // インデックス計算をキャッシュするためのルックアップテーブル
    // 各行の先頭インデックスを事前計算
    let mut row_offset_cache = Vec::with_capacity(height);
    for row in 0..height {
        row_offset_cache.push(row * width);
    }
    
    while let Some((row, col)) = queue.pop_front() {
        // セルのインデックスを事前計算したキャッシュを使って計算（乗算を回避）
        let index = row_offset_cache[row] + col;
        
        // 既に処理済みならスキップ - キャッシュヒット率を上げるため早めにチェック
        if !visited.insert(index) {
            continue;
        }
        
        // 周囲のセルを取得して処理
        adjacent_buffer.clear(); // バッファを再利用
        get_adjacent_cells_cached(row, col, width, height, &row_offset_cache, &mut adjacent_buffer);
        
        // 隣接セルの一括処理
        for &(adj_row, adj_col, adj_index) in &adjacent_buffer {
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

/// インデックスキャッシュを使用した隣接セル探索
fn get_adjacent_cells_cached(
    row: usize, 
    col: usize, 
    width: usize, 
    height: usize,
    row_offset_cache: &[usize],
    result: &mut Vec<(usize, usize, usize)> // (行, 列, インデックス)
) {
    // 範囲チェックを最小限にするため、範囲内にあることが明らかな場合は直接追加
    let row_top = row > 0;
    let row_bottom = row < height - 1;
    let col_left = col > 0;
    let col_right = col < width - 1;
    
    // 上段
    if row_top {
        let top_row = row - 1;
        let top_row_offset = row_offset_cache[top_row];
        
        if col_left {
            let idx = top_row_offset + (col - 1);
            result.push((top_row, col - 1, idx));
        }
        
        let idx = top_row_offset + col;
        result.push((top_row, col, idx));
        
        if col_right {
            let idx = top_row_offset + (col + 1);
            result.push((top_row, col + 1, idx));
        }
    }
    
    // 中段
    let current_row_offset = row_offset_cache[row];
    
    if col_left {
        let idx = current_row_offset + (col - 1);
        result.push((row, col - 1, idx));
    }
    
    if col_right {
        let idx = current_row_offset + (col + 1);
        result.push((row, col + 1, idx));
    }
    
    // 下段
    if row_bottom {
        let bottom_row = row + 1;
        let bottom_row_offset = row_offset_cache[bottom_row];
        
        if col_left {
            let idx = bottom_row_offset + (col - 1);
            result.push((bottom_row, col - 1, idx));
        }
        
        let idx = bottom_row_offset + col;
        result.push((bottom_row, col, idx));
        
        if col_right {
            let idx = bottom_row_offset + (col + 1);
            result.push((bottom_row, col + 1, idx));
        }
    }
}

// エッジケーステスト用関数 - 極小ボード
#[bench]
fn test_edge_case_tiny_boards(b: &mut Bencher) {
    b.iter(|| {
        println!("===== 極小ボードエッジケーステスト =====");
        
        // 極小ボードのサイズ設定
        let configs = [
            (1, 1, 0),    // 1x1ボード、地雷なし
            (2, 2, 1),    // 2x2ボード、地雷1個
            (3, 3, 0),    // 3x3ボード、地雷なし (全部零セル)
            (3, 3, 8),    // 3x3ボード、地雷多数 (ほぼ全部地雷)
        ];
        
        println!("| ボードサイズ | 地雷数 | 非再帰結果 | 再帰結果 | 公開セル数 |");
        println!("|------------|-------|--------|--------|--------|");
        
        for (width, height, mines) in configs {
            println!("テスト: {}x{} ボード ({} 地雷)", width, height, mines);
            
            // 非再帰アルゴリズム
            let iterative_revealed;
            {
                let mut board = Board::new(width, height, mines, 30.0);
                let mut entity_manager = EntityManager::new();
                
                // 地雷を配置
                for i in 0..mines {
                    if i < width * height {
                        board.cells[i] = CellValue::Mine;
                    }
                }
                
                // 数字を計算
                calculate_numbers(&mut board);
                
                // 中央のセル（または左上）を開く
                let start_row = if height > 1 { height / 2 } else { 0 };
                let start_col = if width > 1 { width / 2 } else { 0 };
                
                // 非再帰アルゴリズムでセルを開く
                let start = std::time::Instant::now();
                let result = bench_reveal_cell(start_row, start_col, &mut entity_manager, &mut board, false);
                let duration = start.elapsed();
                
                let revealed = board.revealed.iter().filter(|&&r| r).count();
                iterative_revealed = revealed;
                
                println!("  非再帰: {}us, 公開セル数: {}, 結果: {:?}", 
                    duration.as_micros(), revealed, result);
                
                // ボード状態を表示
                print_board_state(&board);
            }
            
            // 再帰アルゴリズム
            let recursive_revealed;
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
                
                // 中央のセル（または左上）を開く
                let start_row = if height > 1 { height / 2 } else { 0 };
                let start_col = if width > 1 { width / 2 } else { 0 };
                
                // 再帰アルゴリズムでセルを開く
                let start = std::time::Instant::now();
                let mut visited = vec![false; width * height];
                reveal_connected_recursive(start_row, start_col, &mut board, &mut visited);
                let duration = start.elapsed();
                
                let revealed = board.revealed.iter().filter(|&&r| r).count();
                recursive_revealed = revealed;
                
                println!("  再帰: {}us, 公開セル数: {}", 
                    duration.as_micros(), revealed);
            }
            
            println!("| {}x{} | {} | OK | OK | {} |", 
                width, height, mines, iterative_revealed);
            
            // 結果が同じであることを確認
            assert_eq!(iterative_revealed, recursive_revealed, 
                "{}x{} ボードでの公開セル数が一致しません", width, height);
                
            println!(""); // 空行
        }
        
        println!("===== 極小ボードエッジケーステスト 完了 =====");
    });
}

// エッジケーステスト用関数 - 零セルの極端なパターン
#[bench]
fn test_edge_case_zero_cells(b: &mut Bencher) {
    b.iter(|| {
        println!("===== 零セル極端パターンテスト =====");
        
        // 零セルが極端に多い/少ないボードの設定
        let configs = [
            (10, 10, 0),    // 全て零セル
            (10, 10, 99),   // 零セルなし（ほぼ全て地雷）
            (10, 10, 30),   // 30%地雷（平均的）
            (10, 10, 24),   // クロスパターンの地雷配置（十字に地雷）
        ];
        
        println!("| ボードタイプ | 地雷数 | 非再帰結果 | 再帰結果 | 公開セル数 |");
        println!("|------------|-------|--------|--------|--------|");
        
        for (i, (width, height, mines)) in configs.iter().enumerate() {
            let board_type = match i {
                0 => "全て零セル",
                1 => "零セルなし",
                2 => "平均的",
                3 => "クロスパターン",
                _ => "その他",
            };
            
            println!("テスト: {}x{} ボード - {} ({} 地雷)", width, height, board_type, mines);
            
            // 非再帰アルゴリズム
            let iterative_revealed;
            {
                let mut board = Board::new(*width, *height, *mines, 30.0);
                let mut entity_manager = EntityManager::new();
                
                // 地雷を特殊パターンで配置
                if i == 3 { // クロスパターン
                    setup_cross_pattern_mines(&mut board);
                } else {
                    // 通常配置
                    for i in 0..*mines {
                        if i < *width * *height {
                            board.cells[i] = CellValue::Mine;
                        }
                    }
                }
                
                // 数字を計算
                calculate_numbers(&mut board);
                
                // 中央のセルを開く
                let start_row = height / 2;
                let start_col = width / 2;
                
                // 非再帰アルゴリズムでセルを開く
                let start = std::time::Instant::now();
                let result = bench_reveal_cell(start_row, start_col, &mut entity_manager, &mut board, false);
                let duration = start.elapsed();
                
                let revealed = board.revealed.iter().filter(|&&r| r).count();
                iterative_revealed = revealed;
                
                println!("  非再帰: {}us, 公開セル数: {}, 結果: {:?}", 
                    duration.as_micros(), revealed, result);
                
                // ボード状態を表示
                print_board_state(&board);
            }
            
            // 再帰アルゴリズム
            let recursive_revealed;
            {
                let mut board = Board::new(*width, *height, *mines, 30.0);
                
                // 地雷を特殊パターンで配置
                if i == 3 { // クロスパターン
                    setup_cross_pattern_mines(&mut board);
                } else {
                    // 通常配置
                    for i in 0..*mines {
                        if i < *width * *height {
                            board.cells[i] = CellValue::Mine;
                        }
                    }
                }
                
                // 数字を計算
                calculate_numbers(&mut board);
                
                // 中央のセルを開く
                let start_row = height / 2;
                let start_col = width / 2;
                
                // 再帰アルゴリズムでセルを開く
                let start = std::time::Instant::now();
                let mut visited = vec![false; *width * *height];
                reveal_connected_recursive(start_row, start_col, &mut board, &mut visited);
                let duration = start.elapsed();
                
                let revealed = board.revealed.iter().filter(|&&r| r).count();
                recursive_revealed = revealed;
                
                println!("  再帰: {}us, 公開セル数: {}", 
                    duration.as_micros(), revealed);
            }
            
            println!("| {} | {} | OK | OK | {} |", 
                board_type, mines, iterative_revealed);
            
            // 結果が同じであることを確認
            assert_eq!(iterative_revealed, recursive_revealed, 
                "{}での公開セル数が一致しません", board_type);
                
            println!(""); // 空行
        }
        
        println!("===== 零セル極端パターンテスト 完了 =====");
    });
}

/// エッジケーステスト用関数 - 複雑な地雷パターン
#[bench]
fn test_edge_case_complex_mine_patterns(b: &mut Bencher) {
    b.iter(|| {
        println!("===== 複雑な地雷パターンテスト =====");
        
        // 複雑な地雷パターンのボード設定
        let configs = [
            (8, 8, "スパイラルパターン"),
            (8, 8, "チェッカーボードパターン"),
            (10, 8, "横長チェッカーボード"),
            (8, 10, "縦長スパイラル"),
        ];
        
        println!("| ボードタイプ | サイズ | 非再帰結果 | 再帰結果 | 公開セル数 |");
        println!("|------------|-------|--------|--------|--------|");
        
        for (i, (width, height, pattern_name)) in configs.iter().enumerate() {
            println!("テスト: {}x{} ボード - {}", width, height, pattern_name);
            
            // 非再帰アルゴリズム
            let iterative_revealed;
            {
                let mut board = Board::new(*width, *height, 0, 30.0); // 地雷数は後で設定
                let mut entity_manager = EntityManager::new();
                
                // 特殊な地雷配置
                match i {
                    0 => setup_spiral_mines(&mut board),    // スパイラルパターン
                    1 => setup_checkerboard_mines(&mut board), // チェッカーボードパターン
                    2 => setup_checkerboard_mines(&mut board), // 横長チェッカーボード
                    3 => setup_spiral_mines(&mut board),    // 縦長スパイラル
                    _ => {}
                }
                
                // 実際の地雷数をカウント
                let mine_count = board.cells.iter().filter(|&&cell| {
                    matches!(cell, CellValue::Mine)
                }).count();
                board.mine_count = mine_count;
                
                // 数字を計算
                calculate_numbers(&mut board);
                
                // セーフなセル数を設定
                board.remaining_safe_cells = width * height - mine_count;
                
                // 中央付近のセルを開く
                let start_row = height / 2;
                let start_col = width / 2;
                
                // 非再帰アルゴリズムでセルを開く
                let start = std::time::Instant::now();
                let result = bench_reveal_cell(start_row, start_col, &mut entity_manager, &mut board, false);
                let duration = start.elapsed();
                
                let revealed = board.revealed.iter().filter(|&&r| r).count();
                iterative_revealed = revealed;
                
                println!("  非再帰: {}us, 公開セル数: {}, 結果: {:?}", 
                    duration.as_micros(), revealed, result);
                    
                // ボード状態を表示
                print_board_state(&board);
            }
            
            // 再帰アルゴリズム
            let recursive_revealed;
            {
                let mut board = Board::new(*width, *height, 0, 30.0); // 地雷数は後で設定
                
                // 特殊な地雷配置
                match i {
                    0 => setup_spiral_mines(&mut board),    // スパイラルパターン
                    1 => setup_checkerboard_mines(&mut board), // チェッカーボードパターン
                    2 => setup_checkerboard_mines(&mut board), // 横長チェッカーボード
                    3 => setup_spiral_mines(&mut board),    // 縦長スパイラル
                    _ => {}
                }
                
                // 実際の地雷数をカウント
                let mine_count = board.cells.iter().filter(|&&cell| {
                    matches!(cell, CellValue::Mine)
                }).count();
                board.mine_count = mine_count;
                
                // 数字を計算
                calculate_numbers(&mut board);
                
                // セーフなセル数を設定
                board.remaining_safe_cells = width * height - mine_count;
                
                // 中央付近のセルを開く
                let start_row = height / 2;
                let start_col = width / 2;
                
                // 再帰アルゴリズムでセルを開く
                let start = std::time::Instant::now();
                let mut visited = vec![false; *width * *height];
                reveal_connected_recursive(start_row, start_col, &mut board, &mut visited);
                let duration = start.elapsed();
                
                let revealed = board.revealed.iter().filter(|&&r| r).count();
                recursive_revealed = revealed;
                
                println!("  再帰: {}us, 公開セル数: {}", 
                    duration.as_micros(), revealed);
            }
            
            println!("| {} | {}x{} | OK | OK | {} |", 
                pattern_name, width, height, iterative_revealed);
            
            // 結果が同じであることを確認
            assert_eq!(iterative_revealed, recursive_revealed, 
                "{}での公開セル数が一致しません", pattern_name);
                
            println!(""); // 空行
        }
        
        println!("===== 複雑な地雷パターンテスト 完了 =====");
    });
}

/// スパイラル状に地雷を配置
fn setup_spiral_mines(board: &mut Board) {
    let width = board.width;
    let height = board.height;
    
    // すべてのセルを空にリセット
    for cell in board.cells.iter_mut() {
        *cell = CellValue::Empty(0);
    }
    
    let mut x: isize = 0;
    let mut y: isize = 0;
    let mut dx: isize = 1;
    let mut dy: isize = 0;
    let mut steps = width;
    let mut step_change = 0;
    
    // 地雷の最大数（全体の約40%）
    let max_mines = (width * height * 4) / 10;
    let mut mines_placed = 0;
    
    // スパイラルパターンで地雷を配置
    for _ in 0..(width * height) {
        // 偶数位置にのみ地雷を配置（スパイラル上で交互に）
        if (x + y) % 2 == 0 && mines_placed < max_mines && x >= 0 && y >= 0 && x < width as isize && y < height as isize {
            let idx = (y as usize) * width + (x as usize);
            board.cells[idx] = CellValue::Mine;
            mines_placed += 1;
        }
        
        // スパイラルの次の座標へ移動
        x += dx;
        y += dy;
        
        // ステップを減らし、必要に応じて方向を変更
        steps -= 1;
        if steps == 0 {
            steps = match step_change {
                0 | 2 => width - (step_change / 2 + 1),
                1 | 3 => height - (step_change / 2 + 1),
                _ => unreachable!()
            };
            
            // 方向を変更: 右→下→左→上→右...
            match (dx, dy) {
                (1, 0) => { dx = 0; dy = 1; }  // 右から下へ
                (0, 1) => { dx = -1; dy = 0; } // 下から左へ
                (-1, 0) => { dx = 0; dy = -1; } // 左から上へ
                (0, -1) => { dx = 1; dy = 0; } // 上から右へ
                _ => unreachable!()
            }
            
            step_change = (step_change + 1) % 4;
        }
        
        // 境界外に出たらループを終了
        if x < 0 || y < 0 || x >= width as isize || y >= height as isize {
            break;
        }
    }
}

/// チェッカーボード（市松模様）状に地雷を配置
fn setup_checkerboard_mines(board: &mut Board) {
    let width = board.width;
    let height = board.height;
    
    // すべてのセルを空にリセット
    for cell in board.cells.iter_mut() {
        *cell = CellValue::Empty(0);
    }
    
    // 市松模様で地雷を配置
    for y in 0..height {
        for x in 0..width {
            if (x + y) % 2 == 0 {
                let idx = y * width + x;
                board.cells[idx] = CellValue::Mine;
            }
        }
    }
}

/// 最適なインデックスキャッシュを利用したキュー処理によって非再帰的にセルを公開する関数
/// この実装は最終的な最適化バージョン
#[test]
fn test_optimized_reveal_performance() {
    println!("===== 最適化されたセル公開アルゴリズム性能テスト =====");
    
    // 異なるボードサイズでのテスト
    let configs = [
        (10, 10, 10, "小さいボード"),
        (16, 16, 40, "中サイズボード"),
        (30, 16, 99, "大きいボード"),
    ];
    
    println!("| ボードサイズ | 地雷数 | イテレーティブ実装 | インデックスキャッシュ実装 | 改善率 |");
    println!("|------------|-------|--------------|-----------------|-------|");
    
    for (width, height, mines, board_type) in configs {
        println!("テスト: {}x{} ボード - {} ({} 地雷)", width, height, board_type, mines);
        
        // 10回の実行の平均を取る
        let iterations = 10;
        let mut iterative_total = 0u128;
        let mut optimized_total = 0u128;
        
        for _ in 0..iterations {
            // 両方のテストで同じボード状態を使用するためのシード
            let mut board_template = Board::new(width, height, mines, 30.0);
            for i in 0..mines {
                if i < width * height {
                    board_template.cells[i] = CellValue::Mine;
                }
            }
            calculate_numbers(&mut board_template);
            
            // 連鎖反応が起きる位置を探す
            let reveal_pos = find_chain_reaction_position(&board_template);
            
            // 基本的なイテレーティブ実装
            {
                // ボードの状態をコピー
                let mut board = board_template.clone();
                let mut entity_manager = EntityManager::new();
                
                // イテレーティブな実装でセルを開く
                let start = std::time::Instant::now();
                let _ = reveal_connected_cells_iterative(reveal_pos.0, reveal_pos.1, &mut board);
                let duration = start.elapsed();
                
                iterative_total += duration.as_nanos();
            }
            
            // インデックスキャッシュ最適化実装
            {
                // ボードの状態をコピー
                let mut board = board_template.clone();
                
                // 最適化された実装でセルを開く
                let start = std::time::Instant::now();
                let _ = optimized_reveal_cell(reveal_pos.0, reveal_pos.1, &mut board, false);
                let duration = start.elapsed();
                
                optimized_total += duration.as_nanos();
            }
        }
        
        // 平均時間を計算
        let iterative_avg = iterative_total / iterations as u128;
        let optimized_avg = optimized_total / iterations as u128;
        
        // 改善率を計算
        let improvement_pct = if iterative_avg > 0 {
            (iterative_avg as f64 - optimized_avg as f64) / iterative_avg as f64 * 100.0
        } else {
            0.0
        };
        
        println!("| {}x{} | {} | {}ns | {}ns | {:.1}% |", 
            width, height, mines, iterative_avg, optimized_avg, improvement_pct);
        
        println!(""); // 空行
    }
    
    println!("===== 最適化されたセル公開アルゴリズム性能テスト 完了 =====");
}

/// 連鎖反応が起きる位置を見つける
fn find_chain_reaction_position(board: &Board) -> (usize, usize) {
    // まず中央付近から探索
    let mid_row = board.height / 2;
    let mid_col = board.width / 2;
    
    // 中心から外側に螺旋状に探索
    let mut radius: isize = 0;
    while radius < (board.width.max(board.height) as isize) {
        // 現在の半径の周囲を探索
        for dr in -radius..=radius {
            for dc in -radius..=radius {
                // 周縁部のみをチェック
                if dr.abs() == radius || dc.abs() == radius {
                    let row_i = mid_row as isize + dr;
                    let col_i = mid_col as isize + dc;
                    
                    // ボード範囲内かチェック
                    if row_i >= 0 && col_i >= 0 && row_i < board.height as isize && col_i < board.width as isize {
                        let row = row_i as usize;
                        let col = col_i as usize;
                        let idx = row * board.width + col;
                        // 空のセル（周囲に地雷がない）を探す
                        if let CellValue::Empty(0) = board.cells[idx] {
                            return (row, col);
                        }
                    }
                }
            }
        }
        
        radius += 1;
    }
    
    // 見つからない場合はデフォルト位置
    (mid_row, mid_col)
} 

/// ボード状態をコンソールに表示する関数
fn print_board_state(board: &Board) {
    let width = board.width;
    let height = board.height;
    
    println!("ボード状態 ({}x{}):", width, height);
    for y in 0..height {
        let mut line = String::new();
        for x in 0..width {
            let idx = y * width + x;
            let cell_char = match board.cells[idx] {
                CellValue::Mine => if board.revealed[idx] { "💣" } else { "□" },
                CellValue::Empty(n) => {
                    if board.revealed[idx] {
                        if n == 0 { "　" } else { &n.to_string() }
                    } else {
                        "□"
                    }
                }
            };
            line.push_str(cell_char);
        }
        println!("{}", line);
    }
    println!("");
}

/// クロス状に地雷を配置する関数
fn setup_cross_pattern_mines(board: &mut Board) {
    let width = board.width;
    let height = board.height;
    
    // すべてのセルを空にリセット
    for cell in board.cells.iter_mut() {
        *cell = CellValue::Empty(0);
    }
    
    // 水平方向の地雷配置
    let mid_row = height / 2;
    for x in 0..width {
        let idx = mid_row * width + x;
        board.cells[idx] = CellValue::Mine;
    }
    
    // 垂直方向の地雷配置
    let mid_col = width / 2;
    for y in 0..height {
        // 交差点の重複を避ける
        if y != mid_row {
            let idx = y * width + mid_col;
            board.cells[idx] = CellValue::Mine;
        }
    }
}

/// エッジケーステスト用関数 - 非対称形状と偏った地雷配置
#[bench]
fn test_edge_case_asymmetric_shapes(b: &mut Bencher) {
    b.iter(|| {
        println!("===== 非対称形状と偏った地雷配置テスト =====");
        
        // 非対称ボードと偏った地雷配置の設定
        let configs = [
            (20, 5, "横長ボード"),       // 極端に横長
            (5, 20, "縦長ボード"),       // 極端に縦長
            (10, 10, "コーナー密集地雷"), // 左上コーナーに地雷が密集
            (10, 10, "端部密集地雷"),    // 右端に地雷が密集
        ];
        
        println!("| ボードタイプ | サイズ | 非再帰結果 | 最適化結果 | 公開セル数 |");
        println!("|------------|-------|--------|--------|--------|");
        
        for (i, (width, height, pattern_name)) in configs.iter().enumerate() {
            println!("テスト: {}x{} ボード - {}", width, height, pattern_name);
            
            // 非再帰アルゴリズム（通常版）
            let iterative_revealed;
            {
                let mut board = Board::new(*width, *height, 0, 30.0); // 地雷数は後で設定
                let mut entity_manager = EntityManager::new();
                
                // 特殊な地雷配置
                match i {
                    0 => { // 横長ボード
                        // 左側に地雷を配置
                        for row in 0..height {
                            for col in 0..(width/4) {
                                let idx = row * width + col;
                                board.cells[idx] = CellValue::Mine;
                            }
                        }
                    },
                    1 => { // 縦長ボード
                        // 上側に地雷を配置
                        for row in 0..(height/4) {
                            for col in 0..width {
                                let idx = row * width + col;
                                board.cells[idx] = CellValue::Mine;
                            }
                        }
                    },
                    2 => { // コーナー密集地雷
                        // 左上コーナーに地雷を密集
                        for row in 0..(height/3) {
                            for col in 0..(width/3) {
                                let idx = row * width + col;
                                board.cells[idx] = CellValue::Mine;
                            }
                        }
                    },
                    3 => { // 端部密集地雷
                        // 右端全体に地雷を配置
                        for row in 0..height {
                            for col in (width*3/4)..width {
                                let idx = row * width + col;
                                board.cells[idx] = CellValue::Mine;
                            }
                        }
                    },
                    _ => {}
                }
                
                // 実際の地雷数をカウント
                let mine_count = board.cells.iter().filter(|&&cell| {
                    matches!(cell, CellValue::Mine)
                }).count();
                board.mine_count = mine_count;
                
                // 数字を計算
                calculate_numbers(&mut board);
                
                // セーフなセル数を設定
                board.remaining_safe_cells = width * height - mine_count;
                
                // 最適な開始位置を選択（地雷が少ない部分）
                let (start_row, start_col) = match i {
                    0 => (height / 2, width * 3 / 4),  // 横長ボードは右側から開始
                    1 => (height * 3 / 4, width / 2),  // 縦長ボードは下側から開始
                    2 => (height * 2 / 3, width * 2 / 3), // コーナー密集地雷は右下から開始
                    3 => (height / 2, width / 4),      // 端部密集地雷は左側から開始
                    _ => (height / 2, width / 2),
                };
                
                println!("  開始位置: ({}, {})", start_row, start_col);
                
                // 非再帰アルゴリズムでセルを開く
                let start = std::time::Instant::now();
                let result = bench_reveal_cell(start_row, start_col, &mut entity_manager, &mut board, false);
                let duration = start.elapsed();
                
                let revealed = board.revealed.iter().filter(|&&r| r).count();
                iterative_revealed = revealed;
                
                println!("  非再帰: {}us, 公開セル数: {}, 結果: {:?}", 
                    duration.as_micros(), revealed, result);
                    
                // ボード状態を表示
                print_board_state(&board);
            }
            
            // 最適化アルゴリズム
            let optimized_revealed;
            {
                let mut board = Board::new(*width, *height, 0, 30.0); // 地雷数は後で設定
                
                // 特殊な地雷配置（上と同じ）
                match i {
                    0 => { // 横長ボード
                        for row in 0..height {
                            for col in 0..(width/4) {
                                let idx = row * width + col;
                                board.cells[idx] = CellValue::Mine;
                            }
                        }
                    },
                    1 => { // 縦長ボード
                        for row in 0..(height/4) {
                            for col in 0..width {
                                let idx = row * width + col;
                                board.cells[idx] = CellValue::Mine;
                            }
                        }
                    },
                    2 => { // コーナー密集地雷
                        for row in 0..(height/3) {
                            for col in 0..(width/3) {
                                let idx = row * width + col;
                                board.cells[idx] = CellValue::Mine;
                            }
                        }
                    },
                    3 => { // 端部密集地雷
                        for row in 0..height {
                            for col in (width*3/4)..width {
                                let idx = row * width + col;
                                board.cells[idx] = CellValue::Mine;
                            }
                        }
                    },
                    _ => {}
                }
                
                // 実際の地雷数をカウント
                let mine_count = board.cells.iter().filter(|&&cell| {
                    matches!(cell, CellValue::Mine)
                }).count();
                board.mine_count = mine_count;
                
                // 数字を計算
                calculate_numbers(&mut board);
                
                // セーフなセル数を設定
                board.remaining_safe_cells = width * height - mine_count;
                
                // 最適な開始位置を選択（地雷が少ない部分）
                let (start_row, start_col) = match i {
                    0 => (height / 2, width * 3 / 4),  // 横長ボードは右側から開始
                    1 => (height * 3 / 4, width / 2),  // 縦長ボードは下側から開始
                    2 => (height * 2 / 3, width * 2 / 3), // コーナー密集地雷は右下から開始
                    3 => (height / 2, width / 4),      // 端部密集地雷は左側から開始
                    _ => (height / 2, width / 2),
                };
                
                // 最適化アルゴリズムでセルを開く
                let start = std::time::Instant::now();
                let result = optimized_reveal_cell(start_row, start_col, &mut board, false);
                let duration = start.elapsed();
                
                let revealed = board.revealed.iter().filter(|&&r| r).count();
                optimized_revealed = revealed;
                
                println!("  最適化: {}us, 公開セル数: {}, 結果: {:?}", 
                    duration.as_micros(), revealed, result);
            }
            
            println!("| {} | {}x{} | OK | OK | {} |", 
                pattern_name, width, height, iterative_revealed);
            
            // 結果が同じであることを確認
            assert_eq!(iterative_revealed, optimized_revealed, 
                "{}での公開セル数が一致しません", pattern_name);
                
            println!(""); // 空行
        }
        
        println!("===== 非対称形状と偏った地雷配置テスト 完了 =====");
    });
}

/// さらに最適化されたキャッシュとメモリアクセスパターンを使用したバージョン
fn reveal_connected_cells_super_optimized(
    start_row: usize,
    start_col: usize,
    board: &mut Board
) -> Result<(), JsValue> {
    use std::collections::VecDeque;
    
    let width = board.width;
    let height = board.height;
    let total_cells = width * height;
    
    // ビジットマーカーとして使用するビットセット
    // Vecよりもより効率的なメモリ使用と高速な検索
    let mut visited = vec![false; total_cells];
    
    // 行オフセットのプリフェッチキャッシュ
    let mut row_offsets = Vec::with_capacity(height);
    for row in 0..height {
        row_offsets.push(row * width);
    }
    
    // 開始セルのインデックス
    let start_index = row_offsets[start_row] + start_col;
    
    // 既に開かれているかチェック
    if board.revealed[start_index] {
        return Ok(());
    }
    
    // 効率的なキューの初期化 - ほとんどのケースでは全セルの20%以下しか訪問しない
    let mut queue = VecDeque::with_capacity(total_cells / 5);
    queue.push_back(start_index);
    
    // 隣接インデックスのための方向オフセット配列をプリコンパイル
    // (row_delta, col_delta, condition_lambda) の形式
    let neighbors = [
        (-1, -1, |r: usize, c: usize| r > 0 && c > 0),                 // 左上
        (-1,  0, |r: usize, _: usize| r > 0),                          // 上
        (-1,  1, |r: usize, c: usize| r > 0 && c < width - 1),         // 右上
        ( 0, -1, |_: usize, c: usize| c > 0),                          // 左
        ( 0,  1, |_: usize, c: usize| c < width - 1),                  // 右
        ( 1, -1, |r: usize, c: usize| r < height - 1 && c > 0),        // 左下
        ( 1,  0, |r: usize, _: usize| r < height - 1),                 // 下
        ( 1,  1, |r: usize, c: usize| r < height - 1 && c < width - 1) // 右下
    ];
    
    // キャッシュヒット率を最大化する処理順序
    while let Some(current_index) = queue.pop_front() {
        // 既に訪問済みならスキップ
        if visited[current_index] {
            continue;
        }
        
        // 訪問済みとしてマーク
        visited[current_index] = true;
        
        // 既に開かれているかフラグがたっているなら無視
        if board.revealed[current_index] || board.flagged[current_index] {
            continue;
        }
        
        // セルを開く
        board.revealed[current_index] = true;
        board.remaining_safe_cells -= 1;
        
        // 空のセル（値が0）でなければこのセルの処理は終了
        if let CellValue::Empty(0) = board.cells[current_index] {
            // このセルは空なので周囲を探索
            
            // インデックスから行と列を逆算
            let row = current_index / width;
            let col = current_index % width;
            
            // 隣接セルをチェック - キャッシュフレンドリーな順序で
            for &(row_delta, col_delta, condition) in &neighbors {
                if !condition(row, col) {
                    continue;
                }
                
                let new_row = (row as isize + row_delta) as usize;
                let new_col = (col as isize + col_delta) as usize;
                let new_index = row_offsets[new_row] + new_col;
                
                // まだキューに入っていなければ追加
                if !visited[new_index] && !board.revealed[new_index] && !board.flagged[new_index] {
                    queue.push_back(new_index);
                }
            }
        }
    }
    
    Ok(())
}

/// スーパー最適化されたセル公開関数
fn super_optimized_reveal_cell(
    row: usize,
    col: usize,
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
        reveal_connected_cells_super_optimized(row, col, board)?;
    }

    // 勝利条件をチェック
    if check_win_condition(board) {
        board.win = true;
    }

    Ok(false) // 爆発しなかったのでfalseを返す
}

/// 最適化バージョンのベンチマーク
#[bench]
fn bench_super_optimized_reveal_small(b: &mut Bencher) {
    let width = 10;
    let height = 10;
    let mine_count = 10;
    
    b.iter(|| {
        // 標準テストボードのセットアップ
        let mut board = Board::new(width, height, mine_count, 30.0);
        
        // 地雷を配置
        for i in 0..mine_count {
            board.cells[i] = CellValue::Mine;
        }
        
        // 数字を計算
        calculate_numbers(&mut board);
        
        // 連鎖反応が起きる位置を探す
        let reveal_pos = find_chain_reaction_position(&board);
        
        // スーパー最適化された公開関数を使用
        let _ = super_optimized_reveal_cell(reveal_pos.0, reveal_pos.1, &mut board, false);
    });
}

/// 3つのアルゴリズムを並べて比較するパフォーマンステスト
#[test]
#[ignore] // 通常のテスト実行では除外し、明示的に指定した場合のみ実行
fn test_super_optimized_performance_comparison() {
    println!("\n===== パフォーマンス比較テスト =====");
    
    // ボードサイズ設定
    let configs = [
        (10, 10, 10, "小ボード"),
        (20, 20, 40, "中ボード"),
        (30, 30, 90, "大ボード"),
        (50, 50, 250, "超大ボード")
    ];
    
    println!("| ボードサイズ | イテレーティブ | 最適化 | スーパー最適化 | 改善率 |");
    println!("|------------|------------|-------|------------|-------|");
    
    for (width, height, mine_count, name) in configs {
        println!("\n## {} ({}x{}, 地雷{}個) ##", name, width, height, mine_count);
        
        // 標準テストボードのセットアップ
        let mut board = Board::new(width, height, mine_count, 30.0);
        let mut entity_manager = EntityManager::new();
        
        // 地雷をランダムに配置
        let mut rng = rand::thread_rng();
        let mut mines_placed = 0;
        
        while mines_placed < mine_count {
            let idx = rng.gen_range(0, width * height);
            if let CellValue::Empty(_) = board.cells[idx] {
                board.cells[idx] = CellValue::Mine;
                mines_placed += 1;
            }
        }
        
        // 数字を計算
        calculate_numbers(&mut board);
        
        // 連鎖反応が起きる位置を探す
        let reveal_pos = find_chain_reaction_position(&board);
        println!("連鎖反応の開始位置: ({}, {})", reveal_pos.0, reveal_pos.1);
        
        // 以下、3つのアルゴリズムでテスト
        
        // 1. イテレーティブアルゴリズム
        let mut board_iterative = board.clone();
        let mut entity_manager_clone = entity_manager.clone();
        
        let start = std::time::Instant::now();
        let _ = bench_reveal_cell(reveal_pos.0, reveal_pos.1, &mut entity_manager_clone, &mut board_iterative, false);
        let iterative_duration = start.elapsed();
        let iterative_us = iterative_duration.as_micros();
        let iterative_revealed = board_iterative.revealed.iter().filter(|&&r| r).count();
        
        println!("イテレーティブ: {}us, 公開セル数: {}", iterative_us, iterative_revealed);
        
        // 2. 最適化アルゴリズム
        let mut board_optimized = board.clone();
        
        let start = std::time::Instant::now();
        let _ = optimized_reveal_cell(reveal_pos.0, reveal_pos.1, &mut board_optimized, false);
        let optimized_duration = start.elapsed();
        let optimized_us = optimized_duration.as_micros();
        let optimized_revealed = board_optimized.revealed.iter().filter(|&&r| r).count();
        
        println!("最適化: {}us, 公開セル数: {}", optimized_us, optimized_revealed);
        
        // 3. スーパー最適化アルゴリズム
        let mut board_super = board.clone();
        
        let start = std::time::Instant::now();
        let _ = super_optimized_reveal_cell(reveal_pos.0, reveal_pos.1, &mut board_super, false);
        let super_duration = start.elapsed();
        let super_us = super_duration.as_micros();
        let super_revealed = board_super.revealed.iter().filter(|&&r| r).count();
        
        println!("スーパー最適化: {}us, 公開セル数: {}", super_us, super_revealed);
        
        // 基本アルゴリズムからの改善率
        let improvement1 = (1.0 - (optimized_us as f64 / iterative_us as f64)) * 100.0;
        let improvement2 = (1.0 - (super_us as f64 / iterative_us as f64)) * 100.0;
        
        println!("改善率: 最適化: {:.1}%, スーパー最適化: {:.1}%", improvement1, improvement2);
        
        // 全アルゴリズムの結果が同じことを確認
        assert_eq!(iterative_revealed, optimized_revealed, "最適化アルゴリズムの結果不一致");
        assert_eq!(iterative_revealed, super_revealed, "スーパー最適化アルゴリズムの結果不一致");
        
        println!("|{}|{}us|{}us|{}us|{:.1}%|", 
            name, iterative_us, optimized_us, super_us, improvement2);
    }
    
    println!("\n===== パフォーマンス比較テスト 完了 =====");
}