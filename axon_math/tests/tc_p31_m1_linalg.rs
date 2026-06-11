// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
//
// P31-M1 — Linear Algebra Tests

use axon_math::linalg::{Matrix, matmul, dot, transpose, frobenius_norm, mat_add, mat_scale};

// ------------------------------------------------------------------
// Matrix construction
// ------------------------------------------------------------------

#[test]
fn tc_p31_m1_matrix_zeros() {
    let m = Matrix::<f64, 3, 3>::zeros();
    for i in 0..3 {
        for j in 0..3 {
            assert_eq!(m.get(i, j), 0.0);
        }
    }
}

#[test]
fn tc_p31_m1_matrix_identity() {
    let m = Matrix::<f64, 3, 3>::identity();
    for i in 0..3 {
        for j in 0..3 {
            let expected = if i == j { 1.0 } else { 0.0 };
            assert_eq!(m.get(i, j), expected);
        }
    }
}

#[test]
fn tc_p31_m1_matrix_from_array() {
    let m = Matrix::<f32, 2, 2>::from_array([[1.0, 2.0], [3.0, 4.0]]);
    assert_eq!(m.get(0, 0), 1.0_f32);
    assert_eq!(m.get(1, 1), 4.0_f32);
}

// ------------------------------------------------------------------
// Dot product
// ------------------------------------------------------------------

#[test]
fn tc_p31_m1_dot_product_basic() {
    let a = [1.0f64, 2.0, 3.0];
    let b = [4.0f64, 5.0, 6.0];
    let result = dot(&a, &b);
    assert!((result - 32.0).abs() < 1e-10, "dot product should be 32.0");
}

#[test]
fn tc_p31_m1_dot_product_orthogonal() {
    let a = [1.0f64, 0.0, 0.0];
    let b = [0.0f64, 1.0, 0.0];
    let result = dot(&a, &b);
    assert!((result - 0.0).abs() < 1e-10, "orthogonal vectors: dot = 0");
}

#[test]
fn tc_p31_m1_dot_product_single() {
    let a = [7.0f32];
    let b = [3.0f32];
    assert!((dot(&a, &b) - 21.0).abs() < 1e-6);
}

// ------------------------------------------------------------------
// Matrix multiply
// ------------------------------------------------------------------

#[test]
fn tc_p31_m1_matmul_2x2() {
    let a = Matrix::<f64, 2, 2>::from_array([[1.0, 2.0], [3.0, 4.0]]);
    let b = Matrix::<f64, 2, 2>::from_array([[5.0, 6.0], [7.0, 8.0]]);
    let c = matmul(&a, &b);
    // [1*5+2*7, 1*6+2*8] = [19, 22]
    // [3*5+4*7, 3*6+4*8] = [43, 50]
    assert!((c.get(0, 0) - 19.0).abs() < 1e-10);
    assert!((c.get(0, 1) - 22.0).abs() < 1e-10);
    assert!((c.get(1, 0) - 43.0).abs() < 1e-10);
    assert!((c.get(1, 1) - 50.0).abs() < 1e-10);
}

#[test]
fn tc_p31_m1_matmul_identity_left() {
    let id = Matrix::<f64, 3, 3>::identity();
    let a  = Matrix::<f64, 3, 3>::from_array([
        [1.0, 2.0, 3.0],
        [4.0, 5.0, 6.0],
        [7.0, 8.0, 9.0],
    ]);
    let result = matmul(&id, &a);
    for i in 0..3 {
        for j in 0..3 {
            assert!((result.get(i, j) - a.get(i, j)).abs() < 1e-10);
        }
    }
}

#[test]
fn tc_p31_m1_matmul_rect_2x3_3x2() {
    // (2x3) * (3x2) → (2x2)
    let a = Matrix::<f64, 2, 3>::from_array([[1.0, 0.0, 2.0], [0.0, 3.0, 1.0]]);
    let b = Matrix::<f64, 3, 2>::from_array([[1.0, 0.0], [0.0, 1.0], [2.0, 1.0]]);
    let c = matmul(&a, &b);
    // row0: [1+0+4, 0+0+2] = [5, 2]
    // row1: [0+0+2, 0+3+1] = [2, 4]
    assert!((c.get(0, 0) - 5.0).abs() < 1e-10);
    assert!((c.get(0, 1) - 2.0).abs() < 1e-10);
    assert!((c.get(1, 0) - 2.0).abs() < 1e-10);
    assert!((c.get(1, 1) - 4.0).abs() < 1e-10);
}

// ------------------------------------------------------------------
// Transpose
// ------------------------------------------------------------------

#[test]
fn tc_p31_m1_transpose_2x3() {
    let m = Matrix::<f64, 2, 3>::from_array([[1.0, 2.0, 3.0], [4.0, 5.0, 6.0]]);
    let t = transpose(&m);
    assert_eq!(t.get(0, 0), 1.0);
    assert_eq!(t.get(1, 0), 2.0);
    assert_eq!(t.get(2, 0), 3.0);
    assert_eq!(t.get(0, 1), 4.0);
    assert_eq!(t.get(2, 1), 6.0);
}

#[test]
fn tc_p31_m1_transpose_double_identity() {
    let m = Matrix::<f64, 3, 3>::from_array([
        [1.0, 2.0, 3.0],
        [4.0, 5.0, 6.0],
        [7.0, 8.0, 9.0],
    ]);
    let tt = transpose(&transpose(&m));
    for i in 0..3 {
        for j in 0..3 {
            assert!((tt.get(i, j) - m.get(i, j)).abs() < 1e-10);
        }
    }
}

// ------------------------------------------------------------------
// Frobenius norm
// ------------------------------------------------------------------

#[test]
fn tc_p31_m1_frobenius_norm_identity() {
    let id = Matrix::<f64, 3, 3>::identity();
    let norm = frobenius_norm(&id);
    // ||I_3|| = sqrt(3)
    assert!((norm - 3.0_f64.sqrt()).abs() < 1e-10);
}

#[test]
fn tc_p31_m1_frobenius_norm_known() {
    let m = Matrix::<f64, 2, 2>::from_array([[3.0, 4.0], [0.0, 0.0]]);
    let norm = frobenius_norm(&m);
    assert!((norm - 5.0).abs() < 1e-10);
}

// ------------------------------------------------------------------
// Element-wise ops
// ------------------------------------------------------------------

#[test]
fn tc_p31_m1_mat_add() {
    let a = Matrix::<f64, 2, 2>::from_array([[1.0, 2.0], [3.0, 4.0]]);
    let b = Matrix::<f64, 2, 2>::from_array([[10.0, 20.0], [30.0, 40.0]]);
    let c = mat_add(&a, &b);
    assert_eq!(c.get(0, 0), 11.0);
    assert_eq!(c.get(1, 1), 44.0);
}

#[test]
fn tc_p31_m1_mat_scale() {
    let m = Matrix::<f64, 2, 2>::from_array([[1.0, 2.0], [3.0, 4.0]]);
    let s = mat_scale(&m, 2.0);
    assert_eq!(s.get(0, 0), 2.0);
    assert_eq!(s.get(1, 1), 8.0);
}

#[test]
fn tc_p31_m1_mat_scale_zero() {
    let m = Matrix::<f64, 2, 2>::from_array([[5.0, 6.0], [7.0, 8.0]]);
    let s = mat_scale(&m, 0.0);
    for i in 0..2 {
        for j in 0..2 {
            assert_eq!(s.get(i, j), 0.0);
        }
    }
}
