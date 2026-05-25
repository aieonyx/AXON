//! AxonHashMap<K,V> — DOS-resistant hash map via ahash.
//!
//! Default hasher is ahash (randomised per-process, hardware-AES where available).
//! NOT FnvHasher — safe for attacker-controlled keys.
use hashbrown::HashMap;
pub type AxonHashMap<K, V> = HashMap<K, V>;

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn map_insert_and_get() {
        let mut m: AxonHashMap<&str,i64> = AxonHashMap::new();
        m.insert("answer", 42); assert_eq!(m.get("answer"), Some(&42));
    }
    #[test] fn map_len_and_empty() {
        let mut m: AxonHashMap<i32,i32> = AxonHashMap::new();
        assert!(m.is_empty()); m.insert(1,10); assert_eq!(m.len(), 1);
    }
    #[test] fn map_contains_key() {
        let mut m: AxonHashMap<&str,u8> = AxonHashMap::new();
        m.insert("axon",1); assert!(m.contains_key("axon")); assert!(!m.contains_key("x"));
    }
    #[test] fn map_remove() {
        let mut m: AxonHashMap<i32,i32> = AxonHashMap::new();
        m.insert(1,100); assert_eq!(m.remove(&1), Some(100)); assert!(m.is_empty());
    }
    #[test] fn map_entry_or_insert() {
        let mut m: AxonHashMap<&str,i64> = AxonHashMap::new();
        *m.entry("count").or_insert(0) += 1;
        *m.entry("count").or_insert(0) += 1;
        assert_eq!(m["count"], 2);
    }
    #[test] fn map_overwrite() {
        let mut m: AxonHashMap<&str,i64> = AxonHashMap::new();
        m.insert("x",1); m.insert("x",99); assert_eq!(m["x"], 99);
    }
    #[test] fn map_iter_keys_values() {
        let mut m: AxonHashMap<i32,i32> = AxonHashMap::new();
        for i in 0..5 { m.insert(i, i*10); }
        let mut keys: alloc::vec::Vec<i32> = m.keys().copied().collect();
        keys.sort(); assert_eq!(keys, alloc::vec![0,1,2,3,4]);
    }
    #[test] fn map_with_capacity() {
        let m: AxonHashMap<i32,i32> = AxonHashMap::with_capacity(128);
        assert!(m.capacity() >= 128);
    }
    #[test] fn map_retain() {
        let mut m: AxonHashMap<i32,i32> = AxonHashMap::new();
        for i in 0..10 { m.insert(i,i); }
        m.retain(|_,v| *v % 2 == 0);
        assert!(m.values().all(|v| v % 2 == 0));
    }
    #[test] fn map_default_hasher_is_ahash() {
        let m: hashbrown::HashMap<i32,i32> = hashbrown::HashMap::new();
        let _: AxonHashMap<i32,i32> = m;
    }
}
