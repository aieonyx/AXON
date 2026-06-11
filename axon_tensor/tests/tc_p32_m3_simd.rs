// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
//
// P32-M3 — SIMD dispatch tests (scalar path always active)

use axon_tensor::simd::{
    dot_f32, dot_f32_scalar, dot_f64_scalar,
    add_f32, add_f32_scalar, mul_f32_scalar,
};

// ------------------------------------------------------------------
// Scalar dot product
// ------------------------------------------------------------------

#[test]
fn tc_p32_m3_dot_f32_scalar_basic() {
    let a = [1.0f32, 2.0, 3.0, 4.0];
    let b = [4.0f32, 3.0, 2.0, 1.0];
    let r = dot_f32_scalar(&a, &b);
    assert!((r - 20.0).abs() < 1e-6);
}

#[test]
fn tc_p32_m3_dot_f64_scalar_basic() {
    let a = [1.0f64, 2.0, 3.0];
    let b = [4.0f64, 5.0, 6.0];
    let r = dot_f64_scalar(&a, &b);
    assert!((r - 32.0).abs() < 1e-12);
}

#[test]
fn tc_p32_m3_dot_f32_scalar_zeros() {
    let a = [0.0f32; 8];
    let b = [1.0f32; 8];
    assert_eq!(dot_f32_scalar(&a, &b), 0.0);
}

#[test]
fn tc_p32_m3_dot_f32_scalar_single() {
    let a = [7.0f32];
    let b = [3.0f32];
    assert!((dot_f32_scalar(&a, &b) - 21.0).abs() < 1e-6);
}

// ------------------------------------------------------------------
// SIMD dispatch (routes to scalar on non-x86 or without simd feature)
// ------------------------------------------------------------------

#[test]
fn tc_p32_m3_dot_f32_dispatch_basic() {
    let a = [1.0f32, 2.0, 3.0, 4.0];
    let b = [4.0f32, 3.0, 2.0, 1.0];
    let r = dot_f32(&a, &b);
    assert!((r - 20.0).abs() < 1e-5);
}

#[test]
fn tc_p32_m3_dot_f32_dispatch_large() {
    let n = 64;
    let a: Vec<f32> = (0..n).map(|i| i as f32).collect();
    let b: Vec<f32> = vec![1.0f32; n];
    let expected: f32 = (0..n).map(|i| i as f32).sum();
    let r = dot_f32(&a, &b);
    assert!((r - expected).abs() < 1e-3, "got {r}, expected {expected}");
}

#[test]
fn tc_p32_m3_dot_f32_dispatch_tail() {
    // Length 5 — not a multiple of 4, exercises tail path
    let a = [1.0f32, 2.0, 3.0, 4.0, 5.0];
    let b = [1.0f32; 5];
    let r = dot_f32(&a, &b);
    assert!((r - 15.0).abs() < 1e-5);
}

// ------------------------------------------------------------------
// Element-wise add
// ------------------------------------------------------------------

#[test]
fn tc_p32_m3_add_f32_scalar() {
    let a = [1.0f32, 2.0, 3.0, 4.0];
    let b = [10.0f32, 20.0, 30.0, 40.0];
    let mut out = [0.0f32; 4];
    add_f32_scalar(&a, &b, &mut out);
    assert_eq!(out, [11.0, 22.0, 33.0, 44.0]);
}

#[test]
fn tc_p32_m3_add_f32_dispatch() {
    let a = vec![1.0f32; 16];
    let b = vec![2.0f32; 16];
    let mut out = vec![0.0f32; 16];
    add_f32(&a, &b, &mut out);
    for &x in &out {
        assert!((x - 3.0).abs() < 1e-6);
    }
}

// ------------------------------------------------------------------
// Element-wise mul
// ------------------------------------------------------------------

#[test]
fn tc_p32_m3_mul_f32_scalar() {
    let a = [1.0f32, 2.0, 3.0, 4.0];
    let b = [2.0f32, 2.0, 2.0, 2.0];
    let mut out = [0.0f32; 4];
    mul_f32_scalar(&a, &b, &mut out);
    assert_eq!(out, [2.0, 4.0, 6.0, 8.0]);
}

// ------------------------------------------------------------------
// Scalar vs dispatch agreement
// ------------------------------------------------------------------

#[test]
fn tc_p32_m3_scalar_dispatch_agree() {
    let a: Vec<f32> = (0..32).map(|i| i as f32 * 0.1).collect();
    let b: Vec<f32> = (0..32).map(|i| i as f32 * 0.2).collect();
    let scalar   = dot_f32_scalar(&a, &b);
    let dispatch = dot_f32(&a, &b);
    assert!((scalar - dispatch).abs() < 1e-3,
        "scalar={scalar} dispatch={dispatch}");
}
