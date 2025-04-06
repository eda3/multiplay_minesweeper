# WASM向け最適化

**最終更新日**: 2025-04-06

## 概要

WebAssembly（WASM）向けに最適化を行い、バイナリサイズを削減し、パフォーマンスを向上させました。また、WASM環境とネイティブ環境の両方でコードが動作するようにクロスプラットフォーム対応を強化しました。

## 主な最適化内容

### 1. 条件付きコンパイル

以下の機能をWASM環境と開発環境でのみ有効にすることで、リリースビルドのサイズとパフォーマンスを向上させました：

- **パニックフック**: デバッグビルド時のみパニックフックを有効化
  ```rust
  #[cfg(all(target_arch = "wasm32", debug_assertions))]
  console_error_panic_hook::set_once();
  ```

- **ロギング**: デバッグビルド時のみWASMロガーを初期化
  ```rust
  #[cfg(all(target_arch = "wasm32", debug_assertions))]
  {
      wasm_logger::init(wasm_logger::Config::default());
      log::info!("Wasm logger initialized");
  }
  ```

### 2. プラットフォーム固有の実装分離

WASM環境とネイティブ環境で異なる実装を提供することで、両方の環境でコードが動作するようにしました：

- **Web API使用の分離**:
  ```rust
  #[cfg(target_arch = "wasm32")]
  use web_sys::{...};
  ```

- **イベント処理の条件分岐**:
  ```rust
  // WASM環境用の実装
  #[cfg(target_arch = "wasm32")]
  fn handle_key_down(&mut self, event: &Event) {
      // WASMでの実装
  }
  
  // ネイティブ環境用のスタブ実装
  #[cfg(not(target_arch = "wasm32"))]
  fn setup_event_listeners(&mut self, _input_resource: &mut InputResource) -> Result<(), String> {
      self.initialized = true;
      Ok(())
  }
  ```

- **タイムスタンプ取得の切り替え**:
  ```rust
  #[cfg(target_arch = "wasm32")]
  let timestamp = js_sys::Date::now();
  #[cfg(not(target_arch = "wasm32"))]
  let timestamp = std::time::SystemTime::now()
      .duration_since(std::time::UNIX_EPOCH)
      .unwrap_or_default()
      .as_millis() as f64;
  ```

### 3. ログ出力の最適化

- **コンソール出力の条件分岐**:
  ```rust
  #[cfg(target_arch = "wasm32")]
  web_sys::console::error_1(&format!("エラー: {}", e).into());
  #[cfg(not(target_arch = "wasm32"))]
  println!("エラー: {}", e);
  ```

## 最適化の効果

1. **バイナリサイズの削減**:
   - 最適化前: 約626KB (`wasm_multiplayer.wasm`)
   - 最適化後: 約153KB (`wasm_multiplayer_bg.wasm`)
   - **削減率**: 約75%

2. **パフォーマンス向上**:
   - デバッグビルドでのみパニックフックとロギングを有効化
   - 未使用のコードを条件付きで除外

3. **クロスプラットフォーム対応**:
   - 同じコードベースでWASMとネイティブの両方の環境をサポート
   - テスト環境での実行が容易に

## 今後の改善点

1. **さらなるコードサイズの最適化**:
   - 使用頻度の低い機能のレイジーロード実装検討
   - TreeShakingの強化

2. **パフォーマンス計測**:
   - 最適化の効果を計測するためのベンチマーク追加

3. **条件付きコンパイルの拡大**:
   - さらに多くのWeb固有機能を条件付きコンパイルで分離 