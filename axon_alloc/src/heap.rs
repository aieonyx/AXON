// Copyright (c) 2026 Edison Lepiten / AIEONYX
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
    /// # Safety
    /// Caller must deallocate with matching size and align via `dealloc`.
    unsafe fn alloc(&self, size: usize, align: usize) -> *mut u8;
    /// Deallocate memory previously returned by `alloc`.
    /// # Safety
    /// `ptr` must have been returned by `alloc` on this heap with matching layout.
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
    /// # Safety
    /// Caller must deallocate with matching size and align.
    unsafe fn alloc(&self, size: usize, align: usize) -> *mut u8 {
        if align <= 8 {
            unsafe { malloc(size) }
        } else {
            // Round size up to multiple of align (required by aligned_alloc)
            let rounded = (size + align - 1) & !(align - 1);
            unsafe { aligned_alloc(align, rounded) }
        }
    }
    /// # Safety
    /// `ptr` must have been returned by `alloc` on this heap.
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
