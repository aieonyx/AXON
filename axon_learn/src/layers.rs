// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
//
// axon_learn::layers — Neural Network Layers
// Linear, ReLU, Softmax, GELU
// Operate on DynTensor<f32>. Weights stored as DynTensor<f32>.

use alloc::vec::Vec;
use axon_tensor::DynTensor;
use axon_tensor::ops::TensorOps;

// ------------------------------------------------------------------
// Linear layer: y = xW^T + b
// Input:  [batch, in_features]
// Weight: [out_features, in_features]
// Bias:   [out_features]
// Output: [batch, out_features]
// ------------------------------------------------------------------

/// A fully-connected linear layer with weights and bias.
pub struct Linear {
    pub weight: DynTensor<f32>,   // [out, in]
    pub bias:   DynTensor<f32>,   // [out]
    pub in_features:  usize,
    pub out_features: usize,
}

impl Linear {
    /// Create a Linear layer with zero-initialized weights and bias.
    pub fn new(in_features: usize, out_features: usize) -> Self {
        Self {
            weight: DynTensor::zeros(alloc::vec![out_features, in_features]),
            bias:   DynTensor::zeros(alloc::vec![out_features]),
            in_features,
            out_features,
        }
    }

    /// Initialize weights from a flat Vec (row-major, [out, in]).
    pub fn with_weights(mut self, data: Vec<f32>) -> Self {
        assert_eq!(data.len(), self.out_features * self.in_features,
            "Linear::with_weights: data length mismatch");
        self.weight = DynTensor::from_vec(
            alloc::vec![self.out_features, self.in_features], data);
        self
    }

    /// Initialize bias from a Vec of length out_features.
    pub fn with_bias(mut self, data: Vec<f32>) -> Self {
        assert_eq!(data.len(), self.out_features,
            "Linear::with_bias: data length mismatch");
        self.bias = DynTensor::from_vec(alloc::vec![self.out_features], data);
        self
    }

    /// Forward pass: output = input @ weight^T + bias
    /// Input shape: [batch, in_features]
    /// Output shape: [batch, out_features]
    pub fn forward(&self, input: &DynTensor<f32>) -> DynTensor<f32> {
        let batch = input.shape()[0];
        assert_eq!(input.shape()[1], self.in_features,
            "Linear::forward: input feature dim mismatch");

        let mut out = DynTensor::zeros(alloc::vec![batch, self.out_features]);

        for b in 0..batch {
            for o in 0..self.out_features {
                let mut acc = 0.0f32;
                for i in 0..self.in_features {
                    acc += input.get(&[b, i]) * self.weight.get(&[o, i]);
                }
                acc += self.bias.get(&[o]);
                out.set(&[b, o], acc);
            }
        }
        out
    }

    /// Backward pass: compute gradients w.r.t. input, weight, bias.
    /// Returns (grad_input, grad_weight, grad_bias).
    /// grad_output shape: [batch, out_features]
    pub fn backward(
        &self,
        input: &DynTensor<f32>,
        grad_output: &DynTensor<f32>,
    ) -> (DynTensor<f32>, DynTensor<f32>, DynTensor<f32>) {
        let batch = input.shape()[0];

        // grad_input = grad_output @ weight  [batch, in]
        let mut grad_input = DynTensor::zeros(
            alloc::vec![batch, self.in_features]);
        for b in 0..batch {
            for i in 0..self.in_features {
                let mut acc = 0.0f32;
                for o in 0..self.out_features {
                    acc += grad_output.get(&[b, o]) * self.weight.get(&[o, i]);
                }
                grad_input.set(&[b, i], acc);
            }
        }

        // grad_weight = grad_output^T @ input  [out, in]
        let mut grad_weight = DynTensor::zeros(
            alloc::vec![self.out_features, self.in_features]);
        for o in 0..self.out_features {
            for i in 0..self.in_features {
                let mut acc = 0.0f32;
                for b in 0..batch {
                    acc += grad_output.get(&[b, o]) * input.get(&[b, i]);
                }
                grad_weight.set(&[o, i], acc);
            }
        }

        // grad_bias = sum over batch  [out]
        let mut grad_bias = DynTensor::zeros(alloc::vec![self.out_features]);
        for o in 0..self.out_features {
            let mut acc = 0.0f32;
            for b in 0..batch {
                acc += grad_output.get(&[b, o]);
            }
            grad_bias.set(&[o], acc);
        }

        (grad_input, grad_weight, grad_bias)
    }
}

// ------------------------------------------------------------------
// ReLU activation: element-wise max(0, x)
// ------------------------------------------------------------------

