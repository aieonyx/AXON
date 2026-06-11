// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
//
// axon_tensor::dyn_tensor — DynTensor<T>
// Dynamic rank, Vec<usize> shape, heap-backed.
// ML workflow path — rank and shape known at runtime.

use alloc::vec::Vec;
use crate::ops::{TensorScalar, TensorOps, row_major_strides, multi_to_flat};

/// Dynamic-rank tensor. Shape and rank determined at runtime.
#[derive(Clone, Debug, PartialEq)]
pub struct DynTensor<T> {
    shape:   Vec<usize>,
    strides: Vec<usize>,
    data:    Vec<T>,
}

impl<T: TensorScalar> DynTensor<T> {
    /// Create a DynTensor filled with zeros.
    pub fn zeros(shape: Vec<usize>) -> Self {
        let numel = shape.iter().product();
        let strides = row_major_strides(&shape);
        Self {
            shape,
            strides,
            data: alloc::vec![T::zero(); numel],
        }
    }

    /// Create a DynTensor from a flat Vec. Panics if len != product(shape).
    pub fn from_vec(shape: Vec<usize>, data: Vec<T>) -> Self {
        let numel: usize = shape.iter().product();
        assert_eq!(
            data.len(), numel,
            "DynTensor::from_vec: data length {} != shape product {}",
            data.len(), numel
        );
        let strides = row_major_strides(&shape);
        Self { shape, strides, data }
    }

    /// Get element at multi-dimensional index slice.
    /// Panics if index rank != tensor rank or out of bounds.
    pub fn get(&self, idx: &[usize]) -> T {
        assert_eq!(idx.len(), self.shape.len(), "index rank mismatch");
        for (d, (&i, &s)) in idx.iter().zip(self.shape.iter()).enumerate() {
            assert!(i < s, "index out of bounds at dim {d}");
        }
        let flat = multi_to_flat(idx, &self.strides);
        self.data[flat]
    }

    /// Set element at multi-dimensional index slice.
    pub fn set(&mut self, idx: &[usize], val: T) {
        assert_eq!(idx.len(), self.shape.len(), "index rank mismatch");
        for (d, (&i, &s)) in idx.iter().zip(self.shape.iter()).enumerate() {
            assert!(i < s, "index out of bounds at dim {d}");
        }
        let flat = multi_to_flat(idx, &self.strides);
        self.data[flat] = val;
    }

    /// Reshape to a new shape. Panics if total elements differ.
    pub fn reshape(&self, new_shape: Vec<usize>) -> Self {
        let new_numel: usize = new_shape.iter().product();
        assert_eq!(
            self.numel(), new_numel,
            "reshape: element count mismatch {} vs {}",
            self.numel(), new_numel
        );
        Self::from_vec(new_shape, self.data.clone())
    }

    /// Element-wise add. Panics if shapes differ.
    pub fn add(&self, other: &Self) -> Self {
        assert_eq!(self.shape, other.shape, "shape mismatch in add");
        let data: Vec<T> = self.data.iter()
            .zip(other.data.iter())
            .map(|(&a, &b)| a + b)
            .collect();
        Self::from_vec(self.shape.clone(), data)
    }

    /// Element-wise multiply. Panics if shapes differ.
    pub fn mul(&self, other: &Self) -> Self {
        assert_eq!(self.shape, other.shape, "shape mismatch in mul");
        let data: Vec<T> = self.data.iter()
            .zip(other.data.iter())
            .map(|(&a, &b)| a * b)
            .collect();
        Self::from_vec(self.shape.clone(), data)
    }

    /// Scalar multiply.
    pub fn scale(&self, scalar: T) -> Self {
        let data: Vec<T> = self.data.iter().map(|&x| x * scalar).collect();
        Self::from_vec(self.shape.clone(), data)
    }

    /// Matrix multiply for rank-2 tensors.
    /// Returns None if rank != 2 or inner dimensions don't match.
    pub fn matmul(&self, other: &Self) -> Option<Self>
    where
        T: core::ops::AddAssign,
    {
        if self.shape.len() != 2 || other.shape.len() != 2 { return None; }
        let (r, k)  = (self.shape[0],  self.shape[1]);
        let (k2, c) = (other.shape[0], other.shape[1]);
        if k != k2 { return None; }
        let mut out = Self::zeros(alloc::vec![r, c]);
        for i in 0..r {
            for j in 0..c {
                let mut acc = T::zero();
                for p in 0..k {
                    acc += self.get(&[i, p]) * other.get(&[p, j]);
                }
                out.set(&[i, j], acc);
            }
        }
        Some(out)
    }

    /// Transpose rank-2 tensor. Returns None if rank != 2.
    pub fn transpose(&self) -> Option<Self> {
        if self.shape.len() != 2 { return None; }
        let (r, c) = (self.shape[0], self.shape[1]);
        let mut out = Self::zeros(alloc::vec![c, r]);
        for i in 0..r {
            for j in 0..c {
                let v = self.get(&[i, j]);
                out.set(&[j, i], v);
            }
        }
        Some(out)
    }

    /// Slice along axis 0: returns rows [start, end).
    /// Returns None if axis != 0 or bounds invalid.
    pub fn slice_axis0(&self, start: usize, end: usize) -> Option<Self> {
        if self.shape.is_empty() || end > self.shape[0] || start >= end {
            return None;
        }
        let row_size: usize = self.shape[1..].iter().product::<usize>().max(1);
        let data = self.data[start * row_size..end * row_size].to_vec();
        let mut new_shape = self.shape.clone();
        new_shape[0] = end - start;
        Some(Self::from_vec(new_shape, data))
    }

    /// Serialize to flat Vec<u8> for EdisonDB storage (f32 path).
    /// Each f32 element serialized as 4 little-endian bytes.
    #[cfg(feature = "eddb")]
    pub fn to_bytes_f32(&self) -> Vec<u8>
    where
        T: Into<f32> + Copy,
    {
        let mut out = Vec::with_capacity(self.data.len() * 4);
        for &x in &self.data {
            let f: f32 = x.into();
            out.extend_from_slice(&f.to_le_bytes());
        }
        out
    }
}

impl<T: TensorScalar> TensorOps<T> for DynTensor<T> {
    fn numel(&self)           -> usize    { self.data.len() }
    fn data(&self)            -> &[T]     { &self.data }
    fn data_mut(&mut self)    -> &mut [T] { &mut self.data }
    fn rank(&self)            -> usize    { self.shape.len() }
    fn shape(&self)           -> &[usize] { &self.shape }
    fn strides(&self)         -> &[usize] { &self.strides }
}
