// Copyright (c) 2026 Edison Lepitel / AIEONYX
// SPDX-License-Identifier: Apache-2.0
// mem/buffer.rs -- BoundedBuffer: fixed-capacity, no heap reallocation.
// Used for seL4 message buffers, IPC payloads, and audit event queues.
// Capacity is set at creation and never exceeded — no panics on overflow.

#[derive(Debug, Clone, PartialEq)]
pub enum BufferError {
    /// Write would exceed buffer capacity.
    Overflow,
    /// Read requested more bytes than available.
    Underflow,
    /// Buffer capacity is zero.
    ZeroCapacity,
}

impl std::fmt::Display for BufferError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            BufferError::Overflow     => write!(f, "buffer overflow"),
            BufferError::Underflow    => write!(f, "buffer underflow"),
            BufferError::ZeroCapacity => write!(f, "zero capacity buffer"),
        }
    }
}

/// Fixed-capacity byte buffer. Never reallocates. Zeroizes on drop.
impl std::fmt::Debug for BoundedBuffer {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "BoundedBuffer(cap={}, avail={})", self.capacity, self.available())
    }
}

pub struct BoundedBuffer {
    data:     Vec<u8>,
    capacity: usize,
    write_pos: usize,
    read_pos:  usize,
}

impl BoundedBuffer {
    /// Create a new buffer with fixed capacity (bytes).
    pub fn new(capacity: usize) -> Result<Self, BufferError> {
        if capacity == 0 { return Err(BufferError::ZeroCapacity); }
        Ok(BoundedBuffer {
            data:      vec![0u8; capacity],
            capacity,
            write_pos: 0,
            read_pos:  0,
        })
    }

    /// Write bytes into the buffer. Returns Err(Overflow) if insufficient space.
    pub fn write(&mut self, bytes: &[u8]) -> Result<(), BufferError> {
        if self.write_pos + bytes.len() > self.capacity {
            return Err(BufferError::Overflow);
        }
        self.data[self.write_pos..self.write_pos + bytes.len()]
            .copy_from_slice(bytes);
        self.write_pos += bytes.len();
        Ok(())
    }

    /// Read `n` bytes from the buffer. Returns Err(Underflow) if insufficient data.
    pub fn read(&mut self, n: usize) -> Result<Vec<u8>, BufferError> {
        if self.read_pos + n > self.write_pos {
            return Err(BufferError::Underflow);
        }
        let out = self.data[self.read_pos..self.read_pos + n].to_vec();
        self.read_pos += n;
        Ok(out)
    }

    /// Peek at bytes without advancing read position.
    pub fn peek(&self, n: usize) -> Result<&[u8], BufferError> {
        if self.read_pos + n > self.write_pos {
            return Err(BufferError::Underflow);
        }
        Ok(&self.data[self.read_pos..self.read_pos + n])
    }

    /// Reset buffer to empty (zeroizes contents).
    pub fn reset(&mut self) {
        for b in self.data.iter_mut() {
            unsafe { std::ptr::write_volatile(b, 0u8); }
        }
        self.write_pos = 0;
        self.read_pos  = 0;
    }

    pub fn available(&self) -> usize { self.write_pos - self.read_pos }
    pub fn remaining(&self) -> usize { self.capacity - self.write_pos }
    pub fn capacity(&self)  -> usize { self.capacity }
    pub fn is_empty(&self)  -> bool  { self.available() == 0 }
    pub fn is_full(&self)   -> bool  { self.remaining() == 0 }
    pub fn as_written(&self) -> &[u8] { &self.data[..self.write_pos] }
}

impl Drop for BoundedBuffer {
    fn drop(&mut self) { self.reset(); }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_buffer_write_read() {
        let mut buf = BoundedBuffer::new(16).unwrap();
        buf.write(b"sovereign").unwrap();
        let out = buf.read(9).unwrap();
        assert_eq!(out, b"sovereign");
    }

    #[test]
    fn test_buffer_overflow() {
        let mut buf = BoundedBuffer::new(4).unwrap();
        assert_eq!(buf.write(b"hello"), Err(BufferError::Overflow));
    }

    #[test]
    fn test_buffer_underflow() {
        let mut buf = BoundedBuffer::new(8).unwrap();
        assert_eq!(buf.read(4), Err(BufferError::Underflow));
    }

    #[test]
    fn test_buffer_zero_capacity() {
        assert!(matches!(BoundedBuffer::new(0), Err(BufferError::ZeroCapacity)));
    }

    #[test]
    fn test_buffer_available_remaining() {
        let mut buf = BoundedBuffer::new(8).unwrap();
        buf.write(b"abcd").unwrap();
        assert_eq!(buf.available(), 4);
        assert_eq!(buf.remaining(), 4);
    }

    #[test]
    fn test_buffer_reset_zeroizes() {
        let mut buf = BoundedBuffer::new(8).unwrap();
        buf.write(b"secret!!").unwrap();
        buf.reset();
        assert!(buf.is_empty());
        assert_eq!(buf.as_written(), b"");
    }

    #[test]
    fn test_buffer_peek() {
        let mut buf = BoundedBuffer::new(8).unwrap();
        buf.write(b"hello").unwrap();
        assert_eq!(buf.peek(5).unwrap(), b"hello");
        // Peek doesn't advance read pos
        assert_eq!(buf.available(), 5);
    }

    #[test]
    fn test_buffer_is_full() {
        let mut buf = BoundedBuffer::new(4).unwrap();
        assert!(!buf.is_full());
        buf.write(b"abcd").unwrap();
        assert!(buf.is_full());
    }

    #[test]
    fn test_buffer_sequential_writes_reads() {
        let mut buf = BoundedBuffer::new(16).unwrap();
        buf.write(b"axon").unwrap();
        buf.write(b"seL4").unwrap();
        assert_eq!(buf.read(4).unwrap(), b"axon");
        assert_eq!(buf.read(4).unwrap(), b"seL4");
    }
}
