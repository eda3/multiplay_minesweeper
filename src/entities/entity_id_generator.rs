/**
 * エンティティIDジェネレーター
 * 
 * エンティティの一意なIDを効率的に生成・管理するためのジェネレーター
 */
use crate::entities::entity::EntityId;

/// エンティティIDジェネレーター
/// 一意のIDを生成し、削除されたIDを必要に応じて再利用する
#[derive(Debug)]
pub struct EntityIdGenerator {
    /// 次に割り当てるエンティティID
    next_id: u64,
    /// 再利用可能なID
    recycled_ids: Vec<EntityId>,
    /// 削除されたIDを再利用するかどうか
    use_recycled: bool,
}

impl EntityIdGenerator {
    /// 新しいエンティティIDジェネレーターを作成
    pub fn new(use_recycled: bool) -> Self {
        Self {
            next_id: 1, // 0は無効なIDとして予約
            recycled_ids: Vec::new(),
            use_recycled,
        }
    }
    
    /// デフォルト設定のジェネレーターを作成（ID再利用あり）
    pub fn default() -> Self {
        Self::new(true)
    }
    
    /// 新しいIDを生成
    pub fn generate(&mut self) -> EntityId {
        // 再利用設定がオンで、再利用可能なIDがある場合はそれを使用
        if self.use_recycled && !self.recycled_ids.is_empty() {
            return self.recycled_ids.pop().unwrap();
        }
        
        // 新しいIDを生成
        let id = EntityId(self.next_id);
        self.next_id += 1;
        id
    }
    
    /// IDを再利用可能としてマーク
    pub fn recycle(&mut self, id: EntityId) {
        if self.use_recycled {
            self.recycled_ids.push(id);
        }
    }
    
    /// 再利用可能なID数を取得
    pub fn recycled_count(&self) -> usize {
        self.recycled_ids.len()
    }
    
    /// これまでに生成した最大のID値を取得
    pub fn max_id(&self) -> u64 {
        self.next_id - 1
    }
    
    /// ID再利用の設定を変更
    pub fn set_recycling(&mut self, use_recycled: bool) {
        self.use_recycled = use_recycled;
        
        // 再利用をオフにする場合、再利用リストをクリア
        if !use_recycled {
            self.recycled_ids.clear();
        }
    }
} 