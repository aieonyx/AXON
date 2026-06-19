// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
//
// AXONYX concrete type definitions — source of truth for P52 inference.
// Mirrored in axon/ty.ax.
// P53 codegen emits LLVM IR types based on these.

use axon_std_string::AxString;
use axon_hir::HirTy;

pub type TyVar = usize;

#[derive(Debug, Clone, PartialEq)]
pub enum Ty {
    // Primitives
    I32, I64, F64, Bool, Str, Nil,
    // User-defined named type
    Named(AxString),
    // Type variable — used during constraint solving
    Var(TyVar),
    // Unknown — unresolved; treated as wildcard during unification
    Unknown,
}

/// Convert a HirTy to a concrete Ty.
pub fn ty_from_hir(hir_ty: &HirTy) -> Ty {
    match hir_ty {
        HirTy::Named(n) => ty_from_name(n),
        HirTy::Infer    => Ty::Unknown,
    }
}

/// Map a type name string to a primitive Ty.
pub fn ty_from_name(name: &AxString) -> Ty {
    match name.as_str() {
        "i32"  => Ty::I32,
        "i64"  => Ty::I64,
        "f64"  => Ty::F64,
        "bool" => Ty::Bool,
        "str"  => Ty::Str,
        "nil"  => Ty::Nil,
        other  => Ty::Named(AxString::ax_from_str(other)),
    }
}
