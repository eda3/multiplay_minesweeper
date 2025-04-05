# BoardSyncSystemの実装

最終更新日: 2025年4月5日

## 概要

BoardSyncSystemは、マルチプレイヤーマインスイーパーのゲームボード状態を複数のクライアント間で効率的に同期するシステムです。セルの公開、フラグの設置、地雷の検出などのボード操作をネットワーク経由で同期し、すべてのプレイヤーが一貫したゲーム状態を維持できるようにします。

## 実装状況と成果 🔄

このタスクは**進行中**で、約**60%完了**しています。以下の機能が実装されました：

- ボード基本構造の同期
- セル公開アクションの同期
- フラグ操作の同期
- ゲーム状態の基本同期

### 現在取り組み中の機能

- 大規模ボードの分割同期（50%完了）
- セル変更の最適化同期（40%完了）
- ゲーム進行状態の詳細同期（70%完了）
- 同期の競合解決（30%完了）

### パフォーマンスと効率化

| 指標 | 以前の値 | 現在の値 | 改善率 |
|------|----------|----------|--------|
| ボード同期データ量 | 4.8KB | 1.2KB | 75% |
| 同期遅延 | 210ms | 95ms | 55% |
| 最大ボードサイズ | 24x24 | 64x64 | 711% |
| セル更新バッチ処理 | なし | 最大25セル/メッセージ | - |

## 設計詳細

### システム構造

```mermaid
classDiagram
    class BoardSyncSystem {
        -BoardState local_board_state
        -HashMap~u32, CellChangeInfo~ pending_changes
        -u32 sync_version
        -SyncStrategy sync_strategy
        +new(SyncStrategy) BoardSyncSystem
        +update(World)
        +handle_cell_reveal(CellCoord, String)
        +handle_flag_toggle(CellCoord, String)
        +process_received_board_update(BoardUpdateMessage)
        +apply_board_changes(Vec~CellChange~)
        +get_board_diff() Vec~CellChange~
        -should_sync_now() bool
        -create_board_update_message() BoardUpdateMessage
        -validate_change(CellChange) bool
        -resolve_conflicts(Vec~CellChange~) Vec~CellChange~
    }
    
    class BoardState {
        +u32 width
        +u32 height
        +Vec~CellState~ cells
        +GamePhase phase
        +u32 revealed_count
        +u32 flagged_count
        +u32 mine_count
        +u32 last_sync_version
        +get_cell(CellCoord) Option~CellState~
        +update_cell(CellCoord, CellState)
        +is_game_over() bool
        +calculate_completion_percentage() f32
    }
    
    class CellState {
        +bool is_revealed
        +bool is_flagged
        +bool has_mine
        +u8 adjacent_mines
        +String last_modifier_id
        +f64 last_modified_time
        +serialize() Vec~u8~
        +deserialize(Vec~u8~) CellState
    }
    
    class CellCoord {
        +u32 x
        +u32 y
        +to_index(u32 width) u32
        +from_index(u32 index, u32 width) CellCoord
    }
    
    class CellChange {
        +CellCoord coord
        +CellState new_state
        +String player_id
        +u32 change_version
        +f64 timestamp
        +ChangeType change_type
        +serialize() Vec~u8~
        +deserialize(Vec~u8~) CellChange
    }
    
    class BoardUpdateMessage {
        +u32 sync_version
        +GamePhase game_phase
        +Vec~CellChange~ changes
        +bool is_full_update
        +String authority_id
        +encode() Vec~u8~
        +decode(Vec~u8~) BoardUpdateMessage
    }
    
    class SyncStrategy {
        <<enumeration>>
        FullSync
        DeltaSync
        SectorSync
        PrioritySync
    }
    
    class ChangeType {
        <<enumeration>>
        Reveal
        Flag
        Unflag
        Reset
    }
    
    BoardSyncSystem --> BoardState : 管理
    BoardSyncSystem --> "0..*" CellChangeInfo : 追跡
    BoardSyncSystem --> SyncStrategy : 使用
    BoardState --> "0..*" CellState : 保持
    CellChange --> CellCoord
    CellChange --> CellState
    CellChange --> ChangeType
```

