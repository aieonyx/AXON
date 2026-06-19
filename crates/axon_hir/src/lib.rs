// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
//
// axon_hir — HIR lowering for AXONYX.
// Internal HIR layer. Not ARPi-exposed.
// External exposure via ARPi boundary defined at AWP layer.
//
// P51 landmark: AXON lowers its own AST.
// Bridge (lower.rs) mirrors lower.ax exactly — excised at P55.

pub mod error;
pub mod hir;
pub mod lower;

pub use error::{HirError, HirResult};
pub use hir::{
    HirBinOp, HirExpr, HirField, HirFn, HirId,
    HirParam, HirProgram, HirStmt, HirStruct, HirTy, HirUnaryOp,
};
pub use lower::{lower_program, lower_source};
