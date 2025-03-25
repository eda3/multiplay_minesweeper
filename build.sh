#!/bin/bash

echo "Building WASM package... 🦀✨"

# Rustプロジェクトをビルド
cargo build --target wasm32-unknown-unknown --release

# JavaScriptバインディングを生成
echo "Generating JavaScript bindings... 🔄"
wasm-bindgen --target web --out-dir ./pkg/ --no-typescript ./target/wasm32-unknown-unknown/release/wasm_multiplayer.wasm

echo "Done! 🎉"
echo "-------------------------------------"
echo "1. WebSocketサーバーの依存関係をインストール: npm install ws"
echo "2. WebSocketサーバーを起動: node server.js"
echo "3. ローカルHTTPサーバーを起動: python -m http.server"
echo "4. ブラウザで http://localhost:8000 にアクセス"
echo "-------------------------------------" 