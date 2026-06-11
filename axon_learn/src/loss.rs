// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
//
// axon_learn::loss — Loss Functions
// MSE (Mean Squared Error), CrossEntropy
// Operate on DynTensor<f32>.

use axon_tensor::DynTensor;
use axon_tensor::ops::TensorOps;
use alloc::vec::Vec;

// ------------------------------------------------------------------
// MSE — Mean Squared Error
// loss = mean((pred - target)²)
// ------------------------------------------------------------------

/// Compute MSE loss between predictions and targets.
/// Both must have the same shape. Returns scalar f32.
pub fn mse(pred: &DynTensor<f32>, target: &DynTensor<f32>) -> f32 {
    assert_eq!(pred.shape(), target.shape(), "mse: shape mismatch");
    let n = pred.numel();
    assert!(n > 0, "mse: empty tensors");
    let sum: f32 = pred.data().iter()
        .zip(target.data().iter())
        .map(|(&p, &t)| { let d = p - t; d * d })
        .sum();
    sum / n as f32
}

/// MSE backward: grad_pred = 2 * (pred - target) / n
pub fn mse_backward(
    pred: &DynTensor<f32>,
    target: &DynTensor<f32>,
) -> DynTensor<f32> {
    assert_eq!(pred.shape(), target.shape(), "mse_backward: shape mismatch");
    let n = pred.numel() as f32;
    let data: Vec<f32> = pred.data().iter()
        .zip(target.data().iter())
        .map(|(&p, &t)| 2.0 * (p - t) / n)
        .collect();
    DynTensor::from_vec(pred.shape().to_vec(), data)
}

// ------------------------------------------------------------------
// CrossEntropy — negative log-likelihood on softmax probabilities
// Expects `probs` = softmax output [batch, classes]
//         `labels` = one-hot targets [batch, classes]
// loss = -mean(sum_c(label_c * log(prob_c)))
// ------------------------------------------------------------------

/// Compute cross-entropy loss.
/// `probs`:  softmax probabilities [batch, classes]
/// `labels`: one-hot targets       [batch, classes]
pub fn cross_entropy(probs: &DynTensor<f32>, labels: &DynTensor<f32>) -> f32 {
    assert_eq!(probs.shape(), labels.shape(), "cross_entropy: shape mismatch");
    assert_eq!(probs.shape().len(), 2, "cross_entropy: expects rank-2 input");
    let batch = probs.shape()[0];
    let c     = probs.shape()[1];
    let mut total = 0.0f32;
    for b in 0..batch {
        for j in 0..c {
            let p = probs.get(&[b, j]).max(1e-7); // clamp for log stability
            total += labels.get(&[b, j]) * libm::logf(p);
        }
    }
    -total / batch as f32
}

/// CrossEntropy backward w.r.t. pre-softmax logits (combined softmax+CE grad).
/// Combined gradient: grad_logit[b,j] = prob[b,j] - label[b,j]
/// This is the clean form when softmax and cross-entropy are fused.
pub fn cross_entropy_backward(
    probs: &DynTensor<f32>,
    labels: &DynTensor<f32>,
) -> DynTensor<f32> {
    assert_eq!(probs.shape(), labels.shape());
    let batch = probs.shape()[0];
    let c     = probs.shape()[1];
    let mut out = DynTensor::zeros(alloc::vec![batch, c]);
    for b in 0..batch {
        for j in 0..c {
            let g = (probs.get(&[b, j]) - labels.get(&[b, j])) / batch as f32;
            out.set(&[b, j], g);
        }
    }
    out
}

// ------------------------------------------------------------------
// Accuracy helper — argmax comparison
// ------------------------------------------------------------------

/// Compute classification accuracy.
/// `probs`:  [batch, classes] — model output (softmax or logits)
/// `labels`: [batch, classes] — one-hot targets
/// Returns fraction of correct predictions in [0.0, 1.0].
pub fn accuracy(probs: &DynTensor<f32>, labels: &DynTensor<f32>) -> f32 {
    assert_eq!(probs.shape(), labels.shape());
    let batch = probs.shape()[0];
    let c     = probs.shape()[1];
    let mut correct = 0usize;
    for b in 0..batch {
        // argmax of probs
        let mut pred_class = 0usize;
        let mut pred_max = probs.get(&[b, 0]);
        for j in 1..c {
            let v = probs.get(&[b, j]);
            if v > pred_max { pred_max = v; pred_class = j; }
        }
        // argmax of labels
        let mut true_class = 0usize;
        let mut true_max = labels.get(&[b, 0]);
        for j in 1..c {
            let v = labels.get(&[b, j]);
            if v > true_max { true_max = v; true_class = j; }
        }
        if pred_class == true_class { correct += 1; }
    }
    correct as f32 / batch as f32
}
