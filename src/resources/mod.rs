/**
 * ECSパターンのリソース定義
 * 
 * このモジュールではマインスイーパーゲームに必要なグローバルリソース
 * （シングルトン的な共有データ）を定義します
 */

// サブモジュールをエクスポート
mod board_config;
mod game_state;
mod render_state;
mod network_state;

// リソースを再エクスポート
pub use board_config::BoardConfig;
pub use game_state::{GameState, GamePhase};
pub use render_state::RenderState;
pub use network_state::NetworkState; 