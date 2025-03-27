# SystemRegistryとの統合

## 概要
SystemRegistryは、ゲームシステムとリソースマネージャーを統合し、
ECSアーキテクチャの中心となるコンポーネントです。システムの登録、実行順序の管理、
そしてリソースアクセスの仲介を行います。

## クラス構造

```mermaid
classDiagram
    class System {
        <<interface>>
        +update(delta_time: f64)
        +name() String
        +priority() i32
    }
    
    class ResourceManager {
        -HashMap~TypeId, Box~dyn Any~~ resources
        +insert~T~(resource)
        +get~T~() Option~&T~
        +get_mut~T~() Option~&mut T~
    }
    
    class SystemRegistry {
        -Vec~Box~dyn System~~ systems
        -ResourceManager resources
        +new() Self
        +register_system(system)
        +register_systems(systems)
        +add_resource~T~(resource)
        +get_resource~T~() Option~&T~
        +get_resource_mut~T~() Option~&mut T~
        +update(delta_time)
        +update_resources(delta_time)
        +sort_systems()
    }
    
    class InputSystem {
        +update(delta_time)
        +name() String
        +priority() i32
    }
    
    class RenderSystem {
        +update(delta_time)
        +name() String
        +priority() i32
    }
    
    class NetworkSystem {
        +update(delta_time)
        +name() String
        +priority() i32
    }
    
    class GameLogicSystem {
        +update(delta_time)
        +name() String
        +priority() i32
    }
    
    SystemRegistry --> "0..*" System : 管理
    SystemRegistry --> ResourceManager : 所有
    System <|.. InputSystem : 実装
    System <|.. RenderSystem : 実装
    System <|.. NetworkSystem : 実装
    System <|.. GameLogicSystem : 実装
```

## Systemインターフェース

```mermaid
classDiagram
    class System {
        <<trait>>
        +update(delta_time: f64) Result
        +name() String
        +priority() i32
        +init(registry: &mut SystemRegistry) Result~(), String~
        +shutdown()
    }
    
    note for System "すべてのゲームシステムが実装する必要のあるトレイト"
```

## SystemRegistryの初期化

```mermaid
flowchart TD
    A[new] --> B[systems = Vec::new()]
    B --> C[resources = ResourceManager::new()]
    C --> D[SystemRegistry構造体初期化]
```

## システム登録と実行

```mermaid
flowchart TD
    A[register_system] --> B[システムをvectorに追加]
    B --> C[初期化メソッド呼び出し]
    C --> D{初期化成功?}
    D -->|Yes| E[sort_systems呼び出し]
    D -->|No| F[エラー処理]
    
    G[register_systems] --> H[各システムに対して<br>register_system呼び出し]
    
    I[sort_systems] --> J[priorityに基づいてシステムをソート]
    
    K[update] --> L[update_resources呼び出し]
    L --> M[TimeResource更新]
    M --> N[各システムに対してupdate呼び出し]
```

## リソースアクセス委譲

```mermaid
flowchart LR
    A[add_resource] --> B[resources.insert呼び出し]
    
    C[get_resource] --> D[resources.get呼び出し]
    
    E[get_resource_mut] --> F[resources.get_mut呼び出し]
```

## システム実行フロー

```mermaid
sequenceDiagram
    participant Main as ゲームメインループ
    participant Registry as SystemRegistry
    participant TimeRes as TimeResource
    participant InputSys as InputSystem
    participant GameSys as GameLogicSystem
    participant NetworkSys as NetworkSystem
    participant RenderSys as RenderSystem
    
    Main->>Registry: update(delta_time)
    Registry->>TimeRes: 時間リソース更新
    
    Registry->>InputSys: update(delta_time)
    InputSys->>Registry: get_resource_mut::<PlayerStateResource>()
    Registry-->>InputSys: Some(player_state)
    InputSys->>InputSys: 入力処理実行
    
    Registry->>GameSys: update(delta_time)
    GameSys->>Registry: get_resource_mut::<CoreGameResource>()
    Registry-->>GameSys: Some(core_game)
    GameSys->>Registry: get_resource::<PlayerStateResource>()
    Registry-->>GameSys: Some(player_state)
    GameSys->>GameSys: ゲームロジック実行
    
    Registry->>NetworkSys: update(delta_time)
    NetworkSys->>Registry: get_resource::<PlayerStateResource>()
    Registry-->>NetworkSys: Some(player_state)
    NetworkSys->>NetworkSys: ネットワーク同期実行
    
    Registry->>RenderSys: update(delta_time)
    RenderSys->>Registry: get_resource::<CoreGameResource>()
    Registry-->>RenderSys: Some(core_game)
    RenderSys->>Registry: get_resource::<PlayerStateResource>()
    Registry-->>RenderSys: Some(player_state)
    RenderSys->>RenderSys: 描画処理実行
```

## 主要システムの実装計画

```mermaid
classDiagram
    class InputSystem {
        -HtmlCanvasElement canvas
        +new(canvas) Self
        +update(delta_time) Result
        +name() String
        +priority() i32
        -handle_mouse_move(event)
        -handle_mouse_click(event)
        -handle_keyboard(event)
    }
    
    class GameLogicSystem {
        +update(delta_time) Result
        +name() String
        +priority() i32
        -update_game_state()
        -check_win_condition()
        -handle_cell_reveal(x, y)
        -handle_flag_toggle(x, y)
    }
    
    class NetworkSystem {
        -WebSocket connection
        +update(delta_time) Result
        +name() String
        +priority() i32
        -send_position_update()
        -send_game_action(action)
        -handle_incoming_messages()
    }
    
    class RenderSystem {
        -CanvasRenderingContext2d context
        +update(delta_time) Result
        +name() String
        +priority() i32
        -render_board()
        -render_players()
        -render_ui()
        -render_effects()
    }
```

## 優先度ベースのシステム順序

```mermaid
flowchart LR
    A[InputSystem<br>priority=10] --> B[GameLogicSystem<br>priority=20]
    B --> C[NetworkSystem<br>priority=30]
    C --> D[RenderSystem<br>priority=40]
    
    note for A "入力処理は最も早く実行"
    note for B "入力に基づいてゲームロジックを実行"
    note for C "状態変更をネットワークで同期"
    note for D "最終的な状態を描画"
```

## SystemRegistryのファイル構造

```mermaid
classDiagram
    class src/ecs/mod.rs {
        pub mod system
        pub mod system_registry
        pub use system::System
        pub use system_registry::SystemRegistry
    }
    
    class src/ecs/system.rs {
        pub trait System { ... }
    }
    
    class src/ecs/system_registry.rs {
        pub struct SystemRegistry { ... }
        impl SystemRegistry { ... }
    }
    
    class src/systems/mod.rs {
        pub mod input_system
        pub mod game_logic_system
        pub mod network_system
        pub mod render_system
        pub use input_system::InputSystem
        pub use game_logic_system::GameLogicSystem
        pub use network_system::NetworkSystem
        pub use render_system::RenderSystem
    }
```

## ゲームメインループの統合

```mermaid
flowchart TD
    A[start_game関数] --> B[SystemRegistry作成]
    B --> C[各種リソース初期化と追加]
    C --> D[各種システム初期化と登録]
    D --> E[アニメーションフレーム設定]
    
    F[アニメーションフレーム関数] --> G[registry.update呼び出し]
    G --> H[次のフレームをリクエスト]
```

## 次のステップ

- 既存コードの移行計画
- 各システムの詳細実装
- リソースとシステムの連携テスト
- パフォーマンス検証 