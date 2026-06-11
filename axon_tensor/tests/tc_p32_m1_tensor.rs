// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
//
// P32-M1 — Tensor<T,D> const-generic tests

use axon_tensor::Tensor;
use axon_tensor::ops::TensorOps;

// ------------------------------------------------------------------
// Construction
// ------------------------------------------------------------------

#[test]
fn tc_p32_m1_tensor_zeros_rank1() {
    let t = Tensor::<f32, 1>::zeros([8]);
    assert_eq!(t.numel(), 8);
    assert_eq!(t.rank(), 1);
    for i in 0..8 {
        assert_eq!(t.get([i]), 0.0_f32);
    }
}

#[test]
fn tc_p32_m1_tensor_zeros_rank2() {
    let t = Tensor::<f64, 2>::zeros([3, 4]);
    assert_eq!(t.numel(), 12);
    assert_eq!(t.shape(), &[3, 4]);
}

#[test]
fn tc_p32_m1_tensor_zeros_rank3() {
    let t = Tensor::<f32, 3>::zeros([2, 3, 4]);
    assert_eq!(t.numel(), 24);
    assert_eq!(t.rank(), 3);
}

#[test]
fn tc_p32_m1_tensor_from_vec() {
    let data = vec![1.0f32, 2.0, 3.0, 4.0, 5.0, 6.0];
    let t = Tensor::<f32, 2>::from_vec([2, 3], data);
    assert_eq!(t.get([0, 0]), 1.0);
    assert_eq!(t.get([0, 2]), 3.0);
    assert_eq!(t.get([1, 0]), 4.0);
    assert_eq!(t.get([1, 2]), 6.0);
}

// ------------------------------------------------------------------
// Strides and indexing
// ------------------------------------------------------------------

#[test]
fn tc_p32_m1_tensor_strides_row_major() {
    let t = Tensor::<f32, 3>::zeros([2, 3, 4]);
    // row-major: strides = [12, 4, 1]
    assert_eq!(t.strides(), &[12, 4, 1]);
}

#[test]
fn tc_p32_m1_tensor_set_get_roundtrip() {
    let mut t = Tensor::<f64, 2>::zeros([4, 4]);
    t.set([2, 3], 99.0);
    assert_eq!(t.get([2, 3]), 99.0);
}

#[test]
fn tc_p32_m1_tensor_fill() {
    let mut t = Tensor::<f32, 2>::zeros([3, 3]);
    t.fill(7.0);
    for i in 0..3 {
        for j in 0..3 {
            assert_eq!(t.get([i, j]), 7.0);
        }
    }
}

// ------------------------------------------------------------------
// Ops
// ------------------------------------------------------------------

#[test]
fn tc_p32_m1_tensor_sum() {
    let data = vec![1.0f64, 2.0, 3.0, 4.0];
    let t = Tensor::<f64, 1>::from_vec([4], data);
    assert!((t.sum() - 10.0).abs() < 1e-10);
}

#[test]
fn tc_p32_m1_tensor_add() {
    let a = Tensor::<f32, 2>::from_vec([2, 2], vec![1.0, 2.0, 3.0, 4.0]);
    let b = Tensor::<f32, 2>::from_vec([2, 2], vec![10.0, 20.0, 30.0, 40.0]);
    let c = a.add(&b);
    assert_eq!(c.get([0, 0]), 11.0);
    assert_eq!(c.get([1, 1]), 44.0);
}

#[test]
fn tc_p32_m1_tensor_mul_elementwise() {
    let a = Tensor::<f32, 2>::from_vec([2, 2], vec![1.0, 2.0, 3.0, 4.0]);
    let b = Tensor::<f32, 2>::from_vec([2, 2], vec![2.0, 2.0, 2.0, 2.0]);
    let c = a.mul(&b);
    assert_eq!(c.get([0, 0]), 2.0);
    assert_eq!(c.get([1, 1]), 8.0);
}

#[test]
fn tc_p32_m1_tensor_scale() {
    let t = Tensor::<f64, 1>::from_vec([4], vec![1.0, 2.0, 3.0, 4.0]);
    let s = t.scale(3.0);
    assert_eq!(s.get([0]), 3.0);
    assert_eq!(s.get([3]), 12.0);
}

#[test]
fn tc_p32_m1_tensor_matmul_2d() {
    // [1 2] * [5 6] = [19 22]
    // [3 4]   [7 8]   [43 50]
    let a = Tensor::<f64, 2>::from_vec([2, 2], vec![1.0, 2.0, 3.0, 4.0]);
    let b = Tensor::<f64, 2>::from_vec([2, 2], vec![5.0, 6.0, 7.0, 8.0]);
    let c = a.matmul_2d(&b).unwrap();
    assert!((c.get([0, 0]) - 19.0).abs() < 1e-10);
    assert!((c.get([0, 1]) - 22.0).abs() < 1e-10);
    assert!((c.get([1, 0]) - 43.0).abs() < 1e-10);
    assert!((c.get([1, 1]) - 50.0).abs() < 1e-10);
}

#[test]
fn tc_p32_m1_tensor_transpose_2d() {
    let t = Tensor::<f32, 2>::from_vec([2, 3], vec![1.0,2.0,3.0,4.0,5.0,6.0]);
    let tr = t.transpose_2d().unwrap();
    assert_eq!(tr.shape(), &[3, 2]);
    assert_eq!(tr.get([0, 0]), 1.0);
    assert_eq!(tr.get([1, 0]), 2.0);
    assert_eq!(tr.get([2, 1]), 6.0);
}

#[test]
fn tc_p32_m1_tensor_matmul_wrong_rank() {
    let t = Tensor::<f32, 3>::zeros([2, 2, 2]);
    assert!(t.matmul_2d(&t).is_none());
}
