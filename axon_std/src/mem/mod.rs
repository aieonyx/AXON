// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
// mem/mod.rs -- Sovereign memory primitives.
// Zeroize: guaranteed zeroing of sensitive data (keys, secrets).
// BoundedBuffer: fixed-capacity buffer, no heap reallocation.
// Region: typed memory region with bounds checking.
// These map directly to seL4 memory frame management.

pub mod zeroize;
pub mod buffer;
pub mod region;

pub use zeroize::{zeroize, Zeroize, ZeroizeOnDrop};
pub use buffer::{BoundedBuffer, BufferError};
pub use region::{MemRegion, RegionError};
