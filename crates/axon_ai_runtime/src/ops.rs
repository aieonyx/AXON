// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
// Tensor operations -- sovereign AI math.
// Clean-room: studied deep learning mathematics from academic papers only.
// P63.0: matmul, relu, sigmoid, softmax, layer_norm, add_bias.
use crate::tensor::Tensor;
use crate::error::{AiError, AiResult};

/// Matrix multiplication: (M,K) x (K,N) -> (M,N)
pub fn matmul(a: &Tensor, b: &Tensor) -> AiResult<Tensor> {
    if a.ndim() != 2 || b.ndim() != 2 {
        return Err(AiError::InvalidShape(a.shape.clone()));
    }
    let (m, k1) = (a.shape[0], a.shape[1]);
    let (k2, n) = (b.shape[0], b.shape[1]);
    if k1 != k2 {
        return Err(AiError::ShapeMismatch {
            expected: vec![m, k1], got: vec![k2, n]
        });
    }
    let mut data = vec![0.0f32; m * n];
    for i in 0..m {
        for j in 0..n {
            let mut sum = 0.0f32;
            for k in 0..k1 {
                sum += a.data[i * k1 + k] * b.data[k * n + j];
            }
            data[i * n + j] = sum;
        }
    }
    Tensor::new(vec![m, n], data)
}

/// Element-wise ReLU: max(0, x)
pub fn relu(t: &Tensor) -> Tensor {
    t.map(|x| x.max(0.0))
}

/// Element-wise Sigmoid: 1 / (1 + exp(-x))
pub fn sigmoid(t: &Tensor) -> Tensor {
    t.map(|x| 1.0 / (1.0 + (-x).exp()))
}

/// Element-wise Tanh
pub fn tanh(t: &Tensor) -> Tensor {
    t.map(|x| x.tanh())
}

/// Softmax over last dimension (1D tensor)
pub fn softmax(t: &Tensor) -> AiResult<Tensor> {
    if t.ndim() != 1 {
        return Err(AiError::InvalidShape(t.shape.clone()));
    }
    let max_val = t.max();
    let exp_data: Vec<f32> = t.data.iter().map(|&x| (x - max_val).exp()).collect();
    let sum: f32 = exp_data.iter().sum();
    if sum == 0.0 { return Err(AiError::DivisionByZero); }
    let data = exp_data.iter().map(|&x| x / sum).collect();
    Tensor::new(t.shape.clone(), data)
}

/// Add bias vector to each row of a matrix.
/// a: (M, N), bias: (N,) -> (M, N)
pub fn add_bias(a: &Tensor, bias: &Tensor) -> AiResult<Tensor> {
    if a.ndim() != 2 || bias.ndim() != 1 {
        return Err(AiError::InvalidShape(a.shape.clone()));
    }
    let (m, n) = (a.shape[0], a.shape[1]);
    if bias.shape[0] != n {
        return Err(AiError::ShapeMismatch {
            expected: vec![n], got: bias.shape.clone()
        });
    }
    let mut data = a.data.clone();
    for i in 0..m {
        for j in 0..n {
            data[i * n + j] += bias.data[j];
        }
    }
    Tensor::new(vec![m, n], data)
}

/// Layer normalization over last dimension.
/// Normalizes each row of a 2D tensor to mean=0, std=1.
pub fn layer_norm(t: &Tensor, eps: f32) -> AiResult<Tensor> {
    if t.ndim() != 2 {
        return Err(AiError::InvalidShape(t.shape.clone()));
    }
    let (m, n) = (t.shape[0], t.shape[1]);
    let mut data = t.data.clone();
    for i in 0..m {
        let row = &t.data[i*n..(i+1)*n];
        let mean: f32 = row.iter().sum::<f32>() / n as f32;
        let var: f32  = row.iter().map(|&x| (x - mean).powi(2)).sum::<f32>() / n as f32;
        let std        = (var + eps).sqrt();
        for j in 0..n {
            data[i*n+j] = (t.data[i*n+j] - mean) / std;
        }
    }
    Tensor::new(vec![m, n], data)
}

/// Dot product of two 1D tensors.
pub fn dot(a: &Tensor, b: &Tensor) -> AiResult<f32> {
    if a.ndim() != 1 || b.ndim() != 1 || a.shape != b.shape {
        return Err(AiError::ShapeMismatch {
            expected: a.shape.clone(), got: b.shape.clone()
        });
    }
    Ok(a.data.iter().zip(b.data.iter()).map(|(&x,&y)| x*y).sum())
}

/// Cross-entropy loss between predictions (softmax output) and one-hot target.
pub fn cross_entropy(pred: &Tensor, target_idx: usize) -> AiResult<f32> {
    if pred.ndim() != 1 { return Err(AiError::InvalidShape(pred.shape.clone())); }
    if target_idx >= pred.numel() {
        return Err(AiError::ShapeMismatch {
            expected: pred.shape.clone(), got: vec![target_idx]
        });
    }
    let p = pred.data[target_idx].max(1e-7);
    Ok(-p.ln())
}
