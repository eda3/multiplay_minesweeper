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
mod core_game;
mod time;
mod game_config;
mod player_state;
mod resource_manager;

// リソースを再エクスポート
pub use board_config::BoardConfig;
pub use game_state::{GameState, GamePhase};
pub use render_state::RenderState;
pub use network_state::NetworkState;
pub use core_game::{CoreGameResource, GamePhase as CoreGamePhase};
pub use time::TimeResource;
pub use game_config::GameConfigResource;
pub use player_state::PlayerStateResource;
pub use resource_manager::{ResourceManager, ResourceBatch, ResourceBatchMut}; 