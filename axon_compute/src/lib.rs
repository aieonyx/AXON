// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
//
// axon_compute — ONYX Framework: GPU Dispatch + Distributed Compute (Phase 34)
// Sovereign compute layer: GPU kernels, AWP mesh dispatch, BASTION orchestration,
// EdisonDB model storage.
// no_std + alloc.

#![no_std]
#![deny(unsafe_op_in_unsafe_fn)]
#![allow(clippy::needless_range_loop)]

extern crate alloc;

pub mod dispatch;
pub mod kernel;
pub mod mesh;
pub mod checkpoint;

// Convenience re-exports
pub use dispatch::{ComputeBackend, KernelLaunch, BufferDescriptor, LaunchConfig};
pub use kernel::{
    matmul_dispatch, elementwise_add_dispatch,
    elementwise_mul_dispatch, relu_dispatch,
};
pub use mesh::{MeshNode, NodeId, TaskDescriptor, MeshDispatcher};
pub use checkpoint::{ModelCheckpoint, save_checkpoint, load_checkpoint};