/// Apply ReLU element-wise to a DynTensor<f32>.
pub fn relu(x: &DynTensor<f32>) -> DynTensor<f32> {
    let data: Vec<f32> = x.data().iter()
        .map(|&v| if v > 0.0 { v } else { 0.0 })
        .collect();
    DynTensor::from_vec(x.shape().to_vec(), data)
}

/// ReLU backward: gradient is 1 where x > 0, else 0.
pub fn relu_backward(
    x: &DynTensor<f32>,
    grad_output: &DynTensor<f32>,
) -> DynTensor<f32> {
    let data: Vec<f32> = x.data().iter()
        .zip(grad_output.data().iter())
        .map(|(&v, &g)| if v > 0.0 { g } else { 0.0 })
        .collect();
    DynTensor::from_vec(x.shape().to_vec(), data)
}

// ------------------------------------------------------------------
// Softmax: exp(x_i) / sum(exp(x_j)) — numerically stable
// Operates on the last dimension of a [batch, classes] tensor.
// ------------------------------------------------------------------

/// Apply softmax along axis=1 (last dim) to a [batch, C] tensor.
pub fn softmax(x: &DynTensor<f32>) -> DynTensor<f32> {
    assert_eq!(x.shape().len(), 2, "softmax expects rank-2 input");
    let batch = x.shape()[0];
    let c     = x.shape()[1];
    let mut out = DynTensor::zeros(alloc::vec![batch, c]);

    for b in 0..batch {
        // Numerically stable: subtract max before exp
        let mut max_val = x.get(&[b, 0]);
        for j in 1..c {
            let v = x.get(&[b, j]);
            if v > max_val { max_val = v; }
        }
        let mut sum = 0.0f32;
        for j in 0..c {
            let e = libm::expf(x.get(&[b, j]) - max_val);
            out.set(&[b, j], e);
            sum += e;
        }
        for j in 0..c {
            out.set(&[b, j], out.get(&[b, j]) / sum);
        }
    }
    out
}

/// Softmax backward: Jacobian-vector product.
/// grad_input[i] = p[i] * (grad_output[i] - sum_j(p[j]*grad_output[j]))
pub fn softmax_backward(
    probs: &DynTensor<f32>,
    grad_output: &DynTensor<f32>,
) -> DynTensor<f32> {
    assert_eq!(probs.shape().len(), 2);
    let batch = probs.shape()[0];
    let c     = probs.shape()[1];
    let mut out = DynTensor::zeros(alloc::vec![batch, c]);

    for b in 0..batch {
        let dot: f32 = (0..c)
            .map(|j| probs.get(&[b, j]) * grad_output.get(&[b, j]))
            .sum();
        for j in 0..c {
            let g = probs.get(&[b, j]) * (grad_output.get(&[b, j]) - dot);
            out.set(&[b, j], g);
        }
    }
    out
}

// ------------------------------------------------------------------
// GELU activation: x * Φ(x) — Gaussian Error Linear Unit
// Approximation: 0.5 * x * (1 + tanh(√(2/π) * (x + 0.044715x³)))
// ------------------------------------------------------------------

/// Apply GELU element-wise to a DynTensor<f32>.
pub fn gelu(x: &DynTensor<f32>) -> DynTensor<f32> {
    let data: Vec<f32> = x.data().iter().map(|&v| gelu_scalar(v)).collect();
    DynTensor::from_vec(x.shape().to_vec(), data)
}

#[inline]
fn gelu_scalar(x: f32) -> f32 {
    // GELU approximation used by BERT/GPT
    const SQRT_2_OVER_PI: f32 = 0.797_884_6;
    const COEFF: f32 = 0.044715;
    let inner = SQRT_2_OVER_PI * (x + COEFF * x * x * x);
    0.5 * x * (1.0 + libm::tanhf(inner))
}

/// GELU backward: derivative w.r.t. x.
pub fn gelu_backward(
    x: &DynTensor<f32>,
    grad_output: &DynTensor<f32>,
) -> DynTensor<f32> {
    let data: Vec<f32> = x.data().iter()
        .zip(grad_output.data().iter())
        .map(|(&v, &g)| g * gelu_grad_scalar(v))
        .collect();
    DynTensor::from_vec(x.shape().to_vec(), data)
}

#[inline]
fn gelu_grad_scalar(x: f32) -> f32 {
    const SQRT_2_OVER_PI: f32 = 0.797_884_6;
    const COEFF: f32 = 0.044715;
    let inner = SQRT_2_OVER_PI * (x + COEFF * x * x * x);
    let tanh_val = libm::tanhf(inner);
    let sech2 = 1.0 - tanh_val * tanh_val;
    let d_inner = SQRT_2_OVER_PI * (1.0 + 3.0 * COEFF * x * x);
    0.5 * (1.0 + tanh_val) + 0.5 * x * sech2 * d_inner
}
