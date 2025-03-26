/**
 * メイン関数
 * 
 * このファイルはWASMをcrate-typeに含むため必要ですが、
 * 実際のエントリーポイントはlib.rsのstart_game関数です。
 * このmain関数はnative buildの際に呼び出されますが、
 * WebAssembly環境では使われません。
 */
fn main() {
    println!("Hello, world!");
}