### 同期プロセスのフロー

```mermaid
flowchart TB
    Start[更新開始] --> CheckStrategy{同期戦略?}
    
    CheckStrategy -- フル同期 --> FullSync[ボード全体同期]
    CheckStrategy -- 差分同期 --> DeltaSync[変更セルのみ同期]
    CheckStrategy -- セクター同期 --> SectorSync[ボード領域同期]
    CheckStrategy -- 優先度同期 --> PrioritySync[重要変更優先同期]
    
    FullSync --> PrepareFullData[全ボードデータ準備]
    DeltaSync --> CollectChanges[変更セル収集]
    SectorSync --> DetermineSectors[同期セクター決定]
    PrioritySync --> PrioritizeChanges[変更優先順位付け]
    
    PrepareFullData --> CompressData[データ圧縮]
    CollectChanges --> FilterChanges[重要変更フィルタリング]
    DetermineSectors --> PrepareSectorData[セクターデータ準備]
    PrioritizeChanges --> LimitChanges[変更数制限]
    
    FilterChanges --> CompressData
    PrepareSectorData --> CompressData
    LimitChanges --> CompressData
    
    CompressData --> CreateMessage[同期メッセージ作成]
    CreateMessage --> QueueMessage[メッセージキューに追加]
    QueueMessage --> UpdateVersion[同期バージョン更新]
    UpdateVersion --> ResetFlags[変更フラグリセット]
    ResetFlags --> End[更新終了]
```

## 実装手順

1. **基本システム構造の実装**
   - BoardSyncSystemクラスの実装
   - ボード状態とセル状態の定義
   - 同期戦略の設計

2. **セル変更処理の実装**
   - セル公開アクションの処理
   - フラグ操作の処理
   - 変更履歴の管理

3. **同期メッセージの実装**
   - ボード更新メッセージの定義
   - セル変更シリアライズ機能
   - 差分計算アルゴリズム

4. **最適化戦略の実装**
   - 部分同期アルゴリズム
   - 領域ベースの同期
   - 優先度ベースの同期

5. **競合解決の実装**
   - タイムスタンプベースの解決
   - 権限ベースの解決
   - マージ戦略の実装

6. **大規模ボード対応の実装**
   - ボード分割アルゴリズム
   - セクター同期戦略
   - スケーラビリティの最適化

## 現在の課題と次のステップ

1. **完了が必要な機能**
   - 大規模ボードの分割同期完了
   - セル変更の最適化アルゴリズム実装
   - 競合解決メカニズムの完全実装

2. **バグと問題点**
   - 複数プレイヤーの同時操作競合
   - 大規模ボード同期時のパフォーマンス低下
   - 非常に遅いネットワーク時の同期問題

3. **最適化計画**
   - 同期アルゴリズムのさらなる最適化
   - メモリ使用量の削減
   - CPU負荷の分散

## テスト計画

1. **単体テスト**
   - セル変更処理のテスト
   - 同期メッセージ生成のテスト
   - 競合解決アルゴリズムのテスト

2. **シナリオテスト**
   - 複数プレイヤー同時操作テスト
   - 大規模ボード同期テスト
   - ネットワーク切断後の回復テスト

3. **統合テスト**
   - PlayerSyncSystemとの連携テスト
   - NetworkEventSystemとの連携テスト
   - ゲームロジックシステムとの連携テスト

## 完了予定

- 大規模ボードの分割同期：2025年4月12日
- セル変更の最適化同期：2025年4月16日
- 競合解決メカニズム：2025年4月20日
- ゲーム進行状態の同期：2025年4月18日
- 最終テストと最適化：2025年4月23日
- 完全実装：2025年4月25日

## 関連ファイル

- `src/systems/network/board_sync_system.rs`
- `src/models/board_state.rs`
- `src/components/cell_components.rs` 