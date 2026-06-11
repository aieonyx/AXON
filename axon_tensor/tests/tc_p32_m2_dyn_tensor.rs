// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
//
// P32-M2 — DynTensor<T> dynamic-rank tests

use axon_tensor::DynTensor;
use axon_tensor::ops::TensorOps;

// ------------------------------------------------------------------
// Construction
// ------------------------------------------------------------------

#[test]
fn tc_p32_m2_dyn_zeros_rank1() {
    let t = DynTensor::<f32>::zeros(vec![8]);
    assert_eq!(t.numel(), 8);
    assert_eq!(t.rank(), 1);
    assert_eq!(t.get(&[0]), 0.0_f32);
}

#[test]
fn tc_p32_m2_dyn_zeros_rank2() {
    let t = DynTensor::<f64>::zeros(vec![3, 4]);
    assert_eq!(t.numel(), 12);
    assert_eq!(t.shape(), &[3usize, 4]);
}

#[test]
fn tc_p32_m2_dyn_zeros_rank4() {
    let t = DynTensor::<f32>::zeros(vec![2, 3, 4, 5]);
    assert_eq!(t.numel(), 120);
    assert_eq!(t.rank(), 4);
}

#[test]
fn tc_p32_m2_dyn_from_vec() {
    let data = vec![1.0f32, 2.0, 3.0, 4.0, 5.0, 6.0];
    let t = DynTensor::<f32>::from_vec(vec![2, 3], data);
    assert_eq!(t.get(&[0, 0]), 1.0);
    assert_eq!(t.get(&[1, 2]), 6.0);
}

// ------------------------------------------------------------------
// Strides
// ------------------------------------------------------------------

#[test]
fn tc_p32_m2_dyn_strides_rank3() {
    let t = DynTensor::<f32>::zeros(vec![2, 3, 4]);
    assert_eq!(t.strides(), &[12usize, 4, 1]);
}

// ------------------------------------------------------------------
// Set / get
// ------------------------------------------------------------------

#[test]
fn tc_p32_m2_dyn_set_get() {
    let mut t = DynTensor::<f64>::zeros(vec![4, 4]);
    t.set(&[2, 3], 42.0);
    assert_eq!(t.get(&[2, 3]), 42.0);
}

#[test]
fn tc_p32_m2_dyn_fill() {
    let mut t = DynTensor::<f32>::zeros(vec![3, 3]);
    t.fill(5.0);
    assert_eq!(t.sum(), 45.0_f32);
}

// ------------------------------------------------------------------
// Reshape
// ------------------------------------------------------------------

#[test]
fn tc_p32_m2_dyn_reshape() {
    let t = DynTensor::<f32>::from_vec(vec![2, 3], vec![1.0,2.0,3.0,4.0,5.0,6.0]);
    let r = t.reshape(vec![3, 2]);
    assert_eq!(r.shape(), &[3usize, 2]);
    assert_eq!(r.numel(), 6);
    assert_eq!(r.get(&[0, 0]), 1.0);
    assert_eq!(r.get(&[2, 1]), 6.0);
}

#[test]
fn tc_p32_m2_dyn_reshape_to_1d() {
    let t = DynTensor::<f64>::zeros(vec![3, 4]);
    let r = t.reshape(vec![12]);
    assert_eq!(r.rank(), 1);
    assert_eq!(r.numel(), 12);
}

// ------------------------------------------------------------------
// Ops
// ------------------------------------------------------------------

#[test]
fn tc_p32_m2_dyn_add() {
    let a = DynTensor::<f32>::from_vec(vec![2,2], vec![1.0,2.0,3.0,4.0]);
    let b = DynTensor::<f32>::from_vec(vec![2,2], vec![10.0,20.0,30.0,40.0]);
    let c = a.add(&b);
    assert_eq!(c.get(&[0,0]), 11.0);
    assert_eq!(c.get(&[1,1]), 44.0);
}

#[test]
fn tc_p32_m2_dyn_mul() {
    let a = DynTensor::<f64>::from_vec(vec![3], vec![1.0, 2.0, 3.0]);
    let b = DynTensor::<f64>::from_vec(vec![3], vec![4.0, 5.0, 6.0]);
    let c = a.mul(&b);
    assert!((c.get(&[0]) - 4.0).abs() < 1e-10);
    assert!((c.get(&[2]) - 18.0).abs() < 1e-10);
}

#[test]
fn tc_p32_m2_dyn_scale() {
    let t = DynTensor::<f32>::from_vec(vec![4], vec![1.0, 2.0, 3.0, 4.0]);
    let s = t.scale(2.0);
    assert_eq!(s.get(&[3]), 8.0);
}

#[test]
fn tc_p32_m2_dyn_matmul() {
    let a = DynTensor::<f64>::from_vec(vec![2,2], vec![1.0,2.0,3.0,4.0]);
    let b = DynTensor::<f64>::from_vec(vec![2,2], vec![5.0,6.0,7.0,8.0]);
    let c = a.matmul(&b).unwrap();
    assert!((c.get(&[0,0]) - 19.0).abs() < 1e-10);
    assert!((c.get(&[1,1]) - 50.0).abs() < 1e-10);
}

#[test]
fn tc_p32_m2_dyn_matmul_rect() {
    // (2x3) * (3x2) → (2x2)
    let a = DynTensor::<f64>::from_vec(vec![2,3], vec![1.0,0.0,2.0,0.0,3.0,1.0]);
    let b = DynTensor::<f64>::from_vec(vec![3,2], vec![1.0,0.0,0.0,1.0,2.0,1.0]);
    let c = a.matmul(&b).unwrap();
    assert!((c.get(&[0,0]) - 5.0).abs() < 1e-10);
    assert!((c.get(&[1,1]) - 4.0).abs() < 1e-10);
}

#[test]
fn tc_p32_m2_dyn_transpose() {
    let t = DynTensor::<f32>::from_vec(vec![2,3], vec![1.0,2.0,3.0,4.0,5.0,6.0]);
    let tr = t.transpose().unwrap();
    assert_eq!(tr.shape(), &[3usize, 2]);
    assert_eq!(tr.get(&[1, 0]), 2.0);
    assert_eq!(tr.get(&[2, 1]), 6.0);
}

#[test]
fn tc_p32_m2_dyn_slice_axis0() {
    let t = DynTensor::<f32>::from_vec(vec![4,2], vec![
        1.0,2.0, 3.0,4.0, 5.0,6.0, 7.0,8.0
    ]);
    let s = t.slice_axis0(1, 3).unwrap();
    assert_eq!(s.shape(), &[2usize, 2]);
    assert_eq!(s.get(&[0, 0]), 3.0);
    assert_eq!(s.get(&[1, 1]), 6.0);
}

#[test]
fn tc_p32_m2_dyn_slice_invalid() {
    let t = DynTensor::<f32>::zeros(vec![4, 2]);
    assert!(t.slice_axis0(3, 2).is_none()); // start >= end
    assert!(t.slice_axis0(2, 5).is_none()); // end > shape[0]
}
