/**
 * ECSパターンのコンポーネント定義
 * 
 * このモジュールではマインスイーパーゲームに必要な
 * 各種コンポーネント（純粋なデータ構造）を定義します
 */

// サブモジュールをエクスポート
mod cell;
mod player;
mod position;
mod ui;

// コンポーネントを再エクスポート
pub use cell::{CellContent, CellState};
pub use player::PlayerComponent;
pub use position::Position;
pub use ui::{UIElement, Button}; 