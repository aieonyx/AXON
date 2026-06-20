// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
// Tensor -- n-dimensional array, sovereign implementation.
// Clean-room: studied NumPy internals and deep learning math only.
// P63.0: f32 tensors, row-major layout, bounds-checked access.
// P63.1: f16 support, GPU-backed tensors via axon_gpu.
use crate::error::{AiError, AiResult};

pub const MAX_TENSOR_ELEMENTS: usize = 1_000_000_000;

#[derive(Debug, Clone, PartialEq)]
pub struct Tensor {
    pub shape: Vec<usize>,
    pub data:  Vec<f32>,
}

impl Tensor {
    pub fn new(shape: Vec<usize>, data: Vec<f32>) -> AiResult<Self> {
        if shape.is_empty() { return Err(AiError::InvalidShape(shape)); }
        let numel = shape.iter().product::<usize>();
        if numel == 0 { return Err(AiError::EmptyTensor); }
        if numel > MAX_TENSOR_ELEMENTS {
            return Err(AiError::InvalidShape(shape));
        }
        if data.len() != numel {
            return Err(AiError::ShapeMismatch {
                expected: vec![numel], got: vec![data.len()]
            });
        }
        Ok(Tensor { shape, data })
    }

    pub fn zeros(shape: Vec<usize>) -> AiResult<Self> {
        let numel = shape.iter().product::<usize>();
        Self::new(shape, vec![0.0f32; numel])
    }

    pub fn ones(shape: Vec<usize>) -> AiResult<Self> {
        let numel = shape.iter().product::<usize>();
        Self::new(shape, vec![1.0f32; numel])
    }

    pub fn from_scalar(val: f32) -> Self {
        Tensor { shape: vec![1], data: vec![val] }
    }

    pub fn from_vec(data: Vec<f32>) -> AiResult<Self> {
        let n = data.len();
        Self::new(vec![n], data)
    }

    pub fn from_matrix(rows: usize, cols: usize, data: Vec<f32>) -> AiResult<Self> {
        Self::new(vec![rows, cols], data)
    }

    pub fn numel(&self) -> usize { self.data.len() }
    pub fn ndim(&self)  -> usize { self.shape.len() }
    pub fn rank(&self)  -> usize { self.ndim() }

    pub fn is_scalar(&self) -> bool { self.shape == vec![1] }
    pub fn is_vector(&self) -> bool { self.ndim() == 1 }
    pub fn is_matrix(&self) -> bool { self.ndim() == 2 }

    pub fn get(&self, indices: &[usize]) -> AiResult<f32> {
        let idx = self.flat_index(indices)?;
        Ok(self.data[idx])
    }

    pub fn set(&mut self, indices: &[usize], val: f32) -> AiResult<()> {
        let idx = self.flat_index(indices)?;
        self.data[idx] = val;
        Ok(())
    }

    fn flat_index(&self, indices: &[usize]) -> AiResult<usize> {
        if indices.len() != self.shape.len() {
            return Err(AiError::ShapeMismatch {
                expected: self.shape.clone(),
                got: indices.to_vec(),
            });
        }
        let mut idx = 0usize;
        let mut stride = 1usize;
        for (i, (&dim, &index)) in self.shape.iter().zip(indices.iter()).enumerate().rev() {
            if index >= dim {
                return Err(AiError::ShapeMismatch {
                    expected: self.shape.clone(),
                    got: indices.to_vec(),
                });
            }
            idx += index * stride;
            stride *= dim;
        }
        Ok(idx)
    }

    pub fn reshape(&self, new_shape: Vec<usize>) -> AiResult<Tensor> {
        let new_numel = new_shape.iter().product::<usize>();
        if new_numel != self.numel() {
            return Err(AiError::ShapeMismatch {
                expected: vec![self.numel()],
                got: vec![new_numel],
            });
        }
        Ok(Tensor { shape: new_shape, data: self.data.clone() })
    }

    pub fn transpose(&self) -> AiResult<Tensor> {
        if self.ndim() != 2 {
            return Err(AiError::InvalidShape(self.shape.clone()));
        }
        let rows = self.shape[0];
        let cols = self.shape[1];
        let mut data = vec![0.0f32; rows * cols];
        for r in 0..rows {
            for c in 0..cols {
                data[c * rows + r] = self.data[r * cols + c];
            }
        }
        Tensor::new(vec![cols, rows], data)
    }

    pub fn sum(&self) -> f32 { self.data.iter().sum() }
    pub fn mean(&self) -> f32 { self.sum() / self.numel() as f32 }
    pub fn max(&self) -> f32 { self.data.iter().cloned().fold(f32::NEG_INFINITY, f32::max) }
    pub fn min(&self) -> f32 { self.data.iter().cloned().fold(f32::INFINITY, f32::min) }

    pub fn map(&self, f: impl Fn(f32) -> f32) -> Tensor {
        Tensor { shape: self.shape.clone(), data: self.data.iter().map(|&x| f(x)).collect() }
    }

    pub fn elementwise(&self, other: &Tensor, f: impl Fn(f32,f32) -> f32) -> AiResult<Tensor> {
        if self.shape != other.shape {
            return Err(AiError::ShapeMismatch {
                expected: self.shape.clone(), got: other.shape.clone()
            });
        }
        let data = self.data.iter().zip(other.data.iter()).map(|(&a,&b)| f(a,b)).collect();
        Ok(Tensor { shape: self.shape.clone(), data })
    }

    pub fn add(&self, other: &Tensor) -> AiResult<Tensor> {
        self.elementwise(other, |a,b| a+b)
    }

    pub fn sub(&self, other: &Tensor) -> AiResult<Tensor> {
        self.elementwise(other, |a,b| a-b)
    }

    pub fn mul(&self, other: &Tensor) -> AiResult<Tensor> {
        self.elementwise(other, |a,b| a*b)
    }

    pub fn scale(&self, s: f32) -> Tensor {
        self.map(|x| x * s)
    }
}
