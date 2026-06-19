// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0

use axon_std_string::AxString;

#[derive(Debug, PartialEq)]
pub enum InferError {
    TypeMismatch(AxString),
    UndefinedName(AxString),
    PipelineError(AxString),
}

impl core::fmt::Display for InferError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            InferError::TypeMismatch(m)  => write!(f, "infer: type mismatch: {}", m.as_str()),
            InferError::UndefinedName(n) => write!(f, "infer: undefined name: {}", n.as_str()),
            InferError::PipelineError(m) => write!(f, "infer: pipeline error: {}", m.as_str()),
        }
    }
}

pub type InferResult<T> = Result<T, InferError>;
