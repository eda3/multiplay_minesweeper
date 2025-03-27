/**
 * エンティティ関連のテスト
 */
use super::*;

#[cfg(test)]
mod entity_manager_tests {
    use super::*;
    
    #[test]
    fn test_entity_id_generator() {
        let mut generator = EntityIdGenerator::new(true);
        
        // 基本的なID生成
        let id1 = generator.generate();
        let id2 = generator.generate();
        assert_ne!(id1, id2); // IDが異なることを確認
        
        // ID再利用
        generator.recycle(id1);
        let id3 = generator.generate();
        assert_eq!(id1, id3); // 再利用されたIDが返されることを確認
        
        // 再利用リストが空の場合
        let id4 = generator.generate();
        assert_ne!(id2, id4);
        assert_ne!(id3, id4);
    }
    
    #[test]
    fn test_batch_entity_creation() {
        let mut manager = EntityManager::new();
        
        // バッチでエンティティを作成
        let count = 10;
        let ids = manager.create_entities(count);
        
        // 正しい数のエンティティが作成されたか確認
        assert_eq!(ids.len(), count);
        assert_eq!(manager.entity_count(), count);
        
        // すべてのIDが一意であることを確認
        let mut unique_ids = std::collections::HashSet::new();
        for id in &ids {
            unique_ids.insert(*id);
        }
        assert_eq!(unique_ids.len(), count);
    }
    
    #[test]
    fn test_batch_entity_removal() {
        let mut manager = EntityManager::new();
        
        // エンティティを作成
        let ids = manager.create_entities(5);
        assert_eq!(manager.entity_count(), 5);
        
        // 一部のエンティティを削除
        let to_remove = vec![ids[0], ids[2], ids[4]];
        manager.remove_entities(to_remove.clone());
        
        // この時点ではまだ削除されていない
        assert_eq!(manager.entity_count(), 5);
        
        // 削除を確定
        manager.flush_removals();
        
        // 正しく削除されたか確認
        assert_eq!(manager.entity_count(), 2);
        assert!(manager.get_entity(ids[1]).is_some());
        assert!(manager.get_entity(ids[3]).is_some());
        
        // 削除されたエンティティが存在しないことを確認
        for id in to_remove {
            assert!(manager.get_entity(id).is_none());
        }
    }
    
    #[test]
    fn test_component_index_building() {
        let mut manager = EntityManager::new();
        
        // テスト用のコンポーネント
        #[derive(Debug)]
        struct TestComponent(i32);
        
        // 異なるコンポーネントを持つエンティティを作成
        let id1 = manager.create_entity();
        let id2 = manager.create_entity();
        let id3 = manager.create_entity();
        
        manager.get_entity_mut(id1).unwrap().add_component(TestComponent(1));
        manager.get_entity_mut(id2).unwrap().add_component(TestComponent(2));
        // id3にはTestComponentを追加しない
        
        // インデックスを構築
        let indices = manager.build_component_index::<TestComponent>();
        
        // 正しくインデックスが構築されたか確認
        assert_eq!(indices.len(), 2);
        assert!(indices.contains(&id1));
        assert!(indices.contains(&id2));
        assert!(!indices.contains(&id3));
        
        // インデックスを使用したクエリが正しく動作するか確認
        let entities = manager.get_entities_with_component::<TestComponent>();
        assert_eq!(entities.len(), 2);
        assert!(entities.contains(&id1));
        assert!(entities.contains(&id2));
        assert!(!entities.contains(&id3));
    }
    
    #[test]
    fn test_query_with_component_and_tag() {
        let mut manager = EntityManager::new();
        
        // テスト用のコンポーネント
        #[derive(Debug)]
        struct TestComponent(i32);
        
        // エンティティを作成
        let id1 = manager.create_entity();
        let id2 = manager.create_entity();
        let id3 = manager.create_entity();
        let id4 = manager.create_entity();
        
        // エンティティを直接修正する代わりに、Entity作成後にマネージャーに登録する方法を使用
        let mut entity1 = Entity::new(id1);
        entity1.add_component(TestComponent(1));
        entity1.add_tag("tag1");
        manager.register_entity(entity1);
        
        let mut entity2 = Entity::new(id2);
        entity2.add_component(TestComponent(2));
        entity2.add_tag("tag2");
        manager.register_entity(entity2);
        
        let mut entity3 = Entity::new(id3);
        entity3.add_component(TestComponent(3));
        manager.register_entity(entity3);
        
        let mut entity4 = Entity::new(id4);
        entity4.add_tag("tag1");
        manager.register_entity(entity4);
        
        // 複合クエリを実行
        let result = manager.query_with_component_and_tag::<TestComponent>("tag1");
        
        // 結果を検証：id1だけが両方の条件を満たす
        assert_eq!(result.len(), 1);
        assert!(result.contains(&id1));
    }
    
    #[test]
    fn test_hierarchy_relationships() {
        let mut manager = EntityManager::new();
        
        // 親子関係のあるエンティティを作成
        let parent = manager.create_entity();
        let child1 = manager.create_entity();
        let child2 = manager.create_entity();
        
        // 親子関係を設定
        manager.set_parent(child1, parent).unwrap();
        manager.set_parent(child2, parent).unwrap();
        
        // 関係が正しく設定されたか確認
        let parent_entity = manager.get_entity(parent).unwrap();
        let hierarchy = parent_entity.get_component::<Hierarchy>().unwrap();
        
        assert_eq!(hierarchy.children.len(), 2);
        assert!(hierarchy.children.contains(&child1));
        assert!(hierarchy.children.contains(&child2));
        
        // 子の親参照も確認
        let child_entity = manager.get_entity(child1).unwrap();
        let child_hierarchy = child_entity.get_component::<Hierarchy>().unwrap();
        
        assert_eq!(child_hierarchy.parent, Some(parent));
    }
    
    #[test]
    fn test_recursive_entity_removal() {
        let mut manager = EntityManager::new();
        
        // 階層構造を作成
        let parent = manager.create_entity();
        let child1 = manager.create_entity();
        let child2 = manager.create_entity();
        let grandchild = manager.create_entity();
        
        // 親子関係を設定
        manager.set_parent(child1, parent).unwrap();
        manager.set_parent(child2, parent).unwrap();
        manager.set_parent(grandchild, child1).unwrap();
        
        // 再帰的に削除
        manager.remove_entity_recursive(parent);
        manager.flush_removals();
        
        // すべてのエンティティが削除されたか確認
        assert!(manager.get_entity(parent).is_none());
        assert!(manager.get_entity(child1).is_none());
        assert!(manager.get_entity(child2).is_none());
        assert!(manager.get_entity(grandchild).is_none());
    }
} 