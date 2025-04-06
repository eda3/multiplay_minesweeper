/**
 * リソースモジュール
 * 
 * ECSパターンにおけるリソース（システム間で共有されるグローバルなデータ）を定義します
 */

// サブモジュールをエクスポート
pub mod board_resources;
pub mod resource_trait;
pub mod player_state;
pub mod time_resource;
pub mod event_queue_resource;
pub mod network_state;
pub mod render_state;
pub mod resource_manager;
pub mod draw_options;
pub mod event_bus_resource;
pub mod typed_event_bus_resource;

// リソースを再エクスポート
pub use resource_trait::Resource;
pub use resource_manager::ResourceManager;
pub use board_resources::{BoardConfigResource, BoardStateResource};
pub use player_state::PlayerStateResource;
pub use time_resource::TimeResource;
pub use event_queue_resource::{EventQueueResource, GameEvent};
pub use network_state::NetworkResource;
pub use render_state::RenderResource;
pub use draw_options::DrawOptions;
pub use event_bus_resource::EventBusResource;
pub use typed_event_bus_resource::TypedEventBusResource;

// モデルをインポート
// pub use board_state::{CellState, CellValue, Board, BoardConfig}; // board_state からは CellState, BoardConfig のみ使うか、正しいパスからインポート
pub use board_state::{CellState, BoardConfig};

pub use crate::models::cell::CellValue;
// pub use crate::board::Board; // BoardResource エイリアスを使うので不要か？

// 型エイリアス
pub type BoardResource = crate::board::Board;

// レガシーな定義（互換性のため）
pub mod board_state;
pub mod board_config;
pub mod core_game;
pub mod time;
pub mod input_resource;
pub mod mouse_state;
pub mod game_state;
pub mod game_config;
pub mod resource_impl;

// pub use board_state::{CellState, CellValue, Board, BoardConfig}; // 再度コメントアウト
// pub use board_config::BoardConfig as OldBoardConfig; // 削除
// pub use core_game::{GamePhase, CoreGameResource}; // CoreGameResource は core_game にない可能性
// pub use game_state::{GameStateResource, DifficultyLevel}; // game_state からは何も使わないか、core_game からインポート
// pub use core_game::{GameStateResource, DifficultyLevel}; // GameStateResource, DifficultyLevel も core_game にない可能性

// 正しいと思われるパスからインポート
pub use crate::resources::core_game::{GamePhase, DifficultyLevel}; // CoreGameResource を削除
pub use crate::resources::core_game::GameStateResource; 
// pub use crate::resources::core_game::CoreGameResource; // 削除

// pub use time::DeltaTime; // time からは何も使わないか、正しいパスからインポート
// pub use crate::resources::time_resource::DeltaTime; // time_resource にもない可能性
pub use crate::systems::system_registry::DeltaTime; // コンパイラ提案の systems からインポート

pub use input_resource::InputResource;
pub use mouse_state::MouseState;
pub use game_config::GameConfigResource;

// リソース関連のエラー
#[derive(Debug, Clone)]
pub enum ResourceError {
    NotFound(String),
    WrongType(String),
    AlreadyExists(String),
}

impl std::fmt::Display for ResourceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ResourceError::NotFound(name) => write!(f, "Resource not found: {}", name),
            ResourceError::WrongType(name) => write!(f, "Resource type mismatch: {}", name),
            ResourceError::AlreadyExists(name) => write!(f, "Resource already exists: {}", name),
        }
    }
}

/// リソースのバッチ処理のためのラッパー（読み取り専用）
/// 
/// 複数のリソースに対して型安全にアクセスするためのラッパークラスです。
/// ResourceManagerのbatchメソッドから生成され、リソースの読み取り専用操作をサポートします。
/// 
/// # 例
/// 
/// ```
/// world.with_resources(|batch| {
///     // 単一のリソース取得
///     if let Some(config) = batch.get::<GameConfigResource>() {
///         // configを使った処理
///     }
///     
///     // 複数のリソースを一度に取得（両方の命名規則に対応）
///     if let Some((config, board)) = batch.get_many::<GameConfigResource, BoardResource>() {
///         // 両方のリソースを使った処理
///     }
///     
///     // または
///     if let Some((config, board)) = batch.get_resources::<GameConfigResource, BoardResource>() {
///         // 両方のリソースを使った処理（World構造体と同じ命名規則）
///     }
/// });
/// ```
pub struct ResourceBatch<'a> {
    pub resources: &'a ResourceManager,
}

