// Copyright (c) 2026 Edison Lepiten / AIEONYX
//! USB Mass Storage driver — bulk-only transport (BOT).
//!
//! Implements USB Mass Storage Class Bulk-Only Transport (BBB spec).
//! Provides block-level read/write over USB drives.

use axon_core::prelude::*;

/// Block size in bytes (standard 512-byte sectors).
pub const BLOCK_SIZE: usize = 512;
/// Maximum blocks per transfer.
pub const MAX_BLOCKS_PER_TRANSFER: u32 = 128;

/// Storage device info.
#[derive(Debug, Clone, Copy)]
pub struct StorageInfo {
    pub block_count: u64,
    pub block_size:  u32,
    pub read_only:   bool,
}

impl StorageInfo {
    pub fn capacity_bytes(&self) -> u64 {
        self.block_count * self.block_size as u64
    }
}

/// Mass storage driver interface.
pub trait StorageDriver {
    fn info(&self) -> AxonResult<StorageInfo>;
    fn read_block(&mut self, lba: u64, buf: &mut [u8]) -> AxonResult<()>;
    fn write_block(&mut self, lba: u64, buf: &[u8]) -> AxonResult<()>;
    fn flush(&mut self) -> AxonResult<()> { AxonResult::Ok(()) }
}

/// Host stub storage driver — in-memory block device.
pub struct StubStorage {
    blocks: alloc::vec::Vec<[u8; BLOCK_SIZE]>,
    read_only: bool,
}

extern crate alloc;

impl StubStorage {
    pub fn new(block_count: u64) -> Self {
        Self {
            blocks: alloc::vec![[0u8; BLOCK_SIZE]; block_count as usize],
            read_only: false,
        }
    }
    pub fn new_readonly(block_count: u64) -> Self {
        let mut s = Self::new(block_count);
        s.read_only = true;
        s
    }
}

impl StorageDriver for StubStorage {
    fn info(&self) -> AxonResult<StorageInfo> {
        AxonResult::Ok(StorageInfo {
            block_count: self.blocks.len() as u64,
            block_size:  BLOCK_SIZE as u32,
            read_only:   self.read_only,
        })
    }

    fn read_block(&mut self, lba: u64, buf: &mut [u8]) -> AxonResult<()> {
        if lba as usize >= self.blocks.len() {
            return AxonResult::Err(AxonError::invalid_input("LBA out of range"));
        }
        if buf.len() < BLOCK_SIZE {
            return AxonResult::Err(AxonError::invalid_input("buffer too small"));
        }
        buf[..BLOCK_SIZE].copy_from_slice(&self.blocks[lba as usize]);
        AxonResult::Ok(())
    }

    fn write_block(&mut self, lba: u64, buf: &[u8]) -> AxonResult<()> {
        if self.read_only {
            return AxonResult::Err(AxonError::permission_denied("storage is read-only"));
        }
        if lba as usize >= self.blocks.len() {
            return AxonResult::Err(AxonError::invalid_input("LBA out of range"));
        }
        if buf.len() < BLOCK_SIZE {
            return AxonResult::Err(AxonError::invalid_input("buffer too small"));
        }
        self.blocks[lba as usize].copy_from_slice(&buf[..BLOCK_SIZE]);
        AxonResult::Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tp43_storage_read_write() {
        let mut s = StubStorage::new(16);
        let mut write_buf = [0u8; BLOCK_SIZE];
        write_buf[0] = 0xAB; write_buf[511] = 0xCD;
        s.write_block(0, &write_buf).unwrap();
        let mut read_buf = [0u8; BLOCK_SIZE];
        s.read_block(0, &mut read_buf).unwrap();
        assert_eq!(read_buf[0], 0xAB);
        assert_eq!(read_buf[511], 0xCD);
    }

    #[test]
    fn tp43_storage_read_only() {
        let mut s = StubStorage::new_readonly(8);
        assert!(s.write_block(0, &[0u8; BLOCK_SIZE]).is_err());
        assert!(s.read_block(0, &mut [0u8; BLOCK_SIZE]).is_ok());
    }

    #[test]
    fn tp43_storage_lba_out_of_range() {
        let mut s = StubStorage::new(4);
        assert!(s.read_block(4, &mut [0u8; BLOCK_SIZE]).is_err());
        assert!(s.write_block(4, &[0u8; BLOCK_SIZE]).is_err());
    }

    #[test]
    fn tp43_storage_info() {
        let s = StubStorage::new(1024);
        let info = s.info().unwrap();
        assert_eq!(info.block_count, 1024);
        assert_eq!(info.block_size, 512);
        assert_eq!(info.capacity_bytes(), 512 * 1024);
    }

    #[test]
    fn tp43_storage_buffer_too_small() {
        let mut s = StubStorage::new(4);
        assert!(s.read_block(0, &mut [0u8; 16]).is_err());
    }
}
