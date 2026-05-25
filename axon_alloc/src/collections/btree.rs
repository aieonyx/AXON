//! AxonBTreeMap<K,V> — ordered map, no hasher required.
pub type AxonBTreeMap<K, V> = alloc::collections::BTreeMap<K, V>;

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn btree_insert_and_get() {
        let mut m: AxonBTreeMap<i32,&str> = AxonBTreeMap::new();
        m.insert(1,"one"); m.insert(2,"two"); m.insert(3,"three");
        assert_eq!(m.get(&2), Some(&"two"));
    }
    #[test] fn btree_sorted_iteration() {
        let mut m: AxonBTreeMap<i32,i32> = AxonBTreeMap::new();
        m.insert(3,30); m.insert(1,10); m.insert(2,20);
        let keys: alloc::vec::Vec<i32> = m.keys().copied().collect();
        assert_eq!(keys, alloc::vec![1,2,3]);
    }
    #[test] fn btree_remove() {
        let mut m: AxonBTreeMap<i32,i32> = AxonBTreeMap::new();
        m.insert(42,99); assert_eq!(m.remove(&42), Some(99)); assert!(m.is_empty());
    }
    #[test] fn btree_range() {
        let mut m: AxonBTreeMap<i32,i32> = AxonBTreeMap::new();
        for i in 0..10 { m.insert(i, i*10); }
        let range: alloc::vec::Vec<(i32,i32)> = m.range(3..6).map(|(&k,&v)|(k,v)).collect();
        assert_eq!(range, alloc::vec![(3,30),(4,40),(5,50)]);
    }
    #[test] fn btree_contains_key() {
        let mut m: AxonBTreeMap<&str,u8> = AxonBTreeMap::new();
        m.insert("axon",1); assert!(m.contains_key("axon")); assert!(!m.contains_key("x"));
    }
    #[test] fn btree_first_last() {
        let mut m: AxonBTreeMap<i32,i32> = AxonBTreeMap::new();
        m.insert(5,50); m.insert(1,10); m.insert(9,90);
        assert_eq!(m.keys().next().copied(), Some(1));
        assert_eq!(m.keys().last().copied(), Some(9));
    }
}