impl<'a> ResourceBatch<'a> {
    /// 指定した型のリソースを取得
    /// 
    /// 指定した型Tのリソースを読み取り専用で取得します。
    /// リソースが存在しない場合はNoneを返します。
    /// 
    /// # 例
    /// 
    /// ```
    /// if let Some(config) = batch.get::<GameConfigResource>() {
    ///     println!("難易度: {}", config.difficulty);
    /// }
    /// ```
    pub fn get<T: Resource>(&self) -> Option<&T> {
        match self.resources.get::<T>() {
            Ok(rc) => {
                let borrowed = rc.borrow();
                borrowed.downcast_ref::<T>()
                    .map(|r| unsafe { std::mem::transmute::<&T, &T>(r) })
            }
            Err(_) => None
        }
    }
    
    /// 複数のリソースを一度に取得する
    /// 
    /// 2つの異なる型のリソースを同時に読み取り専用で取得します。
    /// いずれかのリソースが存在しない場合はNoneを返します。
    /// 
    /// # 例
    /// 
    /// ```
    /// if let Some((config, board)) = batch.get_many::<GameConfigResource, BoardResource>() {
    ///     // 両方のリソースを使った処理
    /// }
    /// ```
    pub fn get_many<A: Resource, B: Resource>(&self) -> Option<(&A, &B)> {
        let a = self.get::<A>()?;
        let b = self.get::<B>()?;
        Some((a, b))
    }
    
    /// 3つのリソースを一度に取得する
    /// 
    /// 3つの異なる型のリソースを同時に読み取り専用で取得します。
    /// いずれかのリソースが存在しない場合はNoneを返します。
    /// 
    /// # 例
    /// 
    /// ```
    /// if let Some((config, board, player)) = batch.get_many3::<GameConfigResource, BoardResource, PlayerResource>() {
    ///     // 3つのリソースを使った処理
    /// }
    /// ```
    pub fn get_many3<A: Resource, B: Resource, C: Resource>(&self) -> Option<(&A, &B, &C)> {
        let a = self.get::<A>()?;
        let b = self.get::<B>()?;
        let c = self.get::<C>()?;
        Some((a, b, c))
    }
    
    /// 複数のリソースを一度に取得する（World構造体との命名互換用）
    /// 
    /// `get_many`と同じ機能ですが、World構造体のメソッド名と互換性があります。
    /// 
    /// # 例
    /// 
    /// ```
    /// if let Some((config, board)) = batch.get_resources::<GameConfigResource, BoardResource>() {
    ///     // 両方のリソースを使った処理
    /// }
    /// ```
    pub fn get_resources<A: Resource, B: Resource>(&self) -> Option<(&A, &B)> {
        self.get_many::<A, B>()
    }
    
    /// 3つのリソースを一度に取得する（World構造体との命名互換用）
    /// 
    /// `get_many3`と同じ機能ですが、World構造体のメソッド名と互換性があります。
    /// 
    /// # 例
    /// 
    /// ```
    /// if let Some((config, board, player)) = batch.get_resources3::<GameConfigResource, BoardResource, PlayerResource>() {
    ///     // 3つのリソースを使った処理
    /// }
    /// ```
    pub fn get_resources3<A: Resource, B: Resource, C: Resource>(&self) -> Option<(&A, &B, &C)> {
        self.get_many3::<A, B, C>()
    }
}

/// リソースのバッチ処理のためのラッパー（読み書き可能）
/// 
/// 複数のリソースに対して型安全にアクセスするためのラッパークラスです。
/// ResourceManagerのbatch_mutメソッドから生成され、リソースの読み書き操作をサポートします。
/// 
/// # 例
/// 
/// ```
/// world.with_resources_mut(|batch| {
///     // 単一のリソース取得（読み取り専用）
///     if let Some(config) = batch.get::<GameConfigResource>() {
///         // configを使った読み取り専用処理
///     }
///     
///     // 単一のリソース取得（書き込み可能）
///     if let Some(board) = batch.get_mut::<BoardResource>() {
///         board.reset(); // 可変操作
///     }
///     
///     // 複数のリソースを一度に取得（1つは書き込み可能）
///     if let Some((config, board)) = batch.get_many_mut::<GameConfigResource, BoardResource>() {
///         board.configure(config.difficulty); // configは読み取り専用、boardは可変
///     }
///     
///     // 3つのリソースを一度に取得（3つ目のみ可変）
///     if let Some((config, input, player)) = batch.get_many3_mut::<GameConfigResource, InputResource, PlayerResource>() {
///         player.update(config, input); // playerのみ可変
///     }
/// });
/// ```
pub struct ResourceBatchMut<'a> {
    pub resources: &'a mut ResourceManager,
}

