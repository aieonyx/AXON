// Copyright (c) 2026 Edison Lepiten / AIEONYX
//! SovereignAllocator — global allocator for AXON/BASTION nodes.
//!
//! Strategy:
//!   - size <= 8:   SlabPool<8>
//!   - size <= 16:  SlabPool<16>
//!   - size <= 32:  SlabPool<32>
//!   - size <= 64:  SlabPool<64>
//!   - size <= 128: SlabPool<128>
//!   - size <= 256: SlabPool<256>
//!   - size > 256:  HostHeap (fallback to libc on host, seL4 untyped on target)
//!
//! The allocator is registered as #[global_allocator] when the
//! `sovereign_global` feature is enabled, replacing the default allocator.

use core::alloc::{GlobalAlloc, Layout};
use crate::slab::SlabPool;
use crate::heap::{HostHeap, SovereignHeap};

/// The sovereign allocator — slab-first, heap fallback.
pub struct SovereignAllocator {
    pub slab8:   SlabPool<8>,
    pub slab16:  SlabPool<16>,
    pub slab32:  SlabPool<32>,
    pub slab64:  SlabPool<64>,
    pub slab128: SlabPool<128>,
    pub slab256: SlabPool<256>,
    heap:    HostHeap,
}

impl SovereignAllocator {
    pub const fn new() -> Self {
        Self {
            slab8:   SlabPool::new(),
            slab16:  SlabPool::new(),
            slab32:  SlabPool::new(),
            slab64:  SlabPool::new(),
            slab128: SlabPool::new(),
            slab256: SlabPool::new(),
            heap:    HostHeap,
        }
    }

    /// Allocation statistics — useful for BASTION telemetry.
    pub fn stats(&self) -> AllocStats {
        AllocStats {
            slab8:   self.slab8.allocated(),
            slab16:  self.slab16.allocated(),
            slab32:  self.slab32.allocated(),
            slab64:  self.slab64.allocated(),
            slab128: self.slab128.allocated(),
            slab256: self.slab256.allocated(),
        }
    }
}

/// Snapshot of per-slab allocation counts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AllocStats {
    pub slab8:   usize,
    pub slab16:  usize,
    pub slab32:  usize,
    pub slab64:  usize,
    pub slab128: usize,
    pub slab256: usize,
}

impl AllocStats {
    pub fn total_slab_allocations(&self) -> usize {
        self.slab8 + self.slab16 + self.slab32
            + self.slab64 + self.slab128 + self.slab256
    }
}

impl Default for SovereignAllocator {
    fn default() -> Self { Self::new() }
}

unsafe impl GlobalAlloc for SovereignAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let size = layout.size();
        // Route to smallest fitting slab class first.
        if size <= 8   { let p = self.slab8.alloc();   if !p.is_null() { return p; } }
        if size <= 16  { let p = self.slab16.alloc();  if !p.is_null() { return p; } }
        if size <= 32  { let p = self.slab32.alloc();  if !p.is_null() { return p; } }
        if size <= 64  { let p = self.slab64.alloc();  if !p.is_null() { return p; } }
        if size <= 128 { let p = self.slab128.alloc(); if !p.is_null() { return p; } }
        if size <= 256 { let p = self.slab256.alloc(); if !p.is_null() { return p; } }
        // Fallback: heap for large or slab-exhausted allocations.
        unsafe { self.heap.alloc(size, layout.align()) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        // Route dealloc to owning slab or heap.
        if self.slab8.owns(ptr)   { unsafe { self.slab8.dealloc(ptr);   return; } }
        if self.slab16.owns(ptr)  { unsafe { self.slab16.dealloc(ptr);  return; } }
        if self.slab32.owns(ptr)  { unsafe { self.slab32.dealloc(ptr);  return; } }
        if self.slab64.owns(ptr)  { unsafe { self.slab64.dealloc(ptr);  return; } }
        if self.slab128.owns(ptr) { unsafe { self.slab128.dealloc(ptr); return; } }
        if self.slab256.owns(ptr) { unsafe { self.slab256.dealloc(ptr); return; } }
        unsafe { self.heap.dealloc(ptr, layout.size(), layout.align()); }
    }
}

// ── Global allocator registration ────────────────────────────────────────────
// Enabled via `sovereign_global` feature to avoid conflicting with test harness.

#[cfg(feature = "sovereign_global")]
#[global_allocator]
static SOVEREIGN_ALLOC: SovereignAllocator = SovereignAllocator::new();

#[cfg(test)]
mod tests {
    use super::*;
    use core::alloc::Layout;

    fn make_alloc() -> SovereignAllocator { SovereignAllocator::new() }

