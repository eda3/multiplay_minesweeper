/**
 * 座標を表す型定義
 */
export interface Coordinate {
    /**
     * 行（縦位置）
     */
    row: number;

    /**
     * 列（横位置）
     */
    col: number;
}

/**
 * 座標を作成する
 * 
 * @param row 行（縦位置）
 * @param col 列（横位置）
 * @returns 座標オブジェクト
 */
export function createCoordinate(row: number, col: number): Coordinate {
    return { row, col };
}

/**
 * 座標が等しいかどうかを判定
 * 
 * @param a 座標A
 * @param b 座標B
 * @returns 等しい場合はtrue
 */
export function coordsEqual(a: Coordinate, b: Coordinate): boolean {
    return a.row === b.row && a.col === b.col;
}

/**
 * 座標からハッシュ値を計算（高速な検索用）
 * 
 * @param coord 座標
 * @returns ハッシュ値
 */
export function coordHash(coord: Coordinate): string {
    return `${coord.row}:${coord.col}`;
}

/**
 * 座標の配列から隣接する座標を取得
 * 
 * @param coord 基準となる座標
 * @param boardHeight ボードの高さ
 * @param boardWidth ボードの幅
 * @param includeDiagonals 斜め方向も含めるかどうか
 * @returns 隣接する座標の配列
 */
export function getAdjacentCoordinates(
    coord: Coordinate,
    boardHeight: number,
    boardWidth: number,
    includeDiagonals: boolean = true
): Coordinate[] {
    const { row, col } = coord;
    const adjacent: Coordinate[] = [];

    // 周囲8方向の座標を調べる
    const directions = includeDiagonals ? [
        [-1, -1], [-1, 0], [-1, 1],
        [0, -1], [0, 1],
        [1, -1], [1, 0], [1, 1]
    ] : [
        [-1, 0],
        [0, -1], [0, 1],
        [1, 0]
    ];

    for (const [dRow, dCol] of directions) {
        const newRow = row + dRow;
        const newCol = col + dCol;

        // ボードの範囲内かどうかをチェック
        if (newRow >= 0 && newRow < boardHeight && newCol >= 0 && newCol < boardWidth) {
            adjacent.push(createCoordinate(newRow, newCol));
        }
    }

    return adjacent;
} 