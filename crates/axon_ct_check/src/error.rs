// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
// error.rs -- CtViolation and CtError types for P55.7 analysis pass.

#[derive(Debug, Clone, PartialEq)]
pub enum CtViolation {
    /// An if/match expression branches on a Secret<T> value.
    /// field: the function name where the violation occurs.
    SecretBranch { fn_name: String, detail: String },
    /// A return statement is conditioned on a secret value.
    SecretEarlyReturn { fn_name: String },
    /// A parameter of type Secret<T> is used in a branch condition.
    SecretParamBranch { fn_name: String, param: String },
}

impl CtViolation {
    pub fn description(&self) -> String {
        match self {
            CtViolation::SecretBranch { fn_name, detail } =>
                format!("@constant_time violation in '{}': branch on Secret<T> value — {}", fn_name, detail),
            CtViolation::SecretEarlyReturn { fn_name } =>
                format!("@constant_time violation in '{}': early return conditioned on secret", fn_name),
            CtViolation::SecretParamBranch { fn_name, param } =>
                format!("@constant_time violation in '{}': branch on Secret<T> parameter '{}'", fn_name, param),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum CtError {
    Violations(Vec<CtViolation>),
}

pub type CtResult<T> = Result<T, CtError>;
