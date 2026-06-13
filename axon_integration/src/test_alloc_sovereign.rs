// Copyright (c) 2026 Edison Lepiten / AIEONYX
//! Phase 37 integration tests — SovereignAllocator end-to-end.

use axon_alloc::{SovereignAllocator, AllocStats, SovereignHeap, HostHeap};
use core::alloc::{GlobalAlloc, Layout};

fn alloc() -> SovereignAllocator { SovereignAllocator::new() }

#[test]
fn p37_sovereign_alloc_small_slab_route() {
    let a = alloc();
    let l = Layout::from_size_align(8, 1).unwrap();
    let p = unsafe { a.alloc(l) };
    assert!(!p.is_null());
    assert!(a.slab8.owns(p));
    unsafe { a.dealloc(p, l); }
}

#[test]
fn p37_sovereign_alloc_all_slab_classes() {
    let a = alloc();
    for &size in &[8usize, 16, 32, 64, 128, 256] {
        let l = Layout::from_size_align(size, 1).unwrap();
        let p = unsafe { a.alloc(l) };
        assert!(!p.is_null(), "slab alloc failed for size {}", size);
        unsafe { a.dealloc(p, l); }
    }
    assert_eq!(a.stats().total_slab_allocations(), 0);
}

#[test]
fn p37_sovereign_alloc_heap_fallback() {
    let a = alloc();
    let l = Layout::from_size_align(4096, 8).unwrap();
    let p = unsafe { a.alloc(l) };
    assert!(!p.is_null());
    unsafe { a.dealloc(p, l); }
}

#[test]
fn p37_sovereign_alloc_stats_accurate() {
    let a = alloc();
    let l8  = Layout::from_size_align(8,  1).unwrap();
    let l64 = Layout::from_size_align(64, 1).unwrap();
    let p1 = unsafe { a.alloc(l8) };
    let p2 = unsafe { a.alloc(l64) };
    assert_eq!(a.stats(), AllocStats {
        slab8: 1, slab16: 0, slab32: 0,
        slab64: 1, slab128: 0, slab256: 0,
    });
    unsafe { a.dealloc(p1, l8); a.dealloc(p2, l64); }
    assert_eq!(a.stats().total_slab_allocations(), 0);
}

#[test]
fn p37_onyx_hitchhike_sovereign_heap_trait() {
    fn alloc_via_trait(heap: &dyn SovereignHeap, size: usize) -> *mut u8 {
        unsafe { heap.alloc(size, 8) }
    }
    fn dealloc_via_trait(heap: &dyn SovereignHeap, ptr: *mut u8, size: usize) {
        unsafe { heap.dealloc(ptr, size, 8); }
    }
    let heap = HostHeap;
    let p = alloc_via_trait(&heap, 256);
    assert!(!p.is_null());
    dealloc_via_trait(&heap, p, 256);
}

#[test]
fn p37_sovereign_alloc_write_read_all_classes() {
    let a = alloc();
    for &size in &[8usize, 16, 32, 64, 128, 256] {
        let l = Layout::from_size_align(size, 1).unwrap();
        let p = unsafe { a.alloc(l) };
        assert!(!p.is_null());
        unsafe {
            core::ptr::write_bytes(p, 0xAB, size);
            for i in 0..size {
                assert_eq!(*p.add(i), 0xAB, "byte mismatch at {} size {}", i, size);
            }
            a.dealloc(p, l);
        }
    }
}

#[test]
fn p37_sovereign_alloc_boundary_sizes() {
    let a = alloc();
    for &size in &[1usize, 8, 9, 16, 17, 32, 33, 64, 65, 128, 129, 256, 257, 512, 1024, 4096] {
        let l = Layout::from_size_align(size, 1).unwrap();
        let p = unsafe { a.alloc(l) };
        assert!(!p.is_null(), "alloc failed size {}", size);
        unsafe { a.dealloc(p, l); }
    }
}
