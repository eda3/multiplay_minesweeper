use crate::resources::{
    CoreGameResource, GamePhase, TimeResource, 
    PlayerStateResource, GameConfigResource, ResourceManager
};
use crate::system::{SystemRegistry, system_registry::SystemPhase};

/// ECSベースのゲームエンジン
/// リソースとシステムを管理し、ゲームループを実行する
pub struct EcsGame {
    /// リソースマネージャー
    resources: ResourceManager,
    /// システムレジストリ
    systems: SystemRegistry,
    /// 初期化済みかどうか
    initialized: bool,
}

impl EcsGame {
    /// 新しいEcsGameインスタンスを作成
    pub fn new() -> Self {
        Self {
            resources: ResourceManager::new(),
            systems: SystemRegistry::new(),
            initialized: false,
        }
    }

    /// ゲームの初期化
    pub fn initialize(&mut self) {
        if self.initialized {
            return;
        }

        // コアリソースの初期化
        self.setup_core_resources();
        
        // 初期化フェーズのシステムを実行
        self.systems.run_startup(&mut self.resources);
        
        self.initialized = true;
    }

    /// コアリソースの設定
    fn setup_core_resources(&mut self) {
        // CoreGameResourceを追加
        let core_game = CoreGameResource::new();
        self.resources.insert(core_game);
        
        // TimeResourceを追加
        let time = TimeResource::new();
        self.resources.insert(time);
        
        // PlayerStateResourceを追加
        let player_state = PlayerStateResource::new();
        self.resources.insert(player_state);
        
        // GameConfigResourceを追加
        let game_config = GameConfigResource::new();
        self.resources.insert(game_config);
    }

    /// システムを追加
    pub fn add_system<S>(&mut self, system: S) -> usize
    where
        S: 'static + crate::system::System,
    {
        self.systems.add_system(Box::new(system))
    }

    /// ゲームループの1フレームを実行
    pub fn update(&mut self) {
        if !self.initialized {
            self.initialize();
        }

        // TimeResourceを更新
        if let Some(time) = self.resources.get_mut::<TimeResource>() {
            time.begin_frame();
        }

        // 各フェーズのシステムを実行
        self.systems.run_all_phases(&mut self.resources);

        // CoreGameResourceのチェック - ゲームが終了したかどうか
        if let Some(core_game) = self.resources.get::<CoreGameResource>() {
            let phase = core_game.phase();
            if let GamePhase::GameOver { .. } = phase {
                // ゲームオーバー処理
                println!("Game Over! Score: {}", core_game.score());
            }
        }
    }

    /// リソースへの参照を取得
    pub fn get_resource<T: 'static>(&self) -> Option<&T> {
        self.resources.get::<T>()
    }

    /// リソースへの可変参照を取得
    pub fn get_resource_mut<T: 'static>(&mut self) -> Option<&mut T> {
        self.resources.get_mut::<T>()
    }

    /// リソースを追加または更新
    pub fn insert_resource<T: 'static>(&mut self, resource: T) {
        self.resources.insert(resource);
    }

    /// ゲームをスタート
    pub fn start_game(&mut self) {
        if let Some(core_game) = self.resources.get_mut::<CoreGameResource>() {
            core_game.start_game();
        }
    }

    /// ゲームを一時停止
    pub fn pause_game(&mut self) {
        if let Some(core_game) = self.resources.get_mut::<CoreGameResource>() {
            core_game.pause_game();
        }
    }

    /// ゲームを再開
    pub fn resume_game(&mut self) {
        if let Some(core_game) = self.resources.get_mut::<CoreGameResource>() {
            core_game.resume_game();
        }
    }

    /// ゲームを終了
    pub fn end_game(&mut self, win: bool) {
        if let Some(core_game) = self.resources.get_mut::<CoreGameResource>() {
            core_game.end_game(win);
        }
    }

    /// ゲームのフェーズを取得
    pub fn game_phase(&self) -> GamePhase {
        self.resources
            .get::<CoreGameResource>()
            .map_or(GamePhase::Ready, |core| core.phase())
    }
}

impl Default for EcsGame {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // シンプルなテストシステム
    struct TestSystem {
        name: String,
        run_count: usize,
        phase: SystemPhase,
    }

    impl crate::system::System for TestSystem {
        fn name(&self) -> &str {
            &self.name
        }

        fn phase(&self) -> SystemPhase {
            self.phase
        }

        fn run(&mut self, _resources: &mut ResourceManager) {
            self.run_count += 1;
        }
    }

    #[test]
    fn test_ecs_game_initialization() {
        let mut game = EcsGame::new();
        game.initialize();

        // コアリソースが初期化されているか確認
        assert!(game.get_resource::<CoreGameResource>().is_some());
        assert!(game.get_resource::<TimeResource>().is_some());
        assert!(game.get_resource::<PlayerStateResource>().is_some());
        assert!(game.get_resource::<GameConfigResource>().is_some());

        // 初期状態の確認
        assert_eq!(game.game_phase(), GamePhase::Ready);
    }

    #[test]
    fn test_ecs_game_lifecycle() {
        let mut game = EcsGame::new();
        
        // テストシステムを追加
        let update_system = TestSystem {
            name: "UpdateSystem".to_string(),
            run_count: 0,
            phase: SystemPhase::Update,
        };
        
        let render_system = TestSystem {
            name: "RenderSystem".to_string(),
            run_count: 0,
            phase: SystemPhase::Render,
        };
        
        game.add_system(update_system);
        game.add_system(render_system);
        
        // 初期化
        game.initialize();
        
        // ゲームライフサイクルのテスト
        assert_eq!(game.game_phase(), GamePhase::Ready);
        
        // ゲーム開始
        game.start_game();
        assert_eq!(game.game_phase(), GamePhase::Playing);
        
        // 一時停止
        game.pause_game();
        assert_eq!(game.game_phase(), GamePhase::Paused);
        
        // 再開
        game.resume_game();
        assert_eq!(game.game_phase(), GamePhase::Playing);
        
        // 終了
        game.end_game(true);
        assert!(matches!(game.game_phase(), GamePhase::GameOver { win: true }));
    }

    #[test]
    fn test_ecs_game_update() {
        let mut game = EcsGame::new();
        
        // テストシステムを追加
        let update_system = TestSystem {
            name: "UpdateSystem".to_string(),
            run_count: 0,
            phase: SystemPhase::Update,
        };
        
        let system_id = game.add_system(update_system);
        
        // ゲーム開始
        game.initialize();
        game.start_game();
        
        // 更新を実行
        game.update();
        
        // システムが実行されたことを確認
        let systems = &game.systems;
        let system = systems.get_system(system_id).unwrap();
        let test_system = system.downcast_ref::<TestSystem>().unwrap();
        
        assert_eq!(test_system.run_count, 1);
        
        // もう一度更新
        game.update();
        
        let systems = &game.systems;
        let system = systems.get_system(system_id).unwrap();
        let test_system = system.downcast_ref::<TestSystem>().unwrap();
        
        assert_eq!(test_system.run_count, 2);
    }
} 