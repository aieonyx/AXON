// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
// axon_gpu -- sovereign GPU compute abstraction.
// P58.0: CPU fallback backend — full API, correct semantics.
// P58.1: Vulkan backend via ash — device probe, buffer mgmt, SPIR-V kernels.

pub mod buffer;
pub mod device;
pub mod error;
pub mod kernel;

#[cfg(feature = "vulkan")]
pub mod vulkan;

pub use buffer::{GpuBuffer, BufferKind};
pub use device::{GpuDevice, GpuBackend, GpuCapabilities};
pub use error::{GpuError, GpuResult};
pub use kernel::{GpuKernel, KernelOp};
