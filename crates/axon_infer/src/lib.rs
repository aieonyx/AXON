// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
//
// axon_infer — Type inference for AXONYX.
// Internal inference layer. Not ARPi-exposed.
// External exposure via ARPi boundary defined at AWP layer.
//
// P52 landmark: AXON infers its own types.
// Bridge (infer.rs) mirrors constraint.ax + unify.ax — excised at P55.

pub mod constraint;
pub mod error;
pub mod infer;
pub mod ty;
pub mod unify;

pub use error::{InferError, InferResult};
pub use infer::{infer_program, infer_source, InferredFn, InferredProgram};
pub use ty::{ty_from_hir, ty_from_name, Ty, TyVar};
pub use unify::unify_types;
