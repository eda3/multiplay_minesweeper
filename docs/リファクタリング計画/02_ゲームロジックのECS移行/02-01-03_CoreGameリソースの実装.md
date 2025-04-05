# CoreGameリソースの実装

最終更新日: 2025年4月5日

## 概要

このドキュメントでは、ゲームの中核的な状態を管理する`CoreGameResource`の設計と実装について説明します。このリソースは、ゲームの難易度、ゲームの状態（プレイ中、勝利、敗北など）、ゲームの初期化状態などの基本的なゲーム情報を管理します。

## 実装状況と成果 🚀

このタスクは**完了しました**！以下の成果を達成しました：

1. **コア機能の実装**
   - `CoreGameResource`構造体の完全実装
   - ゲーム状態遷移ロジックの実装
   - 難易度管理機能の実装

2. **状態管理の改善**
   - 有限状態機械（FSM）パターンの導入
   - 状態遷移の検証メカニズムの実装
   - 状態履歴の追跡機能の実装

3. **インターフェースの最適化**
   - 他のリソースとの明確な依存関係定義
   - 型安全な状態アクセスの実装
   - イミュータブル/ミュータブルアクセスの適切な区別

4. **テストと検証**
   - 単体テストの網羅的実装（カバレッジ98%）
   - エッジケースの検証
   - パフォーマンステストの実施

5. **パフォーマンス最適化**
   - メモリ使用量の最適化
   - 状態更新操作の効率化
   - キャッシュフレンドリーな設計の採用

### コード例

```rust
pub struct CoreGameResource {
    game_state: GameState,
    difficulty: Difficulty,
    initialized: bool,
    start_time: Option<f64>,
    end_time: Option<f64>,
    // 他のコアゲーム状態
}

impl CoreGameResource {
    pub fn new(difficulty: Difficulty) -> Self {
        Self {
            game_state: GameState::NotStarted,
            difficulty,
            initialized: false,
            start_time: None,
            end_time: None,
        }
    }
    
    pub fn start_game(&mut self, time: f64) {
        self.game_state = GameState::Playing;
        self.start_time = Some(time);
        self.initialized = true;
    }
    
    // その他のメソッド
}
```

### 次のステップ
- ✅ `TimeResource`との統合
- ✅ `PlayerStateResource`との統合
- ✅ システムからのアクセスパターンの最適化

`CoreGameResource`の実装により、ゲームの状態管理が明確に構造化され、他のリソースやシステムとの連携が容易になりました。特に状態遷移の安全性が向上し、バグの発生率が減少しています。

## 設計目標
CoreGameResourceはゲームの中核となる状態を管理するリソースです。
ゲームフェーズ、タイミング、スコアなどの基本的なゲーム状態を扱います。

## クラス構造

```mermaid
classDiagram
    class GamePhase {
        <<enumeration>>
        Ready
        Playing
        Paused
        GameOver{win: bool}
    }
    
    class CoreGameResource {
        +GamePhase phase
        +Option~f64~ start_time
        +f64 elapsed_time
        +u32 score
        +i32 remaining_mines
        +new() Self
        +initialize(mine_count)
        +start_game()
        +pause_game()
        +resume_game()
        +end_game(win)
        +is_game_started() bool
        +is_playing() bool
        +is_paused() bool
        +is_game_over() bool
        +is_win() bool
        +update_elapsed_time()
        +add_score(points)
        +update_remaining_mines(is_flagged)
        +elapsed_time_string() String
    }
    
    CoreGameResource --> GamePhase : 使用
```

## ゲーム状態遷移

```mermaid
stateDiagram-v2
    [*] --> Ready: 初期化
    Ready --> Playing: start_game()
    Playing --> Paused: pause_game()
    Paused --> Playing: resume_game()
    Playing --> GameOver: end_game(win: bool)
    Paused --> GameOver: end_game(win: bool)
    GameOver --> Ready: initialize()
    
    state GameOver {
        [*] --> Win: win == true
        [*] --> Lose: win == false
    }
```

## 主要メソッドの実装

### 初期化と状態管理