impl<'a> ResourceBatchMut<'a> {
    /// 指定した型のリソースを取得（読み取り専用）
    /// 
    /// 指定した型Tのリソースを読み取り専用で取得します。
    /// リソースが存在しない場合はNoneを返します。
    /// 
    /// # 例
    /// 
    /// ```
    /// if let Some(config) = batch.get::<GameConfigResource>() {
    ///     println!("難易度: {}", config.difficulty);
    /// }
    /// ```
    pub fn get<T: Resource>(&self) -> Option<&T> {
        match self.resources.get::<T>() {
            Ok(rc) => {
                let borrowed = rc.borrow();
                borrowed.downcast_ref::<T>()
                    .map(|r| unsafe { std::mem::transmute::<&T, &T>(r) })
            }
            Err(_) => None
        }
    }
    
    /// 指定した型のリソースを取得（読み書き可能）
    /// 
    /// 指定した型Tのリソースを可変で取得します。
    /// リソースが存在しない場合はNoneを返します。
    /// 
    /// # 例
    /// 
    /// ```
    /// if let Some(board) = batch.get_mut::<BoardResource>() {
    ///     board.reset(); // 可変操作
    /// }
    /// ```
    pub fn get_mut<T: Resource>(&mut self) -> Option<&mut T> {
        match self.resources.get_mut::<T>() {
            Ok(rc) => {
                let mut borrowed = rc.borrow_mut();
                borrowed.downcast_mut::<T>()
                    .map(|r| unsafe { std::mem::transmute::<&mut T, &mut T>(r) })
            }
            Err(_) => None
        }
    }
    
    /// 複数のリソースを一度に取得する（すべて読み取り専用）
    /// 
    /// 2つの異なる型のリソースを同時に読み取り専用で取得します。
    /// いずれかのリソースが存在しない場合はNoneを返します。
    /// 
    /// # 例
    /// 
    /// ```
    /// if let Some((config, board)) = batch.get_many::<GameConfigResource, BoardResource>() {
    ///     // 両方のリソースを使った読み取り専用処理
    /// }
    /// ```
    pub fn get_many<A: Resource, B: Resource>(&self) -> Option<(&A, &B)> {
        let a = self.get::<A>()?;
        let b = self.get::<B>()?;
        Some((a, b))
    }
    
    /// 複数のリソースを一度に取得する（World構造体との命名互換用）
    /// 
    /// `get_many`と同じ機能ですが、World構造体のメソッド名と互換性があります。
    /// 
    /// # 例
    /// 
    /// ```
    /// if let Some((config, board)) = batch.get_resources::<GameConfigResource, BoardResource>() {
    ///     // 両方のリソースを使った読み取り専用処理
    /// }
    /// ```
    pub fn get_resources<A: Resource, B: Resource>(&self) -> Option<(&A, &B)> {
        self.get_many::<A, B>()
    }
    
