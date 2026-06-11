// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
//
// axon_math::linalg — Linear Algebra Primitives
// Stack-allocated, const-generic Matrix<T, R, C>
// no_std | Kani-verified arithmetic paths

use core::ops::{Add, Mul, AddAssign};
use num_traits::{Zero, One, Float};

// ------------------------------------------------------------------
// AxonScalar — numeric trait bound used across all of axon_math
// ------------------------------------------------------------------
pub trait AxonScalar:
    Copy
    + Zero
    + One
    + Add<Output = Self>
    + Mul<Output = Self>
    + AddAssign
    + PartialOrd
{
}

// Blanket impl for f32, f64, and integer types that satisfy the bounds.
impl<T> AxonScalar for T where
    T: Copy
        + Zero
        + One
        + Add<Output = T>
        + Mul<Output = T>
        + AddAssign
        + PartialOrd
{
}

// ------------------------------------------------------------------
// Matrix<T, R, C> — const-generic, stack-allocated
// ------------------------------------------------------------------
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Matrix<T, const R: usize, const C: usize> {
    pub data: [[T; C]; R],
}

impl<T: AxonScalar, const R: usize, const C: usize> Matrix<T, R, C> {
    /// Create a matrix filled with zeros.
    #[inline]
    pub fn zeros() -> Self {
        Self {
            data: [[T::zero(); C]; R],
        }
    }

    /// Create a matrix from a raw 2-D array.
    #[inline]
    pub fn from_array(data: [[T; C]; R]) -> Self {
        Self { data }
    }

    /// Get element at (row, col).
    #[inline]
    pub fn get(&self, row: usize, col: usize) -> T {
        self.data[row][col]
    }

    /// Set element at (row, col).
    #[inline]
    pub fn set(&mut self, row: usize, col: usize, val: T) {
        self.data[row][col] = val;
    }
}

// Square identity matrix
impl<T: AxonScalar, const N: usize> Matrix<T, N, N> {
    /// Identity matrix (diagonal = One, rest = Zero).
    pub fn identity() -> Self {
        let mut m = Self::zeros();
        for i in 0..N {
            m.data[i][i] = T::one();
        }
        m
    }
}

// ------------------------------------------------------------------
// Matrix multiplication: (R x K) * (K x C) → (R x C)
// ------------------------------------------------------------------
/// Multiply two stack-allocated matrices.
/// Panics in debug if dimensions are incompatible (const-generic
/// enforces this at compile time via the type system).
pub fn matmul<T: AxonScalar, const R: usize, const K: usize, const C: usize>(
    a: &Matrix<T, R, K>,
    b: &Matrix<T, K, C>,
) -> Matrix<T, R, C> {
    let mut out = Matrix::<T, R, C>::zeros();
    for i in 0..R {
        for j in 0..C {
            for k in 0..K {
                out.data[i][j] += a.data[i][k] * b.data[k][j];
            }
        }
    }
    out
}

// ------------------------------------------------------------------
// Dot product of two fixed-length slices
// ------------------------------------------------------------------
/// Compute the dot product of two equal-length arrays.
pub fn dot<T: AxonScalar, const N: usize>(a: &[T; N], b: &[T; N]) -> T {
    let mut acc = T::zero();
    for i in 0..N {
        acc += a[i] * b[i];
    }
    acc
}

// ------------------------------------------------------------------
// Transpose: (R x C) → (C x R)
// ------------------------------------------------------------------
/// Return the transpose of a matrix.
pub fn transpose<T: AxonScalar, const R: usize, const C: usize>(
    m: &Matrix<T, R, C>,
) -> Matrix<T, C, R> {
    let mut out = Matrix::<T, C, R>::zeros();
    for i in 0..R {
        for j in 0..C {
            out.data[j][i] = m.data[i][j];
        }
    }
    out
}

// ------------------------------------------------------------------
// Frobenius norm (requires Float for sqrt)
// ------------------------------------------------------------------
/// Frobenius norm of a matrix: sqrt(sum of squares of all elements).
pub fn frobenius_norm<T: AxonScalar + Float, const R: usize, const C: usize>(
    m: &Matrix<T, R, C>,
) -> T {
    let mut sum = T::zero();
    for i in 0..R {
        for j in 0..C {
            sum += m.data[i][j] * m.data[i][j];
        }
    }
    sum.sqrt()
}

// ------------------------------------------------------------------
// Element-wise addition
// ------------------------------------------------------------------
pub fn mat_add<T: AxonScalar, const R: usize, const C: usize>(
    a: &Matrix<T, R, C>,
    b: &Matrix<T, R, C>,
) -> Matrix<T, R, C> {
    let mut out = Matrix::<T, R, C>::zeros();
    for i in 0..R {
        for j in 0..C {
            out.data[i][j] = a.data[i][j] + b.data[i][j];
        }
    }
    out
}

// ------------------------------------------------------------------
// Scalar multiply
// ------------------------------------------------------------------
pub fn mat_scale<T: AxonScalar, const R: usize, const C: usize>(
    m: &Matrix<T, R, C>,
    scalar: T,
) -> Matrix<T, R, C> {
    let mut out = *m;
    for i in 0..R {
        for j in 0..C {
            out.data[i][j] = out.data[i][j] * scalar;
        }
    }
    out
}
