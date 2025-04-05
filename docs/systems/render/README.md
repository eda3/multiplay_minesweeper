# レンダリングシステム

## 概要

レンダリングシステムは、マインスイーパーゲームの視覚的表現を担当するコンポーネントです。HTML5のCanvas APIを使用して、ゲーム状態、ゲームボード、セル、地雷、フラグ、およびユーザーインターフェース要素を描画します。

## 責任範囲

- ゲームボードの描画
- 個々のセルの状態表示
- 旗と地雷のアイコン表示
- ゲームオーバー画面の表示
- タイマーとスコア表示
- マルチプレイヤー情報の表示

## 主要な関数

### `render_system`

ゲームの各フレームで呼び出されるメイン関数です。レンダリングリソースとゲーム状態を取得し、適切な描画関数を呼び出します。

```rust
fn render_system(world: &mut World) -> Result<(), JsValue> {
    let render = world.get_resource::<RenderResource>()?;
    let game_state = world.get_resource::<GameStateResource>()?;
    
    // ゲーム状態に基づいて描画する
    match game_state.state {
        GameState::Playing => {
            render_game_board(world)?;
        }
        GameState::GameOver => {
            render_game_over_screen(world)?;
        }
        // 他の状態...
    }
    
    Ok(())
}
```

### `render_game_board`

ゲームボード全体を描画します。盤面のグリッド、セル、地雷、フラグを含みます。

### `render_game_over_screen`

ゲームオーバー時の画面を描画します。最近の修正では、引数の型を参照型から値型に変換しました。

変更前:
```rust
fn render_game_over_screen(
    context: &CanvasRenderingContext2d,
    render: &RenderResource,
    win: &bool,
    score: &u32,
    time: &Duration
) -> Result<(), JsValue> {
    // ...
}
```

変更後:
```rust
fn render_game_over_screen(
    context: &CanvasRenderingContext2d,
    render: &RenderResource,
    win: bool,
    score: u32,
    time: Duration
) -> Result<(), JsValue> {
    // ...
}
```

呼び出し側での変更:
```rust
// 変更前
render_game_over_screen(context, render, win, score, time);

// 変更後 - 参照外しによる値渡し
render_game_over_screen(context, render, *win, *score, *time);
```

## 色パレット

ゲームは一貫した色パレットを使用して視覚的な統一感を提供します：

```rust
const CELL_BACKGROUND: &str = "#DDDDDD";
const CELL_REVEALED: &str = "#BBBBBB";
const CELL_BORDER: &str = "#999999";
const MINE_COLOR: &str = "#FF0000";
const FLAG_COLOR: &str = "#FF9900";
const TEXT_COLOR: &str = "#000000";
const GAME_OVER_BG: &str = "rgba(0, 0, 0, 0.7)";
const GAME_OVER_TEXT: &str = "#FFFFFF";
```

## パフォーマンス考慮事項

- レンダリングループは必要なときだけ描画操作を実行するように設計
- 単純な実装として、毎フレーム全画面を再描画
- 将来的には差分描画の実装を検討

## 将来の改善点

1. **レスポンシブデザイン**：画面サイズに応じたゲームボードのスケーリング
2. **アニメーション効果**：セル開示やゲームオーバー時のスムーズなアニメーション
3. **テーマサポート**：ダークモード/ライトモードなど異なる視覚テーマ
4. **パフォーマンス最適化**：変更された部分のみを再描画
5. **アクセシビリティ対応**：色覚異常対応とキーボードナビゲーション

## 最近の変更点

### 2025-04-05: `render_game_over_screen` 関数の参照外し修正

`render_game_over_screen` 関数の引数を参照型から値型に変更しました。これにより、関数呼び出し時に引数を参照外しする必要が生じましたが、型の一貫性が向上し、コンパイラエラーが解消されました。

```rust
// 修正前のコード
render_game_over_screen(context, render, win, score, time);

// 修正後のコード - 参照外し演算子 * を使用
render_game_over_screen(context, render, *win, *score, *time);
```

この変更は、関数シグネチャと関数呼び出しの間の型の不一致を解消するために行われました。 