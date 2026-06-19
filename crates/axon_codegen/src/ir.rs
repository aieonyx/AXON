// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
//
// LLVM IR type definitions — source of truth for P53 codegen.
// Mirrored in axon/ir.ax.
// P54 native codegen replaces LLVM IR with direct machine code.

use axon_infer::Ty;

/// LLVM IR primitive types.
#[derive(Debug, Clone, PartialEq)]
pub enum IrTy {
    I32,
    I64,
    Double,
    I1,
    I8Ptr,
    Void,
}

impl IrTy {
    pub fn as_str(&self) -> &'static str {
        match self {
            IrTy::I32    => "i32",
            IrTy::I64    => "i64",
            IrTy::Double => "double",
            IrTy::I1     => "i1",
            IrTy::I8Ptr  => "i8*",
            IrTy::Void   => "void",
        }
    }
}

/// Map a sovereign Ty to its LLVM IR type.
pub fn ty_to_ir(ty: &Ty) -> IrTy {
    match ty {
        Ty::I32      => IrTy::I32,
        Ty::I64      => IrTy::I64,
        Ty::F64      => IrTy::Double,
        Ty::Bool     => IrTy::I1,
        Ty::Str      => IrTy::I8Ptr,
        Ty::Nil      => IrTy::Void,
        Ty::Named(_) => IrTy::I8Ptr,
        _            => IrTy::I32,   // Unknown/Var: fallback, inference must complete first
    }
}

/// Return the IR type string for a Ty directly.
pub fn ty_str(ty: &Ty) -> &'static str {
    ty_to_ir(ty).as_str()
}