    /// 複数のリソースを一度に取得する（2つ目が可変）
    /// 
    /// 2つの異なる型のリソースを取得し、1つ目は読み取り専用、2つ目は可変として扱います。
    /// いずれかのリソースが存在しない場合はNoneを返します。
    /// 
    /// # 例
    /// 
    /// ```
    /// if let Some((config, board)) = batch.get_many_mut::<GameConfigResource, BoardResource>() {
    ///     board.configure(config.difficulty); // configは読み取り専用、boardは可変
    /// }
    /// ```
    pub fn get_many_mut<A: Resource, B: Resource>(&mut self) -> Option<(&A, &mut B)> {
        // 型IDを比較して同じ型でないことを確認
        if std::any::TypeId::of::<A>() == std::any::TypeId::of::<B>() {
            return None;
        }
        
        // ポインタ経由でライフタイムの制約を回避
        let a_ptr = match self.resources.get::<A>() {
            Ok(rc) => {
                let borrowed = rc.borrow();
                borrowed.downcast_ref::<A>()
                    .map(|r| r as *const A)
            }
            Err(_) => None
        }?;
        
        let b_ptr = match self.resources.get_mut::<B>() {
            Ok(rc) => {
                let mut borrowed = rc.borrow_mut();
                borrowed.downcast_mut::<B>()
                    .map(|r| r as *mut B)
            }
            Err(_) => None
        }?;
        
        // ポインタから参照を安全に復元
        unsafe {
            Some((&*a_ptr, &mut *b_ptr))
        }
    }
    
    /// 複数のリソースを一度に取得する（2つ目が可変）（World構造体との命名互換用）
    /// 
    /// `get_many_mut`と同じ機能ですが、World構造体のメソッド名と互換性があります。
    /// 
    /// # 例
    /// 
    /// ```
    /// if let Some((config, board)) = batch.get_resources_mut::<GameConfigResource, BoardResource>() {
    ///     board.configure(config.difficulty); // configは読み取り専用、boardは可変
    /// }
    /// ```
    pub fn get_resources_mut<A: Resource, B: Resource>(&mut self) -> Option<(&A, &mut B)> {
        self.get_many_mut::<A, B>()
    }
    
    /// 3つのリソースを一度に取得する（すべて読み取り専用）
    /// 
    /// 3つの異なる型のリソースを同時に読み取り専用で取得します。
    /// いずれかのリソースが存在しない場合はNoneを返します。
    /// 
    /// # 例
    /// 
    /// ```
    /// if let Some((config, board, player)) = batch.get_many3::<GameConfigResource, BoardResource, PlayerResource>() {
    ///     // 3つのリソースを使った読み取り専用処理
    /// }
    /// ```
    pub fn get_many3<A: Resource, B: Resource, C: Resource>(&self) -> Option<(&A, &B, &C)> {
        let a = self.get::<A>()?;
        let b = self.get::<B>()?;
        let c = self.get::<C>()?;
        Some((a, b, c))
    }
    
    /// 3つのリソースを一度に取得する（World構造体との命名互換用）
    /// 
    /// `get_many3`と同じ機能ですが、World構造体のメソッド名と互換性があります。
    /// 
    /// # 例
    /// 
    /// ```
    /// if let Some((config, board, player)) = batch.get_resources3::<GameConfigResource, BoardResource, PlayerResource>() {
    ///     // 3つのリソースを使った読み取り専用処理
    /// }
    /// ```
    pub fn get_resources3<A: Resource, B: Resource, C: Resource>(&self) -> Option<(&A, &B, &C)> {
        self.get_many3::<A, B, C>()
    }
    
    /// 3つのリソースを一度に取得する（3つ目が可変）
    /// 
    /// 3つの異なる型のリソースを取得し、最初の2つは読み取り専用、3つ目は可変として扱います。
    /// いずれかのリソースが存在しない場合はNoneを返します。
    /// 
    /// # 例
    /// 
    /// ```
    /// if let Some((config, input, player)) = batch.get_many3_mut::<GameConfigResource, InputResource, PlayerResource>() {
    ///     player.update(config, input); // playerのみ可変
    /// }
    /// ```
    pub fn get_many3_mut<A: Resource, B: Resource, C: Resource>(&mut self) -> Option<(&A, &B, &mut C)> {
        // 型IDを比較して同じ型でないことを確認
        if std::any::TypeId::of::<A>() == std::any::TypeId::of::<B>() || 
           std::any::TypeId::of::<A>() == std::any::TypeId::of::<C>() || 
           std::any::TypeId::of::<B>() == std::any::TypeId::of::<C>() {
            return None;
        }
        
        // ポインタ経由でライフタイムの制約を回避
        let a_ptr = match self.resources.get::<A>() {
            Ok(rc) => {
                let borrowed = rc.borrow();
                borrowed.downcast_ref::<A>()
                    .map(|r| r as *const A)
            }
            Err(_) => None
        }?;
        
        let b_ptr = match self.resources.get::<B>() {
            Ok(rc) => {
                let borrowed = rc.borrow();
                borrowed.downcast_ref::<B>()
                    .map(|r| r as *const B)
            }
            Err(_) => None
        }?;
        
        let c_ptr = match self.resources.get_mut::<C>() {
            Ok(rc) => {
                let mut borrowed = rc.borrow_mut();
                borrowed.downcast_mut::<C>()
                    .map(|r| r as *mut C)
            }
            Err(_) => None
        }?;
        
        // ポインタから参照を安全に復元
        unsafe {
            Some((&*a_ptr, &*b_ptr, &mut *c_ptr))
        }
    }
    
