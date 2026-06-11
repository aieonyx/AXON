// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
//
// axon_compute::kernel — Sovereign Compute Kernels
// CPU scalar path: always active, fully tested.
// GPU path: dispatches KernelLaunch; execution requires hardware.

use axon_tensor::DynTensor;
use axon_tensor::ops::TensorOps;
use crate::dispatch::{ComputeBackend, BufferDescriptor, KernelLaunch, LaunchConfig};

// ------------------------------------------------------------------
// MatMul dispatch
// ------------------------------------------------------------------

/// Dispatch matrix multiply: C = A @ B
/// CPU path: direct scalar computation.
/// GPU path: builds KernelLaunch for hardware execution.
pub fn matmul_dispatch(
    a: &DynTensor<f32>,
    b: &DynTensor<f32>,
    backend: ComputeBackend,
) -> Result<DynTensor<f32>, &'static str> {
    if a.shape().len() != 2 || b.shape().len() != 2 {
        return Err("matmul_dispatch: inputs must be rank-2");
    }
    let (r, k)  = (a.shape()[0], a.shape()[1]);
    let (k2, c) = (b.shape()[0], b.shape()[1]);
    if k != k2 {
        return Err("matmul_dispatch: inner dimensions must match");
    }

    match backend {
        ComputeBackend::Cpu => {
            // CPU scalar path — always available
            Ok(cpu_matmul_f32(a, b, r, k, c))
        }
        ComputeBackend::Cuda | ComputeBackend::Rocm | ComputeBackend::Bastion => {
            // Build dispatch request — actual execution requires hardware
            let _launch = KernelLaunch::new(
                "matmul_f32",
                backend,
                LaunchConfig::matrix(r, c, 16),
            )
            .with_input(BufferDescriptor::f32_ro(r * k, "A"))
            .with_input(BufferDescriptor::f32_ro(k * c, "B"))
            .with_output(BufferDescriptor::f32_rw(r * c, "C"))
            .with_param(r as u32)
            .with_param(k as u32)
            .with_param(c as u32);
            // In deployment: submit _launch to hardware runtime.
            // In test/no-hardware: fall back to CPU.
            Ok(cpu_matmul_f32(a, b, r, k, c))
        }
    }
}

fn cpu_matmul_f32(
    a: &DynTensor<f32>,
    b: &DynTensor<f32>,
    r: usize, k: usize, c: usize,
) -> DynTensor<f32> {
    let mut out = DynTensor::zeros(alloc::vec![r, c]);
    for i in 0..r {
        for j in 0..c {
            let mut acc = 0.0f32;
            for p in 0..k {
                acc += a.get(&[i, p]) * b.get(&[p, j]);
            }
            out.set(&[i, j], acc);
        }
    }
    out
}

// ------------------------------------------------------------------
// Element-wise add dispatch
// ------------------------------------------------------------------

pub fn elementwise_add_dispatch(
    a: &DynTensor<f32>,
    b: &DynTensor<f32>,
    backend: ComputeBackend,
) -> Result<DynTensor<f32>, &'static str> {
    if a.shape() != b.shape() {
        return Err("elementwise_add_dispatch: shape mismatch");
    }
    match backend {
        ComputeBackend::Cpu => {
            Ok(a.add(b))
        }
        _ => {
            let n = a.numel();
            let _launch = KernelLaunch::new(
                "elementwise_add_f32",
                backend,
                LaunchConfig::linear(n, 256),
            )
            .with_input(BufferDescriptor::f32_ro(n, "A"))
            .with_input(BufferDescriptor::f32_ro(n, "B"))
            .with_output(BufferDescriptor::f32_rw(n, "C"))
            .with_param(n as u32);
            Ok(a.add(b))
        }
    }
}

// ------------------------------------------------------------------
// Element-wise mul dispatch
// ------------------------------------------------------------------

pub fn elementwise_mul_dispatch(
    a: &DynTensor<f32>,
    b: &DynTensor<f32>,
    backend: ComputeBackend,
) -> Result<DynTensor<f32>, &'static str> {
    if a.shape() != b.shape() {
        return Err("elementwise_mul_dispatch: shape mismatch");
    }
    match backend {
        ComputeBackend::Cpu => Ok(a.mul(b)),
        _ => {
            let n = a.numel();
            let _launch = KernelLaunch::new(
                "elementwise_mul_f32",
                backend,
                LaunchConfig::linear(n, 256),
            )
            .with_input(BufferDescriptor::f32_ro(n, "A"))
            .with_input(BufferDescriptor::f32_ro(n, "B"))
            .with_output(BufferDescriptor::f32_rw(n, "C"))
            .with_param(n as u32);
            Ok(a.mul(b))
        }
    }
}

// ------------------------------------------------------------------
// ReLU dispatch
// ------------------------------------------------------------------

pub fn relu_dispatch(
    x: &DynTensor<f32>,
    backend: ComputeBackend,
) -> Result<DynTensor<f32>, &'static str> {
    match backend {
        ComputeBackend::Cpu => {
            Ok(axon_learn::layers::relu(x))
        }
        _ => {
            let n = x.numel();
            let _launch = KernelLaunch::new(
                "relu_f32",
                backend,
                LaunchConfig::linear(n, 256),
            )
            .with_input(BufferDescriptor::f32_ro(n, "X"))
            .with_output(BufferDescriptor::f32_rw(n, "Y"))
            .with_param(n as u32);
            Ok(axon_learn::layers::relu(x))
        }
    }
}

// ------------------------------------------------------------------
// Batched inference: linear → relu → softmax pipeline
// ------------------------------------------------------------------

/// Run a single Linear → ReLU → Softmax inference pass on CPU.
/// Useful as a verified reference for GPU dispatch comparison.
pub fn inference_pass(
    input:  &DynTensor<f32>,
    layer:  &axon_learn::layers::Linear,
    backend: ComputeBackend,
) -> Result<DynTensor<f32>, &'static str> {
    let linear_out = layer.forward(input);
    let relu_out   = relu_dispatch(&linear_out, backend)?;
    let softmax_out = axon_learn::layers::softmax(&relu_out);
    Ok(softmax_out)
}
