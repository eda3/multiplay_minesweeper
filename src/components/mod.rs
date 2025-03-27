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
mod component_trait;
mod component_factory;
mod component_vec;

// コンポーネントを再エクスポート
pub use cell::{CellContent, CellState};
pub use player::PlayerComponent;
pub use position::Position;
pub use ui::{UIElement, Button};

// コンポーネントシステムを再エクスポート
pub use component_trait::{Component, SerializableComponent, ComponentDependencyHandler};
pub use component_factory::ComponentFactory;
pub use component_vec::ComponentVec; 