    #[test]
    fn tp37_alloc_small_routes_to_slab() {
        let a = make_alloc();
        let layout = Layout::from_size_align(8, 1).unwrap();
        let p = unsafe { a.alloc(layout) };
        assert!(!p.is_null());
        assert!(a.slab8.owns(p));
        assert_eq!(a.stats().slab8, 1);
        unsafe { a.dealloc(p, layout); }
        assert_eq!(a.stats().slab8, 0);
    }

    #[test]
    fn tp37_alloc_16_routes_to_slab16() {
        let a = make_alloc();
        let layout = Layout::from_size_align(16, 1).unwrap();
        let p = unsafe { a.alloc(layout) };
        assert!(!p.is_null());
        assert!(a.slab16.owns(p));
        unsafe { a.dealloc(p, layout); }
    }

    #[test]
    fn tp37_alloc_large_routes_to_heap() {
        let a = make_alloc();
        let layout = Layout::from_size_align(512, 8).unwrap();
        let p = unsafe { a.alloc(layout) };
        assert!(!p.is_null());
        // Not owned by any slab
        assert!(!a.slab256.owns(p));
        unsafe { a.dealloc(p, layout); }
    }

    #[test]
    fn tp37_alloc_stats_track_correctly() {
        let a = make_alloc();
        let l8  = Layout::from_size_align(8,  1).unwrap();
        let l32 = Layout::from_size_align(32, 1).unwrap();
        let p1 = unsafe { a.alloc(l8) };
        let p2 = unsafe { a.alloc(l32) };
        let stats = a.stats();
        assert_eq!(stats.slab8,  1);
        assert_eq!(stats.slab32, 1);
        assert_eq!(stats.total_slab_allocations(), 2);
        unsafe { a.dealloc(p1, l8); a.dealloc(p2, l32); }
        assert_eq!(a.stats().total_slab_allocations(), 0);
    }

    #[test]
    fn tp37_alloc_write_read_8() {
        let a = make_alloc();
        let layout = Layout::from_size_align(8, 1).unwrap();
        let p = unsafe { a.alloc(layout) };
        unsafe {
            core::ptr::write(p as *mut u64, 0x5AFE_5AFE_5AFE_5AFEu64);
            assert_eq!(core::ptr::read(p as *const u64), 0x5AFE_5AFE_5AFE_5AFEu64);
            a.dealloc(p, layout);
        }
    }

    #[test]
    fn tp37_alloc_slab_exhaustion_falls_to_heap() {
        let a = make_alloc();
        let layout = Layout::from_size_align(8, 1).unwrap();
        // Exhaust all 256 slab8 slots
        let mut ptrs = [core::ptr::null_mut::<u8>(); 256];
        for p in ptrs.iter_mut() { *p = unsafe { a.alloc(layout) }; }
        // Next alloc must come from heap
        let overflow = unsafe { a.alloc(layout) };
        assert!(!overflow.is_null());
        assert!(!a.slab8.owns(overflow));
        // Cleanup
        unsafe { a.dealloc(overflow, layout); }
        for p in ptrs.iter_mut() { unsafe { a.dealloc(*p, layout); } }
    }

    #[test]
    fn tp37_alloc_boundary_sizes() {
        let a = make_alloc();
        for &size in &[1usize, 8, 9, 16, 17, 32, 33, 64, 65, 128, 129, 256, 257, 1024] {
            let layout = Layout::from_size_align(size, 1).unwrap();
            let p = unsafe { a.alloc(layout) };
            assert!(!p.is_null(), "alloc failed for size {}", size);
            unsafe { a.dealloc(p, layout); }
        }
    }

    #[test]
    fn tp37_alloc_stats_zero_after_full_cycle() {
        let a = make_alloc();
        let layouts: &[Layout] = &[
            Layout::from_size_align(8,   1).unwrap(),
            Layout::from_size_align(16,  1).unwrap(),
            Layout::from_size_align(32,  1).unwrap(),
            Layout::from_size_align(64,  1).unwrap(),
            Layout::from_size_align(128, 1).unwrap(),
            Layout::from_size_align(256, 1).unwrap(),
        ];
        let mut ptrs = [core::ptr::null_mut::<u8>(); 6];
        for (i, layout) in layouts.iter().enumerate() {
            ptrs[i] = unsafe { a.alloc(*layout) };
        }
        assert_eq!(a.stats().total_slab_allocations(), 6);
        for (i, layout) in layouts.iter().enumerate() {
            unsafe { a.dealloc(ptrs[i], *layout); }
        }
        assert_eq!(a.stats().total_slab_allocations(), 0);
    }
}
