// Copyright (c) 2026 Edison Lepiten / AIEONYX
//! seL4 BootInfo — boot-time information passed by the kernel to the root task.
//!
//! The seL4 kernel places a BootInfo struct at a fixed virtual address before
//! handing control to the root task. GENESIS reads it to discover:
//!   - Available untyped memory regions
//!   - Initial capability slots
//!   - IPC buffer location
//!
//! Reference: seL4 Reference Manual §11 (BootInfo)

use axon_sel4::types::Cap;

/// Maximum number of untyped memory regions reported at boot.
pub const MAX_UNTYPED_REGIONS: usize = 64;

/// A single untyped memory region from BootInfo.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UntypedRegion {
    /// Capability slot for this untyped region.
    pub cap:       Cap,
    /// Physical address of the region.
    pub paddr:     u64,
    /// Size in bits (region size = 2^size_bits bytes).
    pub size_bits: u8,
    /// True if this region is device memory (MMIO).
    pub is_device: bool,
}

impl UntypedRegion {
    /// Size in bytes.
    pub fn size_bytes(self) -> u64 { 1u64 << self.size_bits }
}

/// Capability slot range — [start, end).
#[derive(Debug, Clone, Copy)]
pub struct CapRange { pub start: Cap, pub end: Cap }

impl CapRange {
    pub fn len(self) -> u64 { self.end - self.start }
    pub fn is_empty(self) -> bool { self.start >= self.end }
    pub fn contains(self, cap: Cap) -> bool { cap >= self.start && cap < self.end }
}

/// seL4 BootInfo — root task's view of the system at boot.
#[derive(Debug)]
pub struct BootInfo {
    /// Empty CNode slots available for capability allocation.
    pub empty: CapRange,
    /// Untyped memory regions.
    pub untyped: [Option<UntypedRegion>; MAX_UNTYPED_REGIONS],
    /// Number of valid untyped regions.
    pub untyped_count: usize,
    /// IPC buffer virtual address.
    pub ipc_buf_vaddr: u64,
}

impl BootInfo {
    /// Construct a BootInfo from raw boot-time data.
    pub const fn new(empty: CapRange, ipc_buf_vaddr: u64) -> Self {
        Self {
            empty,
            untyped: [None; MAX_UNTYPED_REGIONS],
            untyped_count: 0,
            ipc_buf_vaddr,
        }
    }

    /// Register an untyped memory region.
    pub fn add_untyped(&mut self, region: UntypedRegion) -> bool {
        if self.untyped_count >= MAX_UNTYPED_REGIONS { return false; }
        self.untyped[self.untyped_count] = Some(region);
        self.untyped_count += 1;
        true
    }

    /// Iterate over valid untyped regions.
    pub fn untyped_regions(&self) -> impl Iterator<Item = &UntypedRegion> {
        self.untyped[..self.untyped_count]
            .iter()
            .filter_map(|r| r.as_ref())
    }

    /// Total available RAM in untyped regions (non-device).
    pub fn total_ram_bytes(&self) -> u64 {
        self.untyped_regions()
            .filter(|r| !r.is_device)
            .map(|r| r.size_bytes())
            .sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_boot_info() -> BootInfo {
        let mut bi = BootInfo::new(
            CapRange { start: 10, end: 256 },
            0x1000_0000,
        );
        bi.add_untyped(UntypedRegion { cap: 20, paddr: 0x4000_0000, size_bits: 24, is_device: false });
        bi.add_untyped(UntypedRegion { cap: 21, paddr: 0x9000_0000, size_bits: 12, is_device: true  });
        bi
    }

    #[test]
    fn tp41_bootinfo_untyped_count() {
        let bi = make_boot_info();
        assert_eq!(bi.untyped_count, 2);
    }

    #[test]
    fn tp41_bootinfo_total_ram() {
        let bi = make_boot_info();
        // Only non-device: 2^24 = 16MB
        assert_eq!(bi.total_ram_bytes(), 1 << 24);
    }

    #[test]
    fn tp41_bootinfo_cap_range() {
        let bi = make_boot_info();
        assert_eq!(bi.empty.len(), 246);
        assert!(bi.empty.contains(100));
        assert!(!bi.empty.contains(9));
    }

    #[test]
    fn tp41_bootinfo_untyped_size_bytes() {
        let r = UntypedRegion { cap: 1, paddr: 0, size_bits: 20, is_device: false };
        assert_eq!(r.size_bytes(), 1 << 20); // 1MB
    }

    #[test]
    fn tp41_bootinfo_add_untyped_regions() {
        let mut bi = BootInfo::new(CapRange { start: 10, end: 100 }, 0x1000_0000);
        for i in 0..MAX_UNTYPED_REGIONS {
            assert!(bi.add_untyped(UntypedRegion {
                cap: i as u64 + 20, paddr: i as u64 * 0x1000,
                size_bits: 12, is_device: false,
            }));
        }
        // Full — next add fails
        assert!(!bi.add_untyped(UntypedRegion { cap: 999, paddr: 0, size_bits: 12, is_device: false }));
    }
}
