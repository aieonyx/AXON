// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
//
// axon_math::ffi — Bootstrap FFI Bridge (std-gated, P31-M4)
// Wraps nalgebra SVD for validation against native AXON implementations.
// This module is scaffolding — replaced by native axon_tensor in Phase 32.

#[cfg(feature = "ffi")]
pub mod nalgebra_bridge {
    use nalgebra::{DMatrix, DVector};

    /// Thin wrapper: compute SVD of a row-major f64 slice.
    /// Returns (U, singular_values, V_t) as flat Vec<f64>.
    /// Used to validate axon_math linalg results — not for production paths.
    pub fn svd_f64(
        data: &[f64],
        rows: usize,
        cols: usize,
    ) -> Option<(Vec<f64>, Vec<f64>, Vec<f64>)> {
        if data.len() != rows * cols {
            return None;
        }
        let m = DMatrix::from_row_slice(rows, cols, data);
        let svd = m.svd(true, true);
        let u   = svd.u?.data.as_vec().clone();
        let s   = svd.singular_values.data.as_vec().clone();
        let vt  = svd.v_t?.data.as_vec().clone();
        Some((u, s, vt))
    }

    /// Compute the dot product of two f64 slices via nalgebra.
    /// Used as reference implementation for Kani cross-checks.
    pub fn dot_f64(a: &[f64], b: &[f64]) -> Option<f64> {
        if a.len() != b.len() { return None; }
        let va = DVector::from_column_slice(a);
        let vb = DVector::from_column_slice(b);
        Some(va.dot(&vb))
    }
}
