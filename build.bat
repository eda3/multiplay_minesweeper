@echo off
echo Building WASM package...

rem Build for WASM
cargo build --target wasm32-unknown-unknown --release

rem Generate JavaScript bindings with wasm-bindgen
echo Generating JavaScript bindings...
wasm-bindgen --target web --out-dir ./pkg/ --no-typescript ./target/wasm32-unknown-unknown/release/wasm_multiplayer.wasm

echo Done!
echo -------------------------------------
echo 1. Install WebSocket server dependencies: npm install ws
echo 2. Start the WebSocket server: node server.js
echo 3. Start a local HTTP server: python -m http.server
echo 4. Open your browser at http://localhost:8000
echo -------------------------------------
