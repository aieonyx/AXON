// Copyright (c) 2026 Edison Lepiten / AIEONYX
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
    /// Returns null if the pool is exhausted.
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

impl<const BLOCK: usize> Default for SlabPool<BLOCK> {
    fn default() -> Self { Self::new() }
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
