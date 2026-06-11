// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
//
// axon_tensor::tensor — Tensor<T, const D: usize>
// Fixed rank, const-generic shape, heap-backed data.
// BASTION OS / seL4 / formal verification path.

use alloc::vec::Vec;
use crate::ops::{TensorScalar, TensorOps, row_major_strides, multi_to_flat};

/// Fixed-rank tensor. Rank D is known at compile time.
/// Shape and data are heap-allocated; shape array is const-generic.
#[derive(Clone, Debug, PartialEq)]
pub struct Tensor<T, const D: usize> {
    shape:   [usize; D],
    strides: [usize; D],
    data:    Vec<T>,
}

impl<T: TensorScalar, const D: usize> Tensor<T, D> {
    /// Create a tensor filled with zeros.
    pub fn zeros(shape: [usize; D]) -> Self {
        let numel = shape.iter().product();
        let strides_vec = row_major_strides(&shape);
        let mut strides = [0usize; D];
        strides.copy_from_slice(&strides_vec);
        Self {
            shape,
            strides,
            data: alloc::vec![T::zero(); numel],
        }
    }

    /// Create a tensor from a flat data Vec. Panics if len != product(shape).
    pub fn from_vec(shape: [usize; D], data: Vec<T>) -> Self {
        let numel: usize = shape.iter().product();
        assert_eq!(
            data.len(), numel,
            "Tensor::from_vec: data length {} != shape product {}",
            data.len(), numel
        );
        let strides_vec = row_major_strides(&shape);
        let mut strides = [0usize; D];
        strides.copy_from_slice(&strides_vec);
        Self { shape, strides, data }
    }

    /// Get element at multi-dimensional index. Panics if out of bounds.
    pub fn get(&self, idx: [usize; D]) -> T {
        for d in 0..D {
            assert!(idx[d] < self.shape[d], "index out of bounds at dim {d}");
        }
        let flat = multi_to_flat(&idx, &self.strides);
        self.data[flat]
    }

    /// Set element at multi-dimensional index. Panics if out of bounds.
    pub fn set(&mut self, idx: [usize; D], val: T) {
        for d in 0..D {
            assert!(idx[d] < self.shape[d], "index out of bounds at dim {d}");
        }
        let flat = multi_to_flat(&idx, &self.strides);
        self.data[flat] = val;
    }

    /// Return the shape array.
    pub fn shape_array(&self) -> [usize; D] {
        self.shape
    }

    /// Element-wise add two tensors of the same shape.
    /// Panics if shapes differ.
    pub fn add(&self, other: &Self) -> Self {
        assert_eq!(self.shape, other.shape, "shape mismatch in add");
        let data: Vec<T> = self.data.iter()
            .zip(other.data.iter())
            .map(|(&a, &b)| a + b)
            .collect();
        Self::from_vec(self.shape, data)
    }

    /// Element-wise multiply two tensors of the same shape.
    /// Panics if shapes differ.
    pub fn mul(&self, other: &Self) -> Self {
        assert_eq!(self.shape, other.shape, "shape mismatch in mul");
        let data: Vec<T> = self.data.iter()
            .zip(other.data.iter())
            .map(|(&a, &b)| a * b)
            .collect();
        Self::from_vec(self.shape, data)
    }

    /// Scalar multiply.
    pub fn scale(&self, scalar: T) -> Self {
        let data: Vec<T> = self.data.iter().map(|&x| x * scalar).collect();
        Self::from_vec(self.shape, data)
    }

    /// Matrix multiply: only valid for D=2 tensors.
    /// Returns None if rank != 2 or inner dimensions don't match.
    pub fn matmul_2d(&self, other: &Self) -> Option<Self>
    where
        T: core::ops::AddAssign,
    {
        if D != 2 { return None; }
        let (r, k) = (self.shape[0], self.shape[1]);
        let (k2, c) = (other.shape[0], other.shape[1]);
        if k != k2 { return None; }
        let mut out = Self::zeros({
            let mut s = [0usize; D];
            s[0] = r; s[1] = c; s
        });
        for i in 0..r {
            for j in 0..c {
                let mut acc = T::zero();
                for p in 0..k {
                    acc += self.get({let mut ix=[0;D]; ix[0]=i; ix[1]=p; ix})
                         * other.get({let mut ix=[0;D]; ix[0]=p; ix[1]=j; ix});
                }
                out.set({let mut ix=[0;D]; ix[0]=i; ix[1]=j; ix}, acc);
            }
        }
        Some(out)
    }

    /// Transpose a rank-2 tensor (swap axes 0 and 1).
    /// Returns None if D != 2.
    pub fn transpose_2d(&self) -> Option<Self> {
        if D != 2 { return None; }
        let (r, c) = (self.shape[0], self.shape[1]);
        let mut out = Self::zeros({let mut s=[0usize;D]; s[0]=c; s[1]=r; s});
        for i in 0..r {
            for j in 0..c {
                let v = self.get({let mut ix=[0;D]; ix[0]=i; ix[1]=j; ix});
                out.set({let mut ix=[0;D]; ix[0]=j; ix[1]=i; ix}, v);
            }
        }
        Some(out)
    }
}

impl<T: TensorScalar, const D: usize> TensorOps<T> for Tensor<T, D> {
    fn numel(&self)           -> usize  { self.data.len() }
    fn data(&self)            -> &[T]   { &self.data }
    fn data_mut(&mut self)    -> &mut [T] { &mut self.data }
    fn rank(&self)            -> usize  { D }
    fn shape(&self)           -> &[usize] { &self.shape }
    fn strides(&self)         -> &[usize] { &self.strides }
}
