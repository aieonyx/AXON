// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
//
// P33-M3/M4 — Loss Functions + Optimizer Tests

use axon_learn::loss::{mse, mse_backward, cross_entropy, cross_entropy_backward, accuracy};
use axon_learn::optim::{Sgd, Adam};
use axon_tensor::DynTensor;
use axon_tensor::ops::TensorOps;

// ------------------------------------------------------------------
// MSE Loss
// ------------------------------------------------------------------

#[test]
fn tc_p33_m3_mse_zero() {
    let pred   = DynTensor::from_vec(vec![4], vec![1.0,2.0,3.0,4.0]);
    let target = DynTensor::from_vec(vec![4], vec![1.0,2.0,3.0,4.0]);
    assert!((mse(&pred, &target) - 0.0).abs() < 1e-10);
}

#[test]
fn tc_p33_m3_mse_known() {
    // pred=[0,0], target=[1,1] → MSE = (1+1)/2 = 1.0
    let pred   = DynTensor::from_vec(vec![2], vec![0.0, 0.0]);
    let target = DynTensor::from_vec(vec![2], vec![1.0, 1.0]);
    assert!((mse(&pred, &target) - 1.0).abs() < 1e-6);
}

#[test]
fn tc_p33_m3_mse_nonnegative() {
    let pred   = DynTensor::from_vec(vec![3], vec![1.0, 5.0, -2.0]);
    let target = DynTensor::from_vec(vec![3], vec![2.0, 3.0,  1.0]);
    assert!(mse(&pred, &target) >= 0.0);
}

#[test]
fn tc_p33_m3_mse_backward_shape() {
    let pred   = DynTensor::from_vec(vec![2,3], vec![1.0;6]);
    let target = DynTensor::zeros(vec![2,3]);
    let g = mse_backward(&pred, &target);
    assert_eq!(g.shape(), pred.shape());
}

#[test]
fn tc_p33_m3_mse_backward_values() {
    // pred=[1,0], target=[0,0] → grad = 2*(1-0)/2 = 1.0, 2*(0-0)/2 = 0.0
    let pred   = DynTensor::from_vec(vec![2], vec![1.0, 0.0]);
    let target = DynTensor::zeros(vec![2]);
    let g = mse_backward(&pred, &target);
    assert!((g.get(&[0]) - 1.0).abs() < 1e-6);
    assert!((g.get(&[1]) - 0.0).abs() < 1e-6);
}

// ------------------------------------------------------------------
// CrossEntropy Loss
// ------------------------------------------------------------------

#[test]
fn tc_p33_m3_ce_perfect_prediction() {
    // Softmax already applied: probs = one-hot → loss ≈ 0
    let probs  = DynTensor::from_vec(vec![1,3], vec![0.999, 0.0005, 0.0005]);
    let labels = DynTensor::from_vec(vec![1,3], vec![1.0,   0.0,    0.0]);
    let loss = cross_entropy(&probs, &labels);
    assert!(loss < 0.01, "loss = {loss}");
}

#[test]
fn tc_p33_m3_ce_nonnegative() {
    let probs  = DynTensor::from_vec(vec![2,3],
        vec![0.7,0.2,0.1, 0.1,0.6,0.3]);
    let labels = DynTensor::from_vec(vec![2,3],
        vec![1.0,0.0,0.0, 0.0,1.0,0.0]);
    assert!(cross_entropy(&probs, &labels) >= 0.0);
}

#[test]
fn tc_p33_m3_ce_backward_shape() {
    let probs  = DynTensor::from_vec(vec![2,3],
        vec![0.7,0.2,0.1, 0.1,0.6,0.3]);
    let labels = DynTensor::from_vec(vec![2,3],
        vec![1.0,0.0,0.0, 0.0,1.0,0.0]);
    let g = cross_entropy_backward(&probs, &labels);
    assert_eq!(g.shape(), probs.shape());
}

#[test]
fn tc_p33_m3_ce_backward_sum_zero() {
    // Gradient rows should sum to ~0 (prob - label, label sums to 1)
    let probs  = DynTensor::from_vec(vec![1,3],
        vec![0.33, 0.33, 0.34]);
    let labels = DynTensor::from_vec(vec![1,3],
        vec![1.0, 0.0, 0.0]);
    let g = cross_entropy_backward(&probs, &labels);
    let row_sum: f32 = (0..3).map(|j| g.get(&[0, j])).sum();
    assert!(row_sum.abs() < 1e-5, "grad row sum = {row_sum}");
}

// ------------------------------------------------------------------
// Accuracy
// ------------------------------------------------------------------

#[test]
fn tc_p33_m3_accuracy_perfect() {
    let probs  = DynTensor::from_vec(vec![2,3],
        vec![0.9,0.05,0.05, 0.05,0.9,0.05]);
    let labels = DynTensor::from_vec(vec![2,3],
        vec![1.0,0.0,0.0,   0.0,1.0,0.0]);
    assert!((accuracy(&probs, &labels) - 1.0).abs() < 1e-6);
}

