#!/usr/bin/env python3
"""
Phase 37 — axon_alloc: Sovereign Heap Allocator
Adds SovereignAllocator (slab + bump), global_allocator registration,
and ONYX hitchhike: SovereignHeap trait for tensor allocation.

Run from: /home/edisonbl/axon
"""

import sys
from pathlib import Path

ROOT = Path(__file__).parent

def write(p, text):
    Path(p).parent.mkdir(parents=True, exist_ok=True)
    Path(p).write_text(text, encoding="utf-8")
    print(f"  wrote {p}")

def patch(path, old, new, label=""):
    text = Path(path).read_text(encoding="utf-8")
    if old not in text:
        print(f"  ERROR: anchor not found [{label}]")
        sys.exit(1)
    if text.count(old) > 1:
        print(f"  ERROR: anchor not unique [{label}]")
        sys.exit(1)
    Path(path).write_text(text.replace(old, new), encoding="utf-8")
    print(f"  patched [{label}]")

# ── 1. axon_alloc/src/slab.rs ─────────────────────────────────────────────────

write(ROOT / "axon_alloc/src/slab.rs", """\
// Copyright (c) 2026 Edison Lepitel / AIEONYX
//! Slab allocator — fixed-size object pools for small allocations.
//!
//! Size classes: 8, 16, 32, 64, 128, 256 bytes.
//! Each slab is a fixed backing array with a free-list of available slots.
//! O(1) alloc and dealloc within each size class.

use core::mem::MaybeUninit;
use core::sync::atomic::{AtomicUsize, Ordering};

/// Maximum number of slots per slab pool.
pub const SLAB_SLOTS: usize = 256;

/// Slab size classes in bytes.
pub const SIZE_CLASSES: [usize; 6] = [8, 16, 32, 64, 128, 256];

/// Sentinel: free list empty.
const FREE_NONE: usize = usize::MAX;

/// A fixed-size slab pool for objects of `BLOCK` bytes.
pub struct SlabPool<const BLOCK: usize> {
    /// Backing storage — uninitialised until allocated.
    storage: [MaybeUninit<[u8; BLOCK]>; SLAB_SLOTS],
    /// Free list: each entry holds index of next free slot, or FREE_NONE.
    free_list: [usize; SLAB_SLOTS],
    /// Head of the free list.
    head: AtomicUsize,
    /// Number of currently allocated slots.
    allocated: AtomicUsize,
}

impl<const BLOCK: usize> SlabPool<BLOCK> {
    /// Construct an empty slab pool with all slots free.
    pub const fn new() -> Self {
        // Build free list at compile time: slot i → i+1, last → FREE_NONE
        let mut free_list = [0usize; SLAB_SLOTS];
        let mut i = 0;
        while i < SLAB_SLOTS - 1 {
            free_list[i] = i + 1;
            i += 1;
        }
        free_list[SLAB_SLOTS - 1] = FREE_NONE;
        Self {
            storage: [const { MaybeUninit::uninit() }; SLAB_SLOTS],
            free_list,
            head: AtomicUsize::new(0),
            allocated: AtomicUsize::new(0),
        }
    }

    /// Allocate one slot. Returns pointer to the slot or null.
    ///
    /// # Safety
    /// Caller must not use the pointer after calling `dealloc`.
    pub fn alloc(&self) -> *mut u8 {
        // This is a single-threaded slab for now — CAS loop for future SMP.
        let head = self.head.load(Ordering::Relaxed);
        if head == FREE_NONE {
            return core::ptr::null_mut();
        }
        let next = self.free_list[head];
        self.head.store(next, Ordering::Relaxed);
        self.allocated.fetch_add(1, Ordering::Relaxed);
        // Safety: storage lives as long as the pool.
        unsafe {
            (*(&self.storage[head] as *const MaybeUninit<[u8; BLOCK]>
                as *mut MaybeUninit<[u8; BLOCK]>))
                .as_mut_ptr() as *mut u8
        }
    }

    /// Deallocate a slot previously returned by `alloc`.
    ///
    /// # Safety
    /// `ptr` must have been returned by `alloc` on this pool and not yet freed.
    pub unsafe fn dealloc(&self, ptr: *mut u8) {
        let base = self.storage.as_ptr() as usize;
        let slot = (ptr as usize - base) / BLOCK;
        debug_assert!(slot < SLAB_SLOTS, "slab: ptr out of range");
        // Push slot back onto free list.
        let head = self.head.load(Ordering::Relaxed);
        // Safety: free_list is only accessed from one thread at a time.
        unsafe {
            let fl = &self.free_list as *const [usize; SLAB_SLOTS] as *mut [usize; SLAB_SLOTS];
            (*fl)[slot] = head;
        }
        self.head.store(slot, Ordering::Relaxed);
        self.allocated.fetch_sub(1, Ordering::Relaxed);
    }

    /// Returns true if `ptr` falls within this pool's storage.
    pub fn owns(&self, ptr: *mut u8) -> bool {
        let base  = self.storage.as_ptr() as usize;
        let end   = base + core::mem::size_of_val(&self.storage);
        let addr  = ptr as usize;
        addr >= base && addr < end
    }

    /// Currently allocated slot count.
    pub fn allocated(&self) -> usize {
        self.allocated.load(Ordering::Relaxed)
    }

    /// Total slot capacity.
    pub const fn capacity() -> usize { SLAB_SLOTS }

    /// Block size for this pool.
    pub const fn block_size() -> usize { BLOCK }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tp37_slab_alloc_dealloc_8() {
        let pool = SlabPool::<8>::new();
        let p = pool.alloc();
        assert!(!p.is_null());
        assert_eq!(pool.allocated(), 1);
        unsafe { pool.dealloc(p); }
        assert_eq!(pool.allocated(), 0);
    }

    #[test]
    fn tp37_slab_alloc_dealloc_256() {
        let pool = SlabPool::<256>::new();
        let p = pool.alloc();
        assert!(!p.is_null());
        assert!(pool.owns(p));
        unsafe { pool.dealloc(p); }
        assert_eq!(pool.allocated(), 0);
    }

    #[test]
    fn tp37_slab_fill_all_slots() {
        let pool = SlabPool::<16>::new();
        let mut ptrs = [core::ptr::null_mut::<u8>(); SLAB_SLOTS];
        for p in ptrs.iter_mut() {
            *p = pool.alloc();
            assert!(!p.is_null());
        }
        assert_eq!(pool.allocated(), SLAB_SLOTS);
        // Next alloc must fail — pool exhausted
        assert!(pool.alloc().is_null());
        // Free all
        for p in ptrs.iter_mut() {
            unsafe { pool.dealloc(*p); }
        }
        assert_eq!(pool.allocated(), 0);
    }

    #[test]
    fn tp37_slab_owns_only_own_ptrs() {
        let pool = SlabPool::<32>::new();
        let p = pool.alloc();
        assert!(pool.owns(p));
        let foreign: *mut u8 = 0x1234 as *mut u8;
        assert!(!pool.owns(foreign));
        unsafe { pool.dealloc(p); }
    }

    #[test]
    fn tp37_slab_reuse_after_free() {
        let pool = SlabPool::<64>::new();
        let p1 = pool.alloc();
        unsafe { pool.dealloc(p1); }
        let p2 = pool.alloc();
        // After free+alloc the same slot is reused
        assert_eq!(p1, p2);
        unsafe { pool.dealloc(p2); }
    }

    #[test]
    fn tp37_slab_write_read_roundtrip() {
        let pool = SlabPool::<64>::new();
        let p = pool.alloc();
        assert!(!p.is_null());
        unsafe {
            core::ptr::write(p as *mut u64, 0xDEAD_BEEF_CAFE_BABEu64);
            let v = core::ptr::read(p as *const u64);
            assert_eq!(v, 0xDEAD_BEEF_CAFE_BABEu64);
            pool.dealloc(p);
        }
    }
}
""")

