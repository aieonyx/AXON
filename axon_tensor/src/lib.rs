// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
//
// axon_tensor — ONYX Framework: Tensor Engine (Phase 32)
// Two-tier design: Tensor<T,D> (const) + DynTensor<T> (dynamic)
// no_std + alloc

#![no_std]
#![deny(unsafe_op_in_unsafe_fn)]
#![allow(clippy::needless_range_loop)]

extern crate alloc;

pub mod tensor;
pub mod dyn_tensor;
pub mod ops;
pub mod simd;

#[cfg(feature = "eddb")]
pub mod eddb_bridge;

// Convenience re-exports
pub use tensor::Tensor;
pub use dyn_tensor::DynTensor;
pub use ops::TensorOps;