#[test]
fn tc_p33_m3_accuracy_zero() {
    let probs  = DynTensor::from_vec(vec![2,3],
        vec![0.05,0.05,0.9, 0.05,0.05,0.9]);
    let labels = DynTensor::from_vec(vec![2,3],
        vec![1.0,0.0,0.0,   0.0,1.0,0.0]);
    assert!((accuracy(&probs, &labels) - 0.0).abs() < 1e-6);
}

// ------------------------------------------------------------------
// SGD Optimizer
// ------------------------------------------------------------------

#[test]
fn tc_p33_m4_sgd_step_decreases_param() {
    let mut sgd = Sgd::new(0.1, 0.0);
    let mut param = DynTensor::from_vec(vec![3], vec![1.0, 2.0, 3.0]);
    let grad      = DynTensor::from_vec(vec![3], vec![1.0, 1.0, 1.0]);
    sgd.step(&mut [(&mut param, &grad)]);
    assert!((param.get(&[0]) - 0.9).abs() < 1e-6);
    assert!((param.get(&[1]) - 1.9).abs() < 1e-6);
    assert!((param.get(&[2]) - 2.9).abs() < 1e-6);
}

#[test]
fn tc_p33_m4_sgd_zero_grad_no_change() {
    let mut sgd = Sgd::new(0.1, 0.0);
    let mut param = DynTensor::from_vec(vec![3], vec![1.0, 2.0, 3.0]);
    let grad      = DynTensor::zeros(vec![3]);
    sgd.step(&mut [(&mut param, &grad)]);
    assert_eq!(param.get(&[0]), 1.0);
    assert_eq!(param.get(&[1]), 2.0);
    assert_eq!(param.get(&[2]), 3.0);
}

#[test]
fn tc_p33_m4_sgd_momentum() {
    let mut sgd = Sgd::new(0.1, 0.9);
    let mut param = DynTensor::from_vec(vec![1], vec![1.0]);
    let grad      = DynTensor::from_vec(vec![1], vec![1.0]);
    // Step 1: v = 0.9*0 + 1 = 1; w = 1.0 - 0.1*1 = 0.9
    sgd.step(&mut [(&mut param, &grad)]);
    assert!((param.get(&[0]) - 0.9).abs() < 1e-6);
    // Step 2: v = 0.9*1 + 1 = 1.9; w = 0.9 - 0.1*1.9 = 0.71
    sgd.step(&mut [(&mut param, &grad)]);
    assert!((param.get(&[0]) - 0.71).abs() < 1e-5);
}

// ------------------------------------------------------------------
// Adam Optimizer
// ------------------------------------------------------------------

#[test]
fn tc_p33_m4_adam_step_updates_param() {
    let mut adam = Adam::new(0.01);
    let mut param = DynTensor::from_vec(vec![3], vec![1.0, 1.0, 1.0]);
    let grad      = DynTensor::from_vec(vec![3], vec![1.0, 1.0, 1.0]);
    let before = param.get(&[0]);
    adam.step(&mut [(&mut param, &grad)]);
    assert!(param.get(&[0]) < before, "param should decrease");
}

#[test]
fn tc_p33_m4_adam_zero_grad_no_change() {
    let mut adam = Adam::new(0.01);
    let mut param = DynTensor::from_vec(vec![3], vec![1.0, 2.0, 3.0]);
    let grad      = DynTensor::zeros(vec![3]);
    adam.step(&mut [(&mut param, &grad)]);
    assert_eq!(param.get(&[0]), 1.0);
}

#[test]
fn tc_p33_m4_adam_step_counter() {
    let mut adam = Adam::new(0.001);
    let mut param = DynTensor::zeros(vec![2]);
    let grad      = DynTensor::zeros(vec![2]);
    assert_eq!(adam.current_step(), 0);
    adam.step(&mut [(&mut param, &grad)]);
    assert_eq!(adam.current_step(), 1);
    adam.step(&mut [(&mut param, &grad)]);
    assert_eq!(adam.current_step(), 2);
}

#[test]
fn tc_p33_m4_adam_reset() {
    let mut adam = Adam::new(0.001);
    let mut param = DynTensor::from_vec(vec![2], vec![1.0, 1.0]);
    let grad      = DynTensor::from_vec(vec![2], vec![1.0, 1.0]);
    adam.step(&mut [(&mut param, &grad)]);
    adam.reset();
    assert_eq!(adam.current_step(), 0);
}

#[test]
fn tc_p33_m4_adam_converges_simple() {
    // Adam should drive a single parameter toward zero
    // given a consistent gradient pointing away from 0
    let mut adam = Adam::new(0.1);
    let mut param = DynTensor::from_vec(vec![1], vec![5.0]);
    let grad = DynTensor::from_vec(vec![1], vec![1.0]);
    for _ in 0..100 {
        adam.step(&mut [(&mut param, &grad)]);
    }
    assert!(param.get(&[0]) < 4.0, "param = {}", param.get(&[0]));
}
