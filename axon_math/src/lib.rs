// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
//
// axon_math — ONYX Framework: Core Math Stdlib
// Phase 31 | no_std compatible | BASTION OS / seL4 safe

#![no_std]
#![deny(unsafe_op_in_unsafe_fn)]
#![allow(clippy::needless_range_loop)]

// ------------------------------------------------------------------
// alloc is available when the "std" feature pulls it in transitively,
// but P31 (stack-only) never requires heap allocation.
// ------------------------------------------------------------------

pub mod linalg;
pub mod stats;
pub mod numerical;

#[cfg(feature = "ffi")]
pub mod ffi;

// Convenience re-exports
pub use linalg::{Matrix, dot, transpose};
pub use stats::{mean, variance, std_dev, normalize};
pub use numerical::{fft, ifft, integrate_simpson};
