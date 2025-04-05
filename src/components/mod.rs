/**
 * ECSパターンのコンポーネント定義
 * 
 * このモジュールではマインスイーパーゲームに必要な
 * 各種コンポーネント（純粋なデータ構造）を定義します
 */

// サブモジュールをエクスポート
pub mod cell;
pub mod player;
pub mod position;
pub mod ui;
pub mod component_trait;
pub mod component_factory;
pub mod component_vec;
pub mod board_components;
pub mod renderable;

// コンポーネントを再エクスポート
pub use cell::{CellContent};
pub use player::Player;
pub use position::Position;
pub use ui::{UIElement, Button};
pub use board_components::{CellStateComponent, CellState as BoardCellState, CellContentComponent, GridPositionComponent};

// コンポーネントシステムを再エクスポート
pub use component_trait::{Component, SerializableComponent, ComponentDependencyHandler};
pub use component_factory::ComponentFactory;
pub use component_vec::ComponentVec;
pub use renderable::RenderType; 