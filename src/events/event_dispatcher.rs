/**
 * イベントディスパッチャー
 * 
 * 型安全なイベントの発行と購読を管理するシステム
 */
use std::any::TypeId;
use std::collections::{HashMap, VecDeque};
use std::marker::PhantomData;
use std::fmt::Debug;

use crate::events::event_trait::Event;
use crate::events::event_handlers::{EventCallback, TypedEventCallback};
use crate::entities::EntityManager;

/// イベント購読者トレイト
pub trait EventSubscriber {
    /// イベントを購読
    fn subscribe<E, F>(&mut self, callback: F) -> usize
    where
        E: Event,
        F: Fn(&E, &mut EntityManager) + 'static;
    
    /// イベント購読を解除
    fn unsubscribe(&mut self, id: usize) -> bool;
}

/// イベントディスパッチャー
#[derive(Default)]
pub struct EventDispatcher {
    /// イベントタイプごとのコールバック
    callbacks: HashMap<TypeId, Vec<Box<dyn EventCallback>>>,
    /// 次のサブスクリプションID
    next_id: usize,
    /// サブスクリプションIDからコールバックインデックスへのマッピング
    subscription_map: HashMap<usize, (TypeId, usize)>,
    /// キューに溜まったイベント
    event_queue: VecDeque<Box<dyn Event>>,
}

impl EventDispatcher {
    /// 新しいイベントディスパッチャーを作成
    pub fn new() -> Self {
        Self {
            callbacks: HashMap::new(),
            next_id: 1,
            subscription_map: HashMap::new(),
            // WASM環境でのメモリ効率化: 標準的なイベント数に合わせて初期容量を8に設定
            // イベントディスパッチャーはやや低レベルな処理で、キューに大量のイベントが溜まる前に処理される想定
            event_queue: VecDeque::with_capacity(8),
        }
    }
    
    /// 特定の初期容量でイベントディスパッチャーを作成
    /// 
    /// # 引数
    /// 
    /// * `queue_capacity` - イベントキューの初期容量
    /// 
    /// # 例
    /// 
    /// ```
    /// // 高頻度イベント処理用に大きめの容量を指定
    /// let dispatcher = EventDispatcher::with_capacity(32);
    /// ```
    pub fn with_capacity(queue_capacity: usize) -> Self {
        Self {
            callbacks: HashMap::new(),
            next_id: 1,
            subscription_map: HashMap::new(),
            event_queue: VecDeque::with_capacity(queue_capacity),
        }
    }
    
    /// イベントを発行してすぐに処理
    pub fn publish<E: Event>(&mut self, event: E, entities: &mut EntityManager) {
        let type_id = TypeId::of::<E>();
        
        // イベントタイプに対応するコールバックを取得
        if let Some(callbacks) = self.callbacks.get(&type_id) {
            for callback in callbacks {
                callback.call(&event, entities);
            }
        }
    }
    
    /// イベントをキューに追加
    pub fn enqueue<E: Event>(&mut self, event: E) {
        self.event_queue.push_back(Box::new(event));
    }
    
    /// キューに溜まったイベントを処理
    pub fn process_queue(&mut self, entities: &mut EntityManager) {
        while let Some(event) = self.event_queue.pop_front() {
            let type_id = event.type_id();
            
            // イベントタイプに対応するコールバックを取得
            if let Some(callbacks) = self.callbacks.get(&type_id) {
                for callback in callbacks {
                    callback.call(event.as_ref(), entities);
                }
            }
        }
    }
    
    /// キューの長さを取得
    pub fn queue_len(&self) -> usize {
        self.event_queue.len()
    }
    
    /// キューをクリア
    pub fn clear_queue(&mut self) {
        self.event_queue.clear();
    }
    
    /// イベントタイプのサブスクライバー数を取得
    pub fn subscriber_count<E: Event>(&self) -> usize {
        let type_id = TypeId::of::<E>();
        self.callbacks.get(&type_id).map_or(0, |v| v.len())
    }
}

impl EventSubscriber for EventDispatcher {
    fn subscribe<E, F>(&mut self, callback: F) -> usize
    where
        E: Event,
        F: Fn(&E, &mut EntityManager) + 'static,
    {
        let type_id = TypeId::of::<E>();
        let callback = Box::new(TypedEventCallback::<E, F>::new(callback));
        
        // イベントタイプに対応するコールバックリストを取得または作成
        let callbacks = self.callbacks.entry(type_id).or_insert_with(Vec::new);
        
        // コールバックを追加
        let index = callbacks.len();
        callbacks.push(callback);
        
        // サブスクリプションIDを生成
        let id = self.next_id;
        self.next_id += 1;
        
        // サブスクリプションマップに登録
        self.subscription_map.insert(id, (type_id, index));
        
        id
    }
    
    fn unsubscribe(&mut self, id: usize) -> bool {
        // サブスクリプションマップからエントリを削除
        if let Some((type_id, index)) = self.subscription_map.remove(&id) {
            // イベントタイプに対応するコールバックリストを取得
            if let Some(callbacks) = self.callbacks.get_mut(&type_id) {
                // インデックスが範囲内かチェック
                if index < callbacks.len() {
                    // 最後のコールバックを削除するインデックスに移動して、ポップ
                    if index < callbacks.len() - 1 {
                        callbacks.swap(index, callbacks.len() - 1);
                        
                        // スワップしたコールバックのサブスクリプションIDを更新
                        for (sub_id, (sub_type_id, sub_index)) in &mut self.subscription_map {
                            if *sub_type_id == type_id && *sub_index == callbacks.len() - 1 {
                                *sub_index = index;
                                break;
                            }
                        }
                    }
                    
                    callbacks.pop();
                    return true;
                }
            }
        }
        
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::impl_event;
    
    #[derive(Debug, Clone)]
    struct TestEvent {
        pub value: i32,
    }
    
    impl_event!(TestEvent, "TestEvent");
    
    #[test]
    fn test_event_dispatcher_basic() {
        let mut dispatcher = EventDispatcher::new();
        let mut entities = EntityManager::new();
        let mut handled = false;
        
        // イベントを購読
        let id = dispatcher.subscribe::<TestEvent, _>(|event, _| {
            assert_eq!(event.value, 42);
            handled = true;
        });
        
        // イベントを発行
        dispatcher.publish(TestEvent { value: 42 }, &mut entities);
        
        // ハンドラーが呼ばれたことを確認
        assert!(handled);
        
        // サブスクリプションを解除
        assert!(dispatcher.unsubscribe(id));
        
        // 解除後は無効なIDになる
        assert!(!dispatcher.unsubscribe(id));
    }
    
    #[test]
    fn test_event_queue() {
        let mut dispatcher = EventDispatcher::new();
        let mut entities = EntityManager::new();
        let mut count = 0;
        
        // イベントを購読
        dispatcher.subscribe::<TestEvent, _>(|event, _| {
            count += event.value;
        });
        
        // イベントをキューに追加
        dispatcher.enqueue(TestEvent { value: 10 });
        dispatcher.enqueue(TestEvent { value: 20 });
        dispatcher.enqueue(TestEvent { value: 30 });
        
        // キューの長さを確認
        assert_eq!(dispatcher.queue_len(), 3);
        
        // キューを処理
        dispatcher.process_queue(&mut entities);
        
        // カウントが正しく増えたことを確認
        assert_eq!(count, 60);
        
        // キューが空になったことを確認
        assert_eq!(dispatcher.queue_len(), 0);
    }
} 