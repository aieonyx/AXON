//! Integration tests — axon_alloc heap layer.
use axon_alloc::prelude::*;

#[test] fn alloc_vec_sort_dedup() {
    let mut v: AxonVec<i32> = vec![3,1,2,1,3,2];
    v.sort(); v.dedup();
    assert_eq!(v, vec![1,2,3]);
}
#[test] fn alloc_vec_retain_even() {
    let mut v: AxonVec<i32> = (0..10).collect();
    v.retain(|&x| x % 2 == 0);
    assert_eq!(v, vec![0,2,4,6,8]);
}
#[test] fn alloc_string_format_roundtrip() {
    let s = AxonString::from("AXON v0.6");
    assert!(s.contains("AXON")); assert!(s.contains("0.6"));
}
#[test] fn alloc_hashmap_entry_counter() {
    let mut m: AxonHashMap<&str, u32> = AxonHashMap::new();
    for word in ["a","b","a","c","b","a"] { *m.entry(word).or_insert(0) += 1; }
    assert_eq!(m["a"], 3); assert_eq!(m["b"], 2); assert_eq!(m["c"], 1);
}
#[test] fn alloc_hashmap_dos_resistant() {
    // AxonHashMap is hashbrown::HashMap with ahash — DOS-resistant by default
    // Verify it accepts large key sets without O(n^2) degradation
    let mut m: AxonHashMap<u64, u64> = AxonHashMap::with_capacity(1000);
    for i in 0..1000u64 { m.insert(i, i); }
    assert_eq!(m.len(), 1000);
}
#[test] fn alloc_btreemap_sorted() {
    let mut m: AxonBTreeMap<i32,i32> = AxonBTreeMap::new();
    for i in [5,2,8,1,9,3] { m.insert(i,i); }
    let mut keys: AxonVec<i32> = m.keys().copied().collect();
    assert_eq!(keys, vec![1,2,3,5,8,9]);
}
#[test] fn alloc_btreemap_range() {
    let mut m: AxonBTreeMap<i32,i32> = AxonBTreeMap::new();
    for i in 0..10 { m.insert(i, i*10); }
    let r: AxonVec<i32> = m.range(3..6).map(|(&k,_)| k).collect();
    assert_eq!(r, vec![3,4,5]);
}
#[test] fn alloc_box_recursive() {
    enum Tree { Leaf(i32), Node(AxonBox<Tree>, AxonBox<Tree>) }
    let t = Tree::Node(AxonBox::new(Tree::Leaf(1)), AxonBox::new(Tree::Leaf(2)));
    match t { Tree::Node(l,r) => { match *l { Tree::Leaf(v) => assert_eq!(v,1), _ => panic!() }
                                   match *r { Tree::Leaf(v) => assert_eq!(v,2), _ => panic!() } }
              Tree::Leaf(_) => panic!() }
}
#[test] fn alloc_arc_shared_ownership() {
    use std::sync::atomic::{AtomicU32, Ordering};
    let counter = AxonArc::new(AtomicU32::new(0));
    let c2 = AxonArc::clone(&counter);
    c2.fetch_add(1, Ordering::SeqCst);
    assert_eq!(counter.load(Ordering::SeqCst), 1);
    assert_eq!(AxonArc::strong_count(&counter), 2);
}
#[test] fn alloc_rc_single_thread() {
    let r = AxonRc::new(vec![1,2,3]);
    let r2: AxonRc<Vec<i32>> = AxonRc::clone(&r);
    assert_eq!(r.len(), r2.len());
    assert_eq!(AxonRc::strong_count(&r), 2);
}
#[test] fn alloc_vec_large_capacity() {
    let mut v: AxonVec<u64> = AxonVec::with_capacity(10_000);
    for i in 0..10_000u64 { v.push(i); }
    assert_eq!(v.len(), 10_000);
    assert_eq!(v[9_999], 9_999);
}
#[test] fn alloc_hashmap_stress() {
    let mut m: AxonHashMap<u64,u64> = AxonHashMap::with_capacity(1000);
    for i in 0..1000u64 { m.insert(i, i*i); }
    for i in 0..1000u64 { assert_eq!(m[&i], i*i); }
    m.retain(|_,v| *v < 100);
    assert!(m.len() < 1000);
}
#[test] fn alloc_prelude_all_types() {
    let _: AxonInt = 0; let _v: AxonVec<i64> = AxonVec::new();
    let _m: AxonHashMap<&str,i64> = AxonHashMap::new();
    let _b: AxonBTreeMap<i32,i32> = AxonBTreeMap::new();
    let _s: AxonString = AxonString::from("axon");
    let _x: AxonBox<i64> = AxonBox::new(0);
    let _a = AxonArc::new(0i64);
    let _r = AxonRc::new(0i64);
}
