// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
//
// axon_learn — ONYX Framework: AI/ML Primitives (Phase 33)
// Tape-based autodiff, neural net layers, loss functions, optimizers.
// no_std + alloc. DynTensor<f32> as the training type.

#![no_std]
#![deny(unsafe_op_in_unsafe_fn)]
#![allow(clippy::needless_range_loop)]

extern crate alloc;

pub mod tape;
pub mod layers;
pub mod loss;
pub mod optim;

#[cfg(feature = "eddb")]
pub mod eddb_bridge;

// Convenience re-exports
pub use tape::{Tape, Var, VarId};
pub use layers::{Linear, relu, softmax, gelu};
pub use loss::{mse, cross_entropy};
pub use optim::{Sgd, Adam};
