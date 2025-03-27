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

// 既存のリソースを再エクスポート（名前衝突を避けるためにリネーム）
pub use board_config::BoardConfig as OldBoardConfig;
pub use game_state::{GameState, GamePhase as OldGamePhase};
pub use render_state::RenderState;
pub use network_state::NetworkState;

// 新しいECSリソースを公開
pub use core_game::{CoreGameResource, GamePhase};
pub use time::TimeResource;
pub use game_config::{GameConfigResource, BoardConfig, Difficulty};
pub use player_state::{PlayerStateResource, Player as EcsPlayer, MouseState};
pub use resource_manager::{ResourceManager, ResourceBatch, ResourceBatchMut}; 