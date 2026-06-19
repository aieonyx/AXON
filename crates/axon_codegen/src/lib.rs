// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
//
// axon_codegen — LLVM IR emission for AXONYX.
// Internal codegen layer. Not ARPi-exposed.
// External exposure via ARPi boundary defined at AWP layer.
//
// P53 landmark: AXON emits its own LLVM IR.
// Bridge (emit.rs) mirrors emit.ax exactly — excised at P55.

pub mod emit;
pub mod error;
pub mod ir;

pub use emit::{codegen_source, emit_module};
pub use error::{CodegenError, CodegenResult};
pub use ir::{ty_str, ty_to_ir, IrTy};