    /// 3つのリソースを一度に取得する（3つ目が可変）（World構造体との命名互換用）
    /// 
    /// `get_many3_mut`と同じ機能ですが、World構造体のメソッド名と互換性があります。
    /// 
    /// # 例
    /// 
    /// ```
    /// if let Some((config, input, player)) = batch.get_resources3_mut::<GameConfigResource, InputResource, PlayerResource>() {
    ///     player.update(config, input); // playerのみ可変
    /// }
    /// ```
    pub fn get_resources3_mut<A: Resource, B: Resource, C: Resource>(&mut self) -> Option<(&A, &B, &mut C)> {
        self.get_many3_mut::<A, B, C>()
    }
    
    /// 3つのリソースを一度に取得する（2つめと3つめが可変）
    /// 
    /// 3つの異なる型のリソースを取得し、1つ目は読み取り専用、2つ目と3つ目は可変として扱います。
    /// いずれかのリソースが存在しない場合はNoneを返します。
    /// 
    /// # 例
    /// 
    /// ```
    /// if let Some((config, board, player)) = batch.get_many3_mut2::<GameConfigResource, BoardResource, PlayerResource>() {
    ///     board.reset();
    ///     player.set_position(config.default_position);
    /// }
    /// ```
    pub fn get_many3_mut2<A: Resource, B: Resource, C: Resource>(&mut self) -> Option<(&A, &mut B, &mut C)> {
        // 型IDを比較して同じ型でないことを確認
        if std::any::TypeId::of::<A>() == std::any::TypeId::of::<B>() || 
           std::any::TypeId::of::<A>() == std::any::TypeId::of::<C>() || 
           std::any::TypeId::of::<B>() == std::any::TypeId::of::<C>() {
            return None;
        }
        
        // ポインタ経由でライフタイムの制約を回避
        let a_ptr = match self.resources.get::<A>() {
            Ok(rc) => {
                let borrowed = rc.borrow();
                borrowed.downcast_ref::<A>()
                    .map(|r| r as *const A)
            }
            Err(_) => None
        }?;
        
        let b_ptr = match self.resources.get_mut::<B>() {
            Ok(rc) => {
                let mut borrowed = rc.borrow_mut();
                borrowed.downcast_mut::<B>()
                    .map(|r| r as *mut B)
            }
            Err(_) => None
        }?;
        
        let c_ptr = match self.resources.get_mut::<C>() {
            Ok(rc) => {
                let mut borrowed = rc.borrow_mut();
                borrowed.downcast_mut::<C>()
                    .map(|r| r as *mut C)
            }
            Err(_) => None
        }?;
        
        // ポインタから参照を安全に復元
        unsafe {
            Some((&*a_ptr, &mut *b_ptr, &mut *c_ptr))
        }
    }
    
    /// 3つのリソースを一度に取得する（2つめと3つめが可変）（World構造体との命名互換用）
    /// 
    /// `get_many3_mut2`と同じ機能ですが、World構造体のメソッド名と互換性があります。
    /// 
    /// # 例
    /// 
    /// ```
    /// if let Some((config, board, player)) = batch.get_resources3_mut2::<GameConfigResource, BoardResource, PlayerResource>() {
    ///     board.reset();
    ///     player.set_position(config.default_position);
    /// }
    /// ```
    pub fn get_resources3_mut2<A: Resource, B: Resource, C: Resource>(&mut self) -> Option<(&A, &mut B, &mut C)> {
        self.get_many3_mut2::<A, B, C>()
    }
}

/// リソースシステムの初期化
pub fn init() -> ResourceManager {
    ResourceManager::new()
} 