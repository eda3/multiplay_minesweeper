# Rust + WASM マルチプレイヤーゲーム

このプロジェクトは、Rust と WebAssembly を使用したシンプルなマルチプレイヤーゲームです。
WebSocket を利用して複数プレイヤー間で位置情報をリアルタイムに同期します。

## 特徴

- Rust を WebAssembly にコンパイル
- HTML5 Canvas で描画
- WebSocket によるリアルタイム通信
- マウスでプレイヤーを操作
- マウスクリックでスピードアップ

## 必要なもの

- Rust (1.60以上)
- wasm-pack または wasm-bindgen-cli
- Node.js (WebSocketサーバー用)
- Python 3 (ローカルHTTPサーバー用、オプション)

## セットアップ方法

1. 依存パッケージのインストール:

```bash
# WebSocketサーバー用の依存関係をインストール
npm install ws
```

2. WASMにビルド:

```bash
# ビルドスクリプトを実行
./build.bat
```

3. WebSocketサーバーの起動:

```bash
node server.js
```

4. HTTPサーバーの起動:

```bash
# Python 3 の場合
python -m http.server

# Node.js の場合
npx http-server
```

5. ブラウザでアクセス:

ブラウザで http://localhost:8000 (または使用しているHTTPサーバーに合わせたURL) にアクセスしてください。

## 操作方法

- マウスを動かすと、プレイヤーがマウス方向に追従します
- マウスをクリックすると、移動速度が上がります
- 他のブラウザやタブで開くと、別のプレイヤーとして参加できます

## プロジェクト構造

- `src/lib.rs` - Rustのゲームロジック
- `server.js` - Node.js WebSocketサーバー
- `index.html` - ゲームのHTML/CSSとJavaScriptロード
- `pkg/` - ビルド後のWASMファイルとJavaScriptバインディング

## ライセンス

MIT