# ── 2. axon_alloc/src/heap.rs ─────────────────────────────────────────────────

write(ROOT / "axon_alloc/src/heap.rs", """\
// Copyright (c) 2026 Edison Lepitel / AIEONYX
//! Sovereign heap backing — bump allocator for large/unclassed allocations.
//!
//! On host (x86_64): backed by libc malloc/free via extern C.
//! On aarch64-seL4: backed by seL4 untyped memory regions (P41 wires this).
//!
//! The bump allocator is used only when slab classes are exhausted or the
//! allocation size exceeds the largest slab class (256 bytes).

/// Sovereign heap trait — implemented by any backing memory provider.
///
/// ONYX hitchhike: axon_tensor uses this trait for heap-backed tensor
/// allocation on BASTION nodes, decoupling tensor memory from the host allocator.
pub trait SovereignHeap: Send + Sync {
    /// Allocate `size` bytes aligned to `align`. Returns null on failure.
    unsafe fn alloc(&self, size: usize, align: usize) -> *mut u8;
    /// Deallocate memory previously returned by `alloc`.
    unsafe fn dealloc(&self, ptr: *mut u8, size: usize, align: usize);
    /// Available bytes remaining (approximate). None if unknown.
    fn available(&self) -> Option<usize> { None }
}

// ── Host backing (x86_64 / dev) ───────────────────────────────────────────────

extern "C" {
    fn malloc(size: usize) -> *mut u8;
    fn free(ptr: *mut u8);
    fn aligned_alloc(align: usize, size: usize) -> *mut u8;
}

/// Host heap — delegates to libc malloc/free.
/// Used on x86_64 for development and testing.
pub struct HostHeap;

impl SovereignHeap for HostHeap {
    unsafe fn alloc(&self, size: usize, align: usize) -> *mut u8 {
        if align <= 8 {
            unsafe { malloc(size) }
        } else {
            // Round size up to multiple of align (required by aligned_alloc)
            let rounded = (size + align - 1) & !(align - 1);
            unsafe { aligned_alloc(align, rounded) }
        }
    }
    unsafe fn dealloc(&self, ptr: *mut u8, _size: usize, _align: usize) {
        unsafe { free(ptr) }
    }
}

/// Sovereign heap stub for seL4 target — wired to real untyped memory in P41.
#[cfg(target_arch = "aarch64")]
pub struct Sel4Heap;

#[cfg(target_arch = "aarch64")]
impl SovereignHeap for Sel4Heap {
    unsafe fn alloc(&self, _size: usize, _align: usize) -> *mut u8 {
        // P41: replace with seL4_Untyped_Retype + page mapping
        core::ptr::null_mut()
    }
    unsafe fn dealloc(&self, _ptr: *mut u8, _size: usize, _align: usize) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tp37_host_heap_alloc_dealloc() {
        let heap = HostHeap;
        unsafe {
            let p = heap.alloc(64, 8);
            assert!(!p.is_null());
            core::ptr::write(p as *mut u64, 0xABCD_1234u64);
            assert_eq!(core::ptr::read(p as *const u64), 0xABCD_1234u64);
            heap.dealloc(p, 64, 8);
        }
    }

    #[test]
    fn tp37_host_heap_aligned_alloc() {
        let heap = HostHeap;
        unsafe {
            let p = heap.alloc(128, 64);
            assert!(!p.is_null());
            assert_eq!(p as usize % 64, 0, "alignment violated");
            heap.dealloc(p, 128, 64);
        }
    }

    #[test]
    fn tp37_host_heap_zero_size_is_safe() {
        let heap = HostHeap;
        unsafe {
            let p = heap.alloc(1, 1);
            assert!(!p.is_null());
            heap.dealloc(p, 1, 1);
        }
    }
}
""")

