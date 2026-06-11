// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
//
// P33-M2 — Neural Network Layer Tests

use axon_learn::layers::{Linear, relu, relu_backward, softmax, gelu};
use axon_tensor::DynTensor;
use axon_tensor::ops::TensorOps;

// ------------------------------------------------------------------
// Linear layer
// ------------------------------------------------------------------

#[test]
fn tc_p33_m2_linear_zeros_output() {
    let layer = Linear::new(4, 3);
    let input = DynTensor::zeros(vec![2, 4]);
    let out = layer.forward(&input);
    assert_eq!(out.shape(), &[2usize, 3]);
    for i in 0..6 { assert_eq!(out.get_flat(i), 0.0); }
}

#[test]
fn tc_p33_m2_linear_identity_weight() {
    // 2x2 identity weight, zero bias: output == input
    let layer = Linear::new(2, 2)
        .with_weights(vec![1.0, 0.0, 0.0, 1.0]);
    let input = DynTensor::from_vec(vec![1, 2], vec![3.0, 4.0]);
    let out = layer.forward(&input);
    assert!((out.get(&[0, 0]) - 3.0).abs() < 1e-6);
    assert!((out.get(&[0, 1]) - 4.0).abs() < 1e-6);
}

#[test]
fn tc_p33_m2_linear_bias_only() {
    // Zero weight, non-zero bias: output == bias
    let layer = Linear::new(2, 3)
        .with_bias(vec![1.0, 2.0, 3.0]);
    let input = DynTensor::zeros(vec![1, 2]);
    let out = layer.forward(&input);
    assert!((out.get(&[0, 0]) - 1.0).abs() < 1e-6);
    assert!((out.get(&[0, 1]) - 2.0).abs() < 1e-6);
    assert!((out.get(&[0, 2]) - 3.0).abs() < 1e-6);
}

#[test]
fn tc_p33_m2_linear_forward_known() {
    // W = [[1,2],[3,4]], b = [0,0]
    // x = [[1,0],[0,1]] → y = [[1,3],[2,4]]
    let layer = Linear::new(2, 2)
        .with_weights(vec![1.0, 2.0, 3.0, 4.0]);
    let input = DynTensor::from_vec(vec![2, 2],
        vec![1.0, 0.0, 0.0, 1.0]);
    let out = layer.forward(&input);
    assert!((out.get(&[0, 0]) - 1.0).abs() < 1e-6);
    assert!((out.get(&[0, 1]) - 3.0).abs() < 1e-6);
    assert!((out.get(&[1, 0]) - 2.0).abs() < 1e-6);
    assert!((out.get(&[1, 1]) - 4.0).abs() < 1e-6);
}

#[test]
fn tc_p33_m2_linear_backward_grad_input_shape() {
    let layer = Linear::new(3, 2);
    let input = DynTensor::zeros(vec![4, 3]);
    let grad_out = DynTensor::zeros(vec![4, 2]);
    let (gi, gw, gb) = layer.backward(&input, &grad_out);
    assert_eq!(gi.shape(), &[4usize, 3]);
    assert_eq!(gw.shape(), &[2usize, 3]);
    assert_eq!(gb.shape(), &[2usize]);
}

// ------------------------------------------------------------------
// ReLU
// ------------------------------------------------------------------

#[test]
fn tc_p33_m2_relu_positive() {
    let x = DynTensor::from_vec(vec![4],
        vec![1.0, -2.0, 3.0, -4.0]);
    let y = relu(&x);
    assert_eq!(y.get(&[0]), 1.0);
    assert_eq!(y.get(&[1]), 0.0);
    assert_eq!(y.get(&[2]), 3.0);
    assert_eq!(y.get(&[3]), 0.0);
}

#[test]
fn tc_p33_m2_relu_all_negative() {
    let x = DynTensor::from_vec(vec![3], vec![-1.0, -2.0, -3.0]);
    let y = relu(&x);
    for i in 0..3 { assert_eq!(y.get(&[i]), 0.0); }
}

#[test]
fn tc_p33_m2_relu_backward() {
    let x = DynTensor::from_vec(vec![4],
        vec![1.0, -1.0, 2.0, -2.0]);
    let grad = DynTensor::from_vec(vec![4],
        vec![1.0, 1.0, 1.0, 1.0]);
    let g = relu_backward(&x, &grad);
    assert_eq!(g.get(&[0]), 1.0);
    assert_eq!(g.get(&[1]), 0.0);
    assert_eq!(g.get(&[2]), 1.0);
    assert_eq!(g.get(&[3]), 0.0);
}

// ------------------------------------------------------------------
// Softmax
// ------------------------------------------------------------------

#[test]
fn tc_p33_m2_softmax_sums_to_one() {
    let x = DynTensor::from_vec(vec![1, 4],
        vec![1.0, 2.0, 3.0, 4.0]);
    let y = softmax(&x);
    let s: f32 = (0..4).map(|j| y.get(&[0, j])).sum();
    assert!((s - 1.0).abs() < 1e-6, "softmax sum = {s}");
}

#[test]
fn tc_p33_m2_softmax_uniform_input() {
    // Equal logits → equal probabilities = 1/4
    let x = DynTensor::from_vec(vec![1, 4],
        vec![1.0, 1.0, 1.0, 1.0]);
    let y = softmax(&x);
    for j in 0..4 {
        assert!((y.get(&[0, j]) - 0.25).abs() < 1e-6);
    }
}

#[test]
fn tc_p33_m2_softmax_large_value_stable() {
    // Large values should not produce NaN/Inf (numerical stability)
    let x = DynTensor::from_vec(vec![1, 3],
        vec![1000.0, 1001.0, 1002.0]);
    let y = softmax(&x);
    let s: f32 = (0..3).map(|j| y.get(&[0, j])).sum();
    assert!((s - 1.0).abs() < 1e-5);
    for j in 0..3 { assert!(y.get(&[0, j]).is_finite()); }
}

#[test]
fn tc_p33_m2_softmax_batch() {
    let x = DynTensor::from_vec(vec![2, 3],
        vec![1.0, 2.0, 3.0, 1.0, 1.0, 1.0]);
    let y = softmax(&x);
    // Each row sums to 1
    for b in 0..2 {
        let s: f32 = (0..3).map(|j| y.get(&[b, j])).sum();
        assert!((s - 1.0).abs() < 1e-6);
    }
}

// ------------------------------------------------------------------
// GELU
// ------------------------------------------------------------------

#[test]
fn tc_p33_m2_gelu_zero() {
    let x = DynTensor::from_vec(vec![1], vec![0.0]);
    let y = gelu(&x);
    assert!((y.get(&[0]) - 0.0).abs() < 1e-6);
}

#[test]
fn tc_p33_m2_gelu_positive_large() {
    // GELU(x) ≈ x for large positive x
    let x = DynTensor::from_vec(vec![1], vec![10.0]);
    let y = gelu(&x);
    assert!((y.get(&[0]) - 10.0).abs() < 1e-3);
}

#[test]
fn tc_p33_m2_gelu_negative_large() {
    // GELU(x) ≈ 0 for large negative x
    let x = DynTensor::from_vec(vec![1], vec![-10.0]);
    let y = gelu(&x);
    assert!(y.get(&[0]).abs() < 1e-3);
}

#[test]
fn tc_p33_m2_gelu_shape_preserved() {
    let x = DynTensor::zeros(vec![3, 4]);
    let y = gelu(&x);
    assert_eq!(y.shape(), x.shape());
}
