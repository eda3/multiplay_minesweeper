/**
 * 座標関連の型定義
 * @packageDocumentation
 */

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
export function createCoordinate(row: number, col: number): Coordinate;

/**
 * 座標が等しいかどうかを判定
 * 
 * @param a 座標A
 * @param b 座標B
 * @returns 等しい場合はtrue
 */
export function coordsEqual(a: Coordinate, b: Coordinate): boolean;

/**
 * 座標からハッシュ値を計算（高速な検索用）
 * 
 * @param coord 座標
 * @returns ハッシュ値
 */
export function coordHash(coord: Coordinate): string;

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
    includeDiagonals?: boolean
): Coordinate[]; 