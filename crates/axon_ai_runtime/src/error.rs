// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
// axon_ai_runtime error types.
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum AiError {
    ShapeMismatch { expected: Vec<usize>, got: Vec<usize> },
    InvalidShape(Vec<usize>),
    EmptyTensor,
    DivisionByZero,
    GraphCycle(String),
    NodeNotFound(String),
    InferenceFailed(String),
    ModelNotLoaded,
    UnsupportedOp(String),
}

impl fmt::Display for AiError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AiError::ShapeMismatch { expected, got } =>
                write!(f, "shape mismatch: expected {:?}, got {:?}", expected, got),
            AiError::InvalidShape(s)      => write!(f, "invalid shape: {:?}", s),
            AiError::EmptyTensor          => write!(f, "empty tensor"),
            AiError::DivisionByZero       => write!(f, "division by zero"),
            AiError::GraphCycle(s)        => write!(f, "graph cycle: {}", s),
            AiError::NodeNotFound(s)      => write!(f, "node not found: {}", s),
            AiError::InferenceFailed(s)   => write!(f, "inference failed: {}", s),
            AiError::ModelNotLoaded       => write!(f, "model not loaded"),
            AiError::UnsupportedOp(s)     => write!(f, "unsupported op: {}", s),
        }
    }
}

pub type AiResult<T> = Result<T, AiError>;
