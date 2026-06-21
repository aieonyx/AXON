// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
// axon_ai_runtime -- HANIEL-ONYX sovereign AI inference runtime.
// P63.0: tensor ops, computation graph, feedforward model inference.
// P63.1: transformer attention, embedding, KV cache, WASM model loading.
pub mod error;
pub mod graph;
pub mod model;
pub mod ops;
pub mod tensor;
pub use error::{AiError, AiResult};
pub use graph::{ComputeGraph, GraphNode, NodeOp};
pub use model::{SovereignModel, DenseLayer, Activation};
pub use ops::{matmul, relu, sigmoid, tanh, softmax, add_bias, layer_norm, dot, cross_entropy};
pub use tensor::Tensor;

pub mod attention_ffi;
