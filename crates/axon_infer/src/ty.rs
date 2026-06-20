// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
//
// AXONYX concrete type definitions — source of truth for P52 inference.
// Mirrored in axon/ty.ax.
// P55.5: AxString -> String throughout; HirTy sovereign variants handled.

use axon_hir::HirTy;

pub type TyVar = usize;

#[derive(Debug, Clone, PartialEq)]
pub enum Ty {
    I32, I64, F64, Bool, Str, Nil,
    Named(String),
    Var(TyVar),
    Unknown,
    // P55.5: sovereign type wrappers preserved through inference
    Tainted(Box<Ty>),
    Clean(Box<Ty>),
    Secret(Box<Ty>),
    Auditable(Box<Ty>),
    Money { currency: String, precision: i64 },
    SafeInt { lo: i64, hi: i64 },
}

/// Convert a HirTy to a concrete Ty.
pub fn ty_from_hir(hir_ty: &HirTy) -> Ty {
    match hir_ty {
        HirTy::Named(n)    => ty_from_name(n),
        HirTy::Infer       => Ty::Unknown,
        // P55.5: sovereign wrappers — inner type resolved recursively
        HirTy::Tainted(i)  => Ty::Tainted(Box::new(ty_from_hir(i))),
        HirTy::Clean(i)    => Ty::Clean(Box::new(ty_from_hir(i))),
        HirTy::Secret(i)   => Ty::Secret(Box::new(ty_from_hir(i))),
        HirTy::Auditable(i)=> Ty::Auditable(Box::new(ty_from_hir(i))),
        HirTy::Money { currency, precision } =>
            Ty::Money { currency: currency.clone(), precision: *precision },
        HirTy::SafeInt { lo, hi } => Ty::SafeInt { lo: *lo, hi: *hi },
        // All other sovereign variants resolve to Named for now — P55.6 refines
        HirTy::Expires { inner, .. }       => ty_from_hir(inner),
        HirTy::Resident { inner, .. }      => ty_from_hir(inner),
        HirTy::Refinement { base, .. }     => ty_from_hir(base),
        HirTy::Opaque { name, .. }         => Ty::Named(name.clone()),
    }
}

/// Map a type name string to a primitive Ty.
pub fn ty_from_name(name: &str) -> Ty {
    match name {
        "i32"  => Ty::I32,
        "i64"  => Ty::I64,
        "f64"  => Ty::F64,
        "bool" => Ty::Bool,
        "str"  => Ty::Str,
        "nil"  => Ty::Nil,
        other  => Ty::Named(other.to_string()),
    }
}
