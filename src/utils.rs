/**
 * ゲーム全体で使用するユーティリティ関数を定義するモジュール
 */
use wasm_bindgen::JsValue;
use web_sys::HtmlCanvasElement;

/**
 * インデックスから行と列の座標を計算する
 */
pub fn index_to_coordinates(index: usize, width: usize) -> (usize, usize) {
    let row = index / width;
    let col = index % width;
    (row, col)
}

/**
 * 行と列の座標からインデックスを計算する
 */
pub fn coordinates_to_index(row: usize, col: usize, width: usize) -> usize {
    row * width + col
}

/**
 * マウス座標からボード上のセルインデックスを計算する
 */
pub fn get_cell_index_from_coordinates(
    x: f64,
    y: f64,
    cell_size: f64,
    board_width: usize,
    board_height: usize
) -> Option<usize> {
    // マウス座標をセルの行と列に変換
    let col = (x / cell_size) as usize;
    let row = (y / cell_size) as usize;
    
    // ボード外の場合はNoneを返す
    if col >= board_width || row >= board_height {
        return None;
    }
    
    Some(row * board_width + col)
}

/**
 * キャンバスのサイズを調整する
 * 
 * ウィンドウサイズに合わせてキャンバスのサイズを調整します。
 */
pub fn resize_canvas(canvas: &HtmlCanvasElement) -> Result<(), JsValue> {
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let body = document.body().unwrap();
    
    // 利用可能な幅と高さを取得
    let available_width = body.client_width() as u32;
    let available_height = body.client_height() as u32;
    
    // キャンバスのサイズを設定
    canvas.set_width(available_width);
    canvas.set_height(available_height);
    
    Ok(())
}

/**
 * 周囲の8方向のオフセットを取得する
 */
pub fn get_adjacent_offsets() -> [(isize, isize); 8] {
    [
        (-1, -1), (-1, 0), (-1, 1),
        (0, -1),           (0, 1),
        (1, -1),  (1, 0),  (1, 1)
    ]
} 