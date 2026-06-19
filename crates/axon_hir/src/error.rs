// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0

use axon_std_string::AxString;

#[derive(Debug, PartialEq)]
pub enum HirError {
    UndefinedName(AxString),
    LoweringError(AxString),
}

impl core::fmt::Display for HirError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            HirError::UndefinedName(n) => write!(f, "hir: undefined name: {}", n.as_str()),
            HirError::LoweringError(m) => write!(f, "hir: lowering error: {}", m.as_str()),
        }
    }
}

pub type HirResult<T> = Result<T, HirError>;
