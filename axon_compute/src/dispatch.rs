// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
//
// axon_compute::dispatch — GPU Kernel Dispatch Layer
// Backend abstraction: CPU scalar (always), CUDA/ROCm (feature-gated).
// KernelLaunch, BufferDescriptor, LaunchConfig — fully testable dispatch structs.

use alloc::vec::Vec;
use alloc::string::String;

// ------------------------------------------------------------------
// Compute backend selector
// ------------------------------------------------------------------

/// Available compute backends.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ComputeBackend {
    /// CPU scalar fallback — always available, no hardware required.
    Cpu,
    /// CUDA GPU backend — requires `cuda` feature + CUDA runtime.
    Cuda,
    /// ROCm GPU backend — requires `rocm` feature + ROCm runtime.
    Rocm,
    /// BASTION seL4 node backend — requires `bastion` feature.
    Bastion,
}

impl ComputeBackend {
    /// Returns true if this backend is available in the current build.
    pub fn is_available(self) -> bool {
        match self {
            ComputeBackend::Cpu     => true,
            ComputeBackend::Cuda    => cfg!(feature = "cuda"),
            ComputeBackend::Rocm    => cfg!(feature = "rocm"),
            ComputeBackend::Bastion => cfg!(feature = "bastion"),
        }
    }

    /// Select the best available backend in priority order:
    /// CUDA > ROCm > Bastion > CPU
    pub fn best_available() -> Self {
        if cfg!(feature = "cuda")    { return ComputeBackend::Cuda; }
        if cfg!(feature = "rocm")    { return ComputeBackend::Rocm; }
        if cfg!(feature = "bastion") { return ComputeBackend::Bastion; }
        ComputeBackend::Cpu
    }
}

// ------------------------------------------------------------------
// Buffer descriptor — describes a memory buffer for kernel I/O
// ------------------------------------------------------------------

/// Describes a compute buffer: its size, element type, and ownership.
#[derive(Clone, Debug, PartialEq)]
pub struct BufferDescriptor {
    /// Number of elements.
    pub numel:      usize,
    /// Element size in bytes (4 = f32, 8 = f64).
    pub elem_bytes: usize,
    /// Human-readable label for debugging.
    pub label:      String,
    /// True if this buffer is read-only (input).
    pub read_only:  bool,
}

impl BufferDescriptor {
    /// Create a mutable f32 buffer descriptor.
    pub fn f32_rw(numel: usize, label: &str) -> Self {
        Self {
            numel,
            elem_bytes: 4,
            label: alloc::string::ToString::to_string(label),
            read_only: false,
        }
    }

    /// Create a read-only f32 buffer descriptor.
    pub fn f32_ro(numel: usize, label: &str) -> Self {
        Self {
            numel,
            elem_bytes: 4,
            label: alloc::string::ToString::to_string(label),
            read_only: true,
        }
    }

    /// Total size in bytes.
    pub fn byte_size(&self) -> usize {
        self.numel * self.elem_bytes
    }
}

// ------------------------------------------------------------------
// Launch configuration — grid/block dimensions for GPU kernels
// ------------------------------------------------------------------

/// GPU kernel launch configuration.
/// On CPU fallback, these are ignored; the kernel runs sequentially.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LaunchConfig {
    /// Number of thread blocks (grid dimension).
    pub grid_x:  usize,
    pub grid_y:  usize,
    pub grid_z:  usize,
    /// Threads per block (block dimension).
    pub block_x: usize,
    pub block_y: usize,
    pub block_z: usize,
}

impl LaunchConfig {
    /// Create a 1D launch config covering `n` elements with `block_size` threads.
    pub fn linear(n: usize, block_size: usize) -> Self {
        let blocks = n.div_ceil(block_size);
        Self {
            grid_x: blocks, grid_y: 1, grid_z: 1,
            block_x: block_size, block_y: 1, block_z: 1,
        }
    }

    /// Create a 2D launch config for matrix ops [rows x cols].
    pub fn matrix(rows: usize, cols: usize, block: usize) -> Self {
        Self {
            grid_x:  cols.div_ceil(block),
            grid_y:  rows.div_ceil(block),
            grid_z:  1,
            block_x: block,
            block_y: block,
            block_z: 1,
        }
    }

    /// Total number of threads this config launches.
    pub fn total_threads(&self) -> usize {
        self.grid_x * self.grid_y * self.grid_z
            * self.block_x * self.block_y * self.block_z
    }
}

// ------------------------------------------------------------------
// KernelLaunch — a fully described kernel dispatch request
// ------------------------------------------------------------------

/// A complete kernel dispatch request.
/// Created by kernel wrappers, executed by the dispatch engine.
#[derive(Clone, Debug)]
pub struct KernelLaunch {
    /// Kernel identifier string (e.g. "matmul_f32", "relu_f32").
    pub kernel_id: String,
    /// Target backend.
    pub backend:   ComputeBackend,
    /// Launch configuration.
    pub config:    LaunchConfig,
    /// Input buffer descriptors.
    pub inputs:    Vec<BufferDescriptor>,
    /// Output buffer descriptors.
    pub outputs:   Vec<BufferDescriptor>,
    /// Scalar parameters (e.g. matrix dimensions, alpha/beta).
    pub params:    Vec<u32>,
}

impl KernelLaunch {
    /// Create a new kernel launch request.
    pub fn new(
        kernel_id: &str,
        backend: ComputeBackend,
        config: LaunchConfig,
    ) -> Self {
        Self {
            kernel_id: alloc::string::ToString::to_string(kernel_id),
            backend,
            config,
            inputs:  Vec::new(),
            outputs: Vec::new(),
            params:  Vec::new(),
        }
    }

    /// Add an input buffer.
    pub fn with_input(mut self, buf: BufferDescriptor) -> Self {
        self.inputs.push(buf);
        self
    }

    /// Add an output buffer.
    pub fn with_output(mut self, buf: BufferDescriptor) -> Self {
        self.outputs.push(buf);
        self
    }

    /// Add a scalar parameter.
    pub fn with_param(mut self, p: u32) -> Self {
        self.params.push(p);
        self
    }

    /// Validate the launch request — returns Err with reason if invalid.
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.kernel_id.is_empty() {
            return Err("kernel_id must not be empty");
        }
        if !self.backend.is_available() {
            return Err("requested backend is not available in this build");
        }
        if self.config.block_x == 0 || self.config.block_y == 0
            || self.config.block_z == 0
        {
            return Err("block dimensions must be non-zero");
        }
        if self.config.grid_x == 0 || self.config.grid_y == 0
            || self.config.grid_z == 0
        {
            return Err("grid dimensions must be non-zero");
        }
        Ok(())
    }
}
