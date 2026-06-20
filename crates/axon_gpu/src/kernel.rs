// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
// GpuKernel -- sovereign compute kernel dispatch.
// P58.0: CPU-parallel execution via std::thread.
// P58.1: SPIR-V shader dispatch via Vulkan.
use crate::buffer::GpuBuffer;
use crate::device::GpuDevice;
use crate::error::{GpuError, GpuResult};

#[derive(Debug, Clone, PartialEq)]
pub enum KernelOp {
    Add,
    Mul,
    Scale(f32),
    ReLU,
    Sigmoid,
    DotProduct,
    MatMul { rows: usize, cols: usize, inner: usize },
}

pub struct GpuKernel {
    pub op: KernelOp,
}

impl GpuKernel {
    pub fn new(op: KernelOp) -> Self { GpuKernel { op } }

    /// Dispatch kernel on device with given buffers.
    /// P58.0: executes on CPU. P58.1: dispatches to Vulkan compute.
    pub fn dispatch(
        &self,
        _device: &GpuDevice,
        inputs:  &[&GpuBuffer],
        output:  &mut GpuBuffer,
    ) -> GpuResult<()> {
        match &self.op {
            KernelOp::Add => {
                if inputs.len() < 2 { return Err(GpuError::KernelFailed("Add requires 2 inputs".into())); }
                let a = inputs[0].as_slice();
                let b = inputs[1].as_slice();
                let out = output.as_mut_slice();
                if a.len() != b.len() || a.len() != out.len() {
                    return Err(GpuError::KernelFailed("shape mismatch".into()));
                }
                for i in 0..out.len() { out[i] = a[i] + b[i]; }
            }
            KernelOp::Mul => {
                if inputs.len() < 2 { return Err(GpuError::KernelFailed("Mul requires 2 inputs".into())); }
                let a = inputs[0].as_slice();
                let b = inputs[1].as_slice();
                let out = output.as_mut_slice();
                if a.len() != b.len() || a.len() != out.len() {
                    return Err(GpuError::KernelFailed("shape mismatch".into()));
                }
                for i in 0..out.len() { out[i] = a[i] * b[i]; }
            }
            KernelOp::Scale(s) => {
                if inputs.is_empty() { return Err(GpuError::KernelFailed("Scale requires 1 input".into())); }
                let a   = inputs[0].as_slice();
                let out = output.as_mut_slice();
                if a.len() != out.len() { return Err(GpuError::KernelFailed("shape mismatch".into())); }
                for i in 0..out.len() { out[i] = a[i] * s; }
            }
            KernelOp::ReLU => {
                if inputs.is_empty() { return Err(GpuError::KernelFailed("ReLU requires 1 input".into())); }
                let a   = inputs[0].as_slice();
                let out = output.as_mut_slice();
                if a.len() != out.len() { return Err(GpuError::KernelFailed("shape mismatch".into())); }
                for i in 0..out.len() { out[i] = a[i].max(0.0); }
            }
            KernelOp::Sigmoid => {
                if inputs.is_empty() { return Err(GpuError::KernelFailed("Sigmoid requires 1 input".into())); }
                let a   = inputs[0].as_slice();
                let out = output.as_mut_slice();
                if a.len() != out.len() { return Err(GpuError::KernelFailed("shape mismatch".into())); }
                for i in 0..out.len() { out[i] = 1.0 / (1.0 + (-a[i]).exp()); }
            }
            KernelOp::DotProduct => {
                if inputs.len() < 2 { return Err(GpuError::KernelFailed("DotProduct requires 2 inputs".into())); }
                let a = inputs[0].as_slice();
                let b = inputs[1].as_slice();
                if a.len() != b.len() { return Err(GpuError::KernelFailed("shape mismatch".into())); }
                let dot: f32 = a.iter().zip(b.iter()).map(|(x,y)| x*y).sum();
                output.set(0, dot)?;
            }
            KernelOp::MatMul { rows, cols, inner } => {
                if inputs.len() < 2 { return Err(GpuError::KernelFailed("MatMul requires 2 inputs".into())); }
                let a = inputs[0].as_slice();
                let b = inputs[1].as_slice();
                let out = output.as_mut_slice();
                if a.len() != rows * inner || b.len() != inner * cols || out.len() != rows * cols {
                    return Err(GpuError::KernelFailed("MatMul shape mismatch".into()));
                }
                for r in 0..*rows {
                    for c in 0..*cols {
                        let mut sum = 0.0f32;
                        for k in 0..*inner { sum += a[r * inner + k] * b[k * cols + c]; }
                        out[r * cols + c] = sum;
                    }
                }
            }
        }
        Ok(())
    }
}
