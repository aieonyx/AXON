// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
// GpuBuffer -- sovereign GPU memory buffer.
// P58.0: CPU-backed Vec<f32>. P58.1: Vulkan device memory.
use crate::error::{GpuError, GpuResult};

#[derive(Debug, Clone, PartialEq)]
pub enum BufferKind {
    Input,
    Output,
    Intermediate,
}

#[derive(Debug)]
pub struct GpuBuffer {
    pub kind:    BufferKind,
    pub len:     usize,
    data:        Vec<f32>,
}

impl GpuBuffer {
    /// Allocate a zeroed GPU buffer of `len` f32 elements.
    pub fn zeros(kind: BufferKind, len: usize) -> GpuResult<Self> {
        if len == 0 { return Err(GpuError::InvalidBuffer); }
        Ok(GpuBuffer { kind, len, data: vec![0.0f32; len] })
    }

    /// Allocate and fill from a CPU slice.
    pub fn from_slice(kind: BufferKind, data: &[f32]) -> GpuResult<Self> {
        if data.is_empty() { return Err(GpuError::InvalidBuffer); }
        Ok(GpuBuffer { kind, len: data.len(), data: data.to_vec() })
    }

    /// Read buffer contents back to CPU.
    pub fn to_vec(&self) -> Vec<f32> { self.data.clone() }

    /// Read a single element.
    pub fn get(&self, idx: usize) -> GpuResult<f32> {
        self.data.get(idx).copied().ok_or(GpuError::InvalidBuffer)
    }

    /// Write a single element.
    pub fn set(&mut self, idx: usize, val: f32) -> GpuResult<()> {
        if idx >= self.len { return Err(GpuError::InvalidBuffer); }
        self.data[idx] = val;
        Ok(())
    }

    /// Fill entire buffer with a scalar value.
    pub fn fill(&mut self, val: f32) {
        self.data.iter_mut().for_each(|x| *x = val);
    }

    pub fn as_slice(&self) -> &[f32] { &self.data }
    pub fn as_mut_slice(&mut self) -> &mut [f32] { &mut self.data }
    pub fn size_bytes(&self) -> usize { self.len * 4 }
}
