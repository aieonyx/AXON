// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
// Sovereign AI model -- feedforward network with named layers.
// P63.0: dense layers, activation functions, inference pipeline.
// P63.1: transformer attention, embedding tables, KV cache.
use crate::tensor::Tensor;
use crate::ops::{matmul, relu, sigmoid, softmax, add_bias, layer_norm};
use crate::error::{AiError, AiResult};

#[derive(Debug, Clone, PartialEq)]
pub enum Activation {
    ReLU,
    Sigmoid,
    Softmax,
    Tanh,
    None,
}

#[derive(Debug, Clone)]
pub struct DenseLayer {
    pub name:       String,
    pub weights:    Tensor,
    pub bias:       Option<Tensor>,
    pub activation: Activation,
}

impl DenseLayer {
    pub fn new(name: &str, weights: Tensor, bias: Option<Tensor>, activation: Activation) -> AiResult<Self> {
        if weights.ndim() != 2 {
            return Err(AiError::InvalidShape(weights.shape.clone()));
        }
        if let Some(ref b) = bias {
            if b.ndim() != 1 || b.shape[0] != weights.shape[1] {
                return Err(AiError::ShapeMismatch {
                    expected: vec![weights.shape[1]], got: b.shape.clone()
                });
            }
        }
        Ok(DenseLayer { name: name.to_string(), weights, bias, activation })
    }

    pub fn forward(&self, input: &Tensor) -> AiResult<Tensor> {
        // input: (batch, in_features) or (in_features,)
        let x = if input.ndim() == 1 {
            input.reshape(vec![1, input.numel()])?
        } else {
            input.clone()
        };
        let mut out = matmul(&x, &self.weights)?;
        if let Some(ref b) = self.bias {
            out = add_bias(&out, b)?;
        }
        out = match self.activation {
            Activation::ReLU    => relu(&out),
            Activation::Sigmoid => sigmoid(&out),
            Activation::Tanh    => out.map(|x| x.tanh()),
            Activation::Softmax => {
                // softmax per row for 2D
                let (m, n) = (out.shape[0], out.shape[1]);
                let mut data = vec![0.0f32; m * n];
                for i in 0..m {
                    let row = Tensor::from_vec(out.data[i*n..(i+1)*n].to_vec())?;
                    let sm  = softmax(&row)?;
                    data[i*n..(i+1)*n].copy_from_slice(&sm.data);
                }
                Tensor::new(vec![m, n], data)?
            }
            Activation::None => out,
        };
        Ok(out)
    }

    pub fn in_features(&self)  -> usize { self.weights.shape[0] }
    pub fn out_features(&self) -> usize { self.weights.shape[1] }
}

pub struct SovereignModel {
    pub name:   String,
    pub layers: Vec<DenseLayer>,
}

impl SovereignModel {
    pub fn new(name: &str) -> Self {
        SovereignModel { name: name.to_string(), layers: vec![] }
    }

    pub fn add_layer(&mut self, layer: DenseLayer) {
        self.layers.push(layer);
    }

    pub fn layer_count(&self) -> usize { self.layers.len() }

    pub fn infer(&self, input: &Tensor) -> AiResult<Tensor> {
        if self.layers.is_empty() { return Err(AiError::ModelNotLoaded); }
        let mut x = input.clone();
        for layer in &self.layers {
            x = layer.forward(&x)?;
        }
        Ok(x)
    }

    pub fn predict_class(&self, input: &Tensor) -> AiResult<usize> {
        let out = self.infer(input)?;
        let flat = if out.ndim() == 2 {
            Tensor::from_vec(out.data[..out.shape[1]].to_vec())?
        } else {
            out
        };
        let max_idx = flat.data.iter().enumerate()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
            .map(|(i, _)| i)
            .ok_or(AiError::EmptyTensor)?;
        Ok(max_idx)
    }
}
