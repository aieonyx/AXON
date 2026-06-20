// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
// axon_gpu error types.
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum GpuError {
    NoDevice,
    OutOfMemory(usize),
    KernelFailed(String),
    InvalidBuffer,
    UnsupportedOp(String),
    BackendError(String),
}

impl fmt::Display for GpuError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            GpuError::NoDevice           => write!(f, "no GPU device available"),
            GpuError::OutOfMemory(n)     => write!(f, "GPU out of memory: {} bytes requested", n),
            GpuError::KernelFailed(s)    => write!(f, "kernel failed: {}", s),
            GpuError::InvalidBuffer      => write!(f, "invalid buffer"),
            GpuError::UnsupportedOp(s)   => write!(f, "unsupported operation: {}", s),
            GpuError::BackendError(s)    => write!(f, "backend error: {}", s),
        }
    }
}

pub type GpuResult<T> = Result<T, GpuError>;
