// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
// axon_ct_check -- P55.7 @constant_time static analysis pass.
// Walks the HIR of @constant_time functions and flags violations:
//   - Branching (if/match) on Secret<T> typed expressions
//   - Early return conditioned on a secret value
//   - Variable-time operations on secret data (future: array indexing)
// This is a structural check, not a full IFC type system.
// P55.7 M2: HIR-level violation detection.

pub mod error;
pub mod check;

pub use error::{CtError, CtViolation, CtResult};
pub use check::check_program;
