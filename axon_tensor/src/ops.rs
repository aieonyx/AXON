// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
//
// axon_tensor::ops — TensorOps trait
// Shared interface for Tensor<T,D> and DynTensor<T>

use num_traits::{Zero, NumCast};
use core::ops::{Add, Mul, AddAssign};

/// Scalar bound used across axon_tensor.
pub trait TensorScalar:
    Copy
    + Zero
    + Add<Output = Self>
    + Mul<Output = Self>
    + AddAssign
    + NumCast
    + PartialOrd
{
}

impl<T> TensorScalar for T where
    T: Copy
        + Zero
        + Add<Output = T>
        + Mul<Output = T>
        + AddAssign
        + NumCast
        + PartialOrd
{
}

/// Core tensor operations — implemented by both Tensor<T,D> and DynTensor<T>.
pub trait TensorOps<T: TensorScalar> {
    /// Total number of elements.
    fn numel(&self) -> usize;

    /// Raw flat data slice.
    fn data(&self) -> &[T];

    /// Mutable raw flat data slice.
    fn data_mut(&mut self) -> &mut [T];

    /// Number of dimensions (rank).
    fn rank(&self) -> usize;

    /// Shape as a slice of dimension sizes.
    fn shape(&self) -> &[usize];

    /// Strides as a slice (row-major).
    fn strides(&self) -> &[usize];

    /// Get element at flat index.
    fn get_flat(&self, idx: usize) -> T {
        self.data()[idx]
    }

    /// Set element at flat index.
    fn set_flat(&mut self, idx: usize, val: T) {
        self.data_mut()[idx] = val;
    }

    /// Fill all elements with a scalar value.
    fn fill(&mut self, val: T) {
        for x in self.data_mut() {
            *x = val;
        }
    }

    /// Element-wise sum of all elements.
    fn sum(&self) -> T {
        let mut acc = T::zero();
        for &x in self.data() {
            acc += x;
        }
        acc
    }
}

/// Compute row-major strides from a shape slice.
/// strides[i] = product of shape[i+1..rank]
pub fn row_major_strides(shape: &[usize]) -> alloc::vec::Vec<usize> {
    let rank = shape.len();
    let mut strides = alloc::vec![1usize; rank];
    for i in (0..rank.saturating_sub(1)).rev() {
        strides[i] = strides[i + 1] * shape[i + 1];
    }
    strides
}

/// Convert multi-dimensional index to flat index using strides.
pub fn multi_to_flat(idx: &[usize], strides: &[usize]) -> usize {
    idx.iter().zip(strides.iter()).map(|(&i, &s)| i * s).sum()
}
