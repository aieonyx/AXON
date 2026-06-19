// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0

use axon_std_string::AxString;

#[derive(Debug, PartialEq)]
pub enum CodegenError {
    UnknownType,
    UndefinedName(AxString),
    PipelineError(AxString),
}

impl core::fmt::Display for CodegenError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            CodegenError::UnknownType       => write!(f, "codegen: unknown type — inference incomplete"),
            CodegenError::UndefinedName(n)  => write!(f, "codegen: undefined name: {}", n.as_str()),
            CodegenError::PipelineError(m)  => write!(f, "codegen: pipeline error: {}", m.as_str()),
        }
    }
}

pub type CodegenResult<T> = Result<T, CodegenError>;
