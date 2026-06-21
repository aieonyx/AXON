// Copyright (c) 2026 Edison Lepitel / AIEONYX
// SPDX-License-Identifier: Apache-2.0
// mem/region.rs -- MemRegion: typed memory region with bounds checking.
// Abstracts seL4 memory frames — a region has a base, size, and permissions.
// Used by axon_sel4 IPC binding to describe shared memory capabilities.

#[derive(Debug, Clone, PartialEq)]
pub enum RegionError {
    OutOfBounds { offset: usize, len: usize, region_size: usize },
    NotWritable,
    ZeroSize,
}

impl std::fmt::Display for RegionError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            RegionError::OutOfBounds { offset, len, region_size } =>
                write!(f, "out of bounds: offset={} len={} region={}", offset, len, region_size),
            RegionError::NotWritable => write!(f, "region is read-only"),
            RegionError::ZeroSize    => write!(f, "zero-size region"),
        }
    }
}

/// Memory region permissions.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RegionPerms {
    ReadOnly,
    ReadWrite,
}

/// A bounded memory region — analogous to a seL4 memory frame capability.
impl std::fmt::Debug for MemRegion {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "MemRegion(size={}, perms={:?})", self.data.len(), self.perms)
    }
}

pub struct MemRegion {
    data:  Vec<u8>,
    perms: RegionPerms,
}

impl MemRegion {
    /// Allocate a new zeroed region of `size` bytes.
    pub fn new(size: usize, perms: RegionPerms) -> Result<Self, RegionError> {
        if size == 0 { return Err(RegionError::ZeroSize); }
        Ok(MemRegion { data: vec![0u8; size], perms })
    }

    /// Create a read-only view over existing bytes.
    pub fn from_bytes(bytes: &[u8]) -> Self {
        MemRegion { data: bytes.to_vec(), perms: RegionPerms::ReadOnly }
    }

    pub fn size(&self)  -> usize { self.data.len() }
    pub fn perms(&self) -> RegionPerms { self.perms }
    pub fn is_writable(&self) -> bool { self.perms == RegionPerms::ReadWrite }

    /// Read `len` bytes at `offset`.
    pub fn read(&self, offset: usize, len: usize) -> Result<&[u8], RegionError> {
        if offset + len > self.data.len() {
            return Err(RegionError::OutOfBounds {
                offset, len, region_size: self.data.len()
            });
        }
        Ok(&self.data[offset..offset+len])
    }

    /// Write bytes at `offset`. Requires ReadWrite permission.
    pub fn write(&mut self, offset: usize, bytes: &[u8]) -> Result<(), RegionError> {
        if self.perms == RegionPerms::ReadOnly { return Err(RegionError::NotWritable); }
        if offset + bytes.len() > self.data.len() {
            return Err(RegionError::OutOfBounds {
                offset, len: bytes.len(), region_size: self.data.len()
            });
        }
        self.data[offset..offset+bytes.len()].copy_from_slice(bytes);
        Ok(())
    }

    /// Zero the entire region.
    pub fn zeroize(&mut self) {
        for b in self.data.iter_mut() {
            unsafe { std::ptr::write_volatile(b, 0u8); }
        }
    }
}

impl Drop for MemRegion {
    fn drop(&mut self) { self.zeroize(); }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_region_read_write() {
        let mut r = MemRegion::new(16, RegionPerms::ReadWrite).unwrap();
        r.write(0, b"axon").unwrap();
        assert_eq!(r.read(0, 4).unwrap(), b"axon");
    }

    #[test]
    fn test_region_readonly_rejects_write() {
        let mut r = MemRegion::from_bytes(b"readonly data");
        assert_eq!(r.write(0, b"x"), Err(RegionError::NotWritable));
    }

    #[test]
    fn test_region_out_of_bounds_read() {
        let r = MemRegion::new(4, RegionPerms::ReadOnly).unwrap();
        assert!(matches!(r.read(2, 4), Err(RegionError::OutOfBounds{..})));
    }

    #[test]
    fn test_region_out_of_bounds_write() {
        let mut r = MemRegion::new(4, RegionPerms::ReadWrite).unwrap();
        assert!(matches!(r.write(2, b"hello"), Err(RegionError::OutOfBounds{..})));
    }

    #[test]
    fn test_region_zero_size() {
        assert!(matches!(MemRegion::new(0, RegionPerms::ReadWrite), Err(RegionError::ZeroSize)));
    }

    #[test]
    fn test_region_zeroize() {
        let mut r = MemRegion::new(8, RegionPerms::ReadWrite).unwrap();
        r.write(0, b"secret!!").unwrap();
        r.zeroize();
        assert_eq!(r.read(0, 8).unwrap(), &[0u8; 8]);
    }

    #[test]
    fn test_region_size_perms() {
        let r = MemRegion::new(64, RegionPerms::ReadWrite).unwrap();
        assert_eq!(r.size(), 64);
        assert!(r.is_writable());
    }

    #[test]
    fn test_region_from_bytes_readonly() {
        let r = MemRegion::from_bytes(b"hello");
        assert_eq!(r.perms(), RegionPerms::ReadOnly);
        assert!(!r.is_writable());
        assert_eq!(r.read(0,5).unwrap(), b"hello");
    }
}