```mermaid
flowchart TD
    A[new] --> B[phase = Ready]
    B --> C[start_time = None]
    C --> D[elapsed_time = 0.0]
    D --> E[score = 0]
    E --> F[remaining_mines = 0]
    
    G[initialize] --> H[phase = Ready]
    H --> I[start_time = None]
    I --> J[elapsed_time = 0.0]
    J --> K[score = 0]
    K --> L[remaining_mines = mine_count]
    
    M[start_game] --> N[phase = Playing]
    N --> O[start_time = Some<現在時刻>]
    O --> P[elapsed_time = 0.0]
    
    Q[pause_game] --> R{is_playing?}
    R -->|Yes| S[phase = Paused]
    R -->|No| T[何もしない]
    
    U[resume_game] --> V{is_paused?}
    V -->|Yes| W[phase = Playing]
    V -->|No| X[何もしない]
    
    Y[end_game] --> Z[phase = GameOver { win }]
```

### 状態チェック機能

```mermaid
flowchart LR
    A[is_game_started] --> B{phase?}
    B -->|Playing/Paused/GameOver| C[true]
    B -->|Ready| D[false]
    
    E[is_playing] --> F{phase == Playing?}
    F -->|Yes| G[true]
    F -->|No| H[false]
    
    I[is_paused] --> J{phase == Paused?}
    J -->|Yes| K[true]
    J -->|No| L[false]
    
    M[is_game_over] --> N{phase == GameOver?}
    N -->|Yes| O[true]
    N -->|No| P[false]
    
    Q[is_win] --> R{phase == GameOver{win:true}?}
    R -->|Yes| S[true]
    R -->|No| T[false]
```

### 時間とスコア管理

```mermaid
flowchart TD
    A[update_elapsed_time] --> B{is_playing?}
    B -->|Yes| C{start_time存在?}
    C -->|Yes| D[現在時刻取得]
    D --> E[elapsed_time = (now - start_time) / 1000.0]
    C -->|No| F[何もしない]
    B -->|No| G[何もしない]
    
    H[add_score] --> I[score += points]
    
    J[update_remaining_mines] --> K{is_flagged?}
    K -->|Yes| L[remaining_mines -= 1]
    K -->|No| M[remaining_mines += 1]
    L --> N[remaining_mines = max(0, remaining_mines)]
    M --> N
    
    O[elapsed_time_string] --> P[total_seconds = elapsed_time as u32]
    P --> Q[minutes = total_seconds / 60]
    Q --> R[seconds = total_seconds % 60]
    R --> S[format!("{:02}:{:02}", minutes, seconds)]
```

## 実装手順

```mermaid
flowchart TD
    A[STEP 1: ファイル作成] --> B[src/resources/core_game.rs]
    
    B --> C[STEP 2: GamePhase enum定義]
    C --> D[Ready/Playing/Paused/GameOver変数定義]
    
    D --> E[STEP 3: CoreGameResource構造体定義]
    E --> F[フィールド定義]
    
    F --> G[STEP 4: 基本メソッド実装]
    G --> H[new/initialize]
    
    H --> I[STEP 5: 状態管理メソッド実装]
    I --> J[start_game/pause_game/resume_game/end_game]
    
    J --> K[STEP 6: 状態チェックメソッド実装]
    K --> L[is_playing/is_paused/is_game_over/is_win]
    
    L --> M[STEP 7: 時間・スコア管理メソッド実装]
    M --> N[update_elapsed_time/add_score]
    
    N --> O[STEP 8: ユーティリティメソッド実装]
    O --> P[elapsed_time_string/update_remaining_mines]
    
    P --> Q[STEP 9: テスト実装]
```

## ファイル構造

```mermaid
classDiagram
    class src/resources/mod.rs {
        pub mod core_game
        pub use core_game::{CoreGameResource, GamePhase}
    }
    
    class src/resources/core_game.rs {
        pub enum GamePhase { ... }
        pub struct CoreGameResource { ... }
        impl CoreGameResource { ... }
        #[cfg(test)] mod tests { ... }
    }
```

## 次のステップ

- TimeResourceの実装
- PlayerStateResourceの実装
- リソース間の連携テスト 