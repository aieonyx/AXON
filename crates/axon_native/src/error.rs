// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0

use axon_std_string::AxString;

#[derive(Debug, PartialEq)]
pub enum NativeError {
    UndefinedName(AxString),
    UnsupportedExpr(AxString),
    PipelineError(AxString),
}

impl core::fmt::Display for NativeError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            NativeError::UndefinedName(n)   => write!(f, "native: undefined: {}", n.as_str()),
            NativeError::UnsupportedExpr(m) => write!(f, "native: unsupported: {}", m.as_str()),
            NativeError::PipelineError(m)   => write!(f, "native: pipeline: {}", m.as_str()),
        }
    }
}

pub type NativeResult<T> = Result<T, NativeError>;