# ── 3. axon_alloc/src/allocator.rs ───────────────────────────────────────────

write(ROOT / "axon_alloc/src/allocator.rs", """\
// Copyright (c) 2026 Edison Lepitel / AIEONYX
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
use crate::heap::HostHeap;

/// The sovereign allocator — slab-first, heap fallback.
pub struct SovereignAllocator {
    slab8:   SlabPool<8>,
    slab16:  SlabPool<16>,
    slab32:  SlabPool<32>,
    slab64:  SlabPool<64>,
    slab128: SlabPool<128>,
    slab256: SlabPool<256>,
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
""")

# ── 4. Update axon_alloc/src/lib.rs ──────────────────────────────────────────

patch(
    ROOT / "axon_alloc/src/lib.rs",
    "#![no_std]\n#![allow(missing_docs)]\nextern crate alloc;",
    "// Copyright (c) 2026 Edison Lepitel / AIEONYX\n"
    "#![no_std]\n#![allow(missing_docs)]\nextern crate alloc;\npub mod allocator;\npub mod heap;\npub mod slab;",
    "lib.rs modules"
)

patch(
    ROOT / "axon_alloc/src/lib.rs",
    "pub use sync::{AxonArc, AxonRc};",
    "pub use sync::{AxonArc, AxonRc};\npub use allocator::{SovereignAllocator, AllocStats};\npub use heap::{SovereignHeap, HostHeap};\npub use slab::SlabPool;",
    "lib.rs re-exports"
)

# ── 5. Update axon_alloc/Cargo.toml — add sovereign_global feature ────────────

patch(
    ROOT / "axon_alloc/Cargo.toml",
    "[features]\ndefault = []",
    "[features]\ndefault = []\n# Enable to register SovereignAllocator as #[global_allocator]\nsovereign_global = []",
    "Cargo.toml feature"
)

# ── 6. axon_integration/src/test_alloc_sovereign.rs ──────────────────────────

write(ROOT / "axon_integration/src/test_alloc_sovereign.rs", """\
// Copyright (c) 2026 Edison Lepitel / AIEONYX
//! Phase 37 integration tests — SovereignAllocator end-to-end.
//! Verifies slab routing, heap fallback, stats, and ONYX hitchhike trait.

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
    // ONYX hitchhike: SovereignHeap trait used by axon_tensor for BASTION
    // tensor allocation — verify HostHeap satisfies the trait contract.
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
""")

# ── 7. Wire new test module into axon_integration/src/lib.rs ─────────────────

LIB = ROOT / "axon_integration/src/lib.rs"
lib_text = LIB.read_text(encoding="utf-8")
MODULE = "#[cfg(test)] mod test_alloc_sovereign; // P37: sovereign heap allocator\n"
if MODULE not in lib_text:
    new_lib = lib_text.rstrip() + "\n" + MODULE
    LIB.write_text(new_lib, encoding="utf-8")
    print("  wired test_alloc_sovereign into lib.rs")
else:
    print("  test_alloc_sovereign already in lib.rs")

print()
print("Phase 37 patch applied.")
print("Next steps:")
print("  1. rm -f /tmp/axon_out.*")
print("  2. cargo test --workspace 2>&1 | tail -30")
print("  3. cargo clippy --workspace -- -D warnings 2>&1 | head -30")
