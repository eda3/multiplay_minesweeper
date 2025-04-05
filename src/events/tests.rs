#[cfg(test)]
mod event_system_tests {
    use std::sync::{Arc, Mutex};
    use crate::events::{Event, EventBus, EventHandler};
    use crate::events::game_events::{GameStartEvent, GameEndEvent};
    use crate::events::board_events::{BoardInitializedEvent, CellRevealedEvent};
    use crate::models::difficulty::Difficulty;
    use crate::models::coordinate::Coordinate;
    use crate::models::cell::CellValue;
    use crate::impl_event;

    #[derive(Debug, Clone)]
    struct TestEvent {
        pub message: String,
        pub value: i32,
    }

    impl_event!(TestEvent, "Test");

    #[test]
    fn test_event_subscribe_and_publish() {
        // イベントバスを作成
        let event_bus = EventBus::new().with_debug(true);
        
        // イベント受信を記録するための共有変数
        let received = Arc::new(Mutex::new(Vec::<String>::new()));
        let received_clone = received.clone();
        
        // イベントを購読
        let handler = event_bus.subscribe("テストハンドラ", move |event: &TestEvent| {
            let mut guard = received_clone.lock().unwrap();
            guard.push(format!("受信: {} ({})", event.message, event.value));
        });
        
        // イベントを発行
        event_bus.publish(TestEvent {
            message: "こんにちは".to_string(),
            value: 42,
        });
        
        // 結果を検証
        let result = received.lock().unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], "受信: こんにちは (42)");
    }
    
    #[test]
    fn test_multiple_handlers() {
        let event_bus = EventBus::new();
        
        let count1 = Arc::new(Mutex::new(0));
        let count2 = Arc::new(Mutex::new(0));
        
        let count1_clone = count1.clone();
        let count2_clone = count2.clone();
        
        // 2つのハンドラを登録
        event_bus.subscribe("ハンドラ1", move |_: &TestEvent| {
            let mut guard = count1_clone.lock().unwrap();
            *guard += 1;
        });
        
        event_bus.subscribe("ハンドラ2", move |_: &TestEvent| {
            let mut guard = count2_clone.lock().unwrap();
            *guard += 1;
        });
        
        // イベントを3回発行
        for i in 0..3 {
            event_bus.publish(TestEvent {
                message: format!("イベント{}", i),
                value: i,
            });
        }
        
        // 各ハンドラが3回呼ばれたことを確認
        assert_eq!(*count1.lock().unwrap(), 3);
        assert_eq!(*count2.lock().unwrap(), 3);
    }
    
    #[test]
    fn test_once_handler() {
        let event_bus = EventBus::new();
        
        let count = Arc::new(Mutex::new(0));
        let count_clone = count.clone();
        
        // 一度だけ実行するハンドラを登録
        let handler = event_bus.subscribe("一回だけ", move |_: &TestEvent| {
            let mut guard = count_clone.lock().unwrap();
            *guard += 1;
        }).once();
        
        // イベントを3回発行
        for i in 0..3 {
            event_bus.publish(TestEvent {
                message: format!("イベント{}", i),
                value: i,
            });
        }
        
        // ハンドラが1回だけ呼ばれたことを確認
        assert_eq!(*count.lock().unwrap(), 1);
    }
    
    #[test]
    fn test_unregister_handler() {
        let event_bus = EventBus::new();
        
        let count = Arc::new(Mutex::new(0));
        let count_clone = count.clone();
        
        // ハンドラを登録
        let handler = event_bus.subscribe("削除テスト", move |_: &TestEvent| {
            let mut guard = count_clone.lock().unwrap();
            *guard += 1;
        });
        
        // 最初のイベントを発行
        event_bus.publish(TestEvent {
            message: "最初".to_string(),
            value: 1,
        });
        
        // ハンドラを削除
        event_bus.unregister_handler::<TestEvent>(handler.id);
        
        // さらにイベントを発行
        event_bus.publish(TestEvent {
            message: "二回目".to_string(),
            value: 2,
        });
        
        // ハンドラが1回だけ呼ばれたことを確認
        assert_eq!(*count.lock().unwrap(), 1);
    }
    
    #[test]
    fn test_priority_handler() {
        let event_bus = EventBus::new();
        
        let execution_order = Arc::new(Mutex::new(Vec::<i32>::new()));
        let execution_order_clone1 = execution_order.clone();
        let execution_order_clone2 = execution_order.clone();
        let execution_order_clone3 = execution_order.clone();
        
        // 優先度の異なる3つのハンドラを登録（順序をシャッフル）
        event_bus.subscribe("優先度低", move |_: &TestEvent| {
            let mut guard = execution_order_clone1.lock().unwrap();
            guard.push(1);
        }).with_priority(1);
        
        event_bus.subscribe("優先度高", move |_: &TestEvent| {
            let mut guard = execution_order_clone3.lock().unwrap();
            guard.push(3);
        }).with_priority(3);
        
        event_bus.subscribe("優先度中", move |_: &TestEvent| {
            let mut guard = execution_order_clone2.lock().unwrap();
            guard.push(2);
        }).with_priority(2);
        
        // イベントを発行
        event_bus.publish(TestEvent {
            message: "優先度テスト".to_string(),
            value: 0,
        });
        
        // 優先度順に実行されたことを確認
        let result = execution_order.lock().unwrap();
        assert_eq!(*result, vec![3, 2, 1]);
    }
    
    #[test]
    fn test_real_game_events() {
        let event_bus = EventBus::new();
        
        let game_started = Arc::new(Mutex::new(false));
        let board_initialized = Arc::new(Mutex::new(false));
        let game_ended = Arc::new(Mutex::new(false));
        
        let game_started_clone = game_started.clone();
        let board_initialized_clone = board_initialized.clone();
        let game_ended_clone = game_ended.clone();
        
        // ゲーム開始イベントの購読
        event_bus.subscribe("ゲーム開始ハンドラ", move |event: &GameStartEvent| {
            let mut guard = game_started_clone.lock().unwrap();
            *guard = true;
            
            assert_eq!(event.difficulty, Difficulty::Medium);
        });
        
        // ボード初期化イベントの購読
        event_bus.subscribe("ボード初期化ハンドラ", move |event: &BoardInitializedEvent| {
            let mut guard = board_initialized_clone.lock().unwrap();
            *guard = true;
            
            assert_eq!(event.width, 16);
            assert_eq!(event.height, 16);
            assert_eq!(event.mine_count, 40);
        });
        
        // ゲーム終了イベントの購読
        event_bus.subscribe("ゲーム終了ハンドラ", move |event: &GameEndEvent| {
            let mut guard = game_ended_clone.lock().unwrap();
            *guard = true;
            
            assert_eq!(event.is_win, true);
        });
        
        // イベントを順番に発行
        event_bus.publish(GameStartEvent {
            difficulty: Difficulty::Medium,
            custom_width: None,
            custom_height: None,
            custom_mines: None,
            is_multiplayer: false,
            session_id: None,
        });
        
        event_bus.publish(BoardInitializedEvent {
            width: 16,
            height: 16,
            mine_count: 40,
            first_click: Some(Coordinate::new(8, 8)),
        });
        
        // セル開示イベントの発行
        event_bus.publish(CellRevealedEvent {
            coord: Coordinate::new(8, 8),
            value: CellValue::Empty(0),
            is_chain: false,
        });
        
        // ゲーム終了イベントの発行
        event_bus.publish(GameEndEvent {
            is_win: true,
            play_time: 120,
            revealed_cells: 216,
            flagged_cells: 40,
            score: 1500,
        });
        
        // すべてのイベントが処理されたことを確認
        assert_eq!(*game_started.lock().unwrap(), true);
        assert_eq!(*board_initialized.lock().unwrap(), true);
        assert_eq!(*game_ended.lock().unwrap(), true);
    }
} 