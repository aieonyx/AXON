// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
//
// Type unification engine — mirrors unify.ax.
// Structural equality with Unknown as wildcard.
// Full union-find with type variables available at P57 (axon_crypto era).

use crate::ty::Ty;
use crate::error::{InferError, InferResult};
use axon_std_string::AxString;

/// Unify two types. Unknown matches anything (wildcard).
/// Returns Ok(()) if compatible, Err(TypeMismatch) if not.
pub fn unify_types(t1: &Ty, t2: &Ty) -> InferResult<()> {
    match (t1, t2) {
        // Unknown is a wildcard — matches anything
        (Ty::Unknown, _) | (_, Ty::Unknown) => Ok(()),
        // Structural equality for primitives
        (Ty::I32,  Ty::I32)  => Ok(()),
        (Ty::I64,  Ty::I64)  => Ok(()),
        (Ty::F64,  Ty::F64)  => Ok(()),
        (Ty::Bool, Ty::Bool) => Ok(()),
        (Ty::Str,  Ty::Str)  => Ok(()),
        (Ty::Nil,  Ty::Nil)  => Ok(()),
        // Named types: match by name
        (Ty::Named(a), Ty::Named(b)) if a == b => Ok(()),
        // Type variables: compatible for now (full solver at P57)
        (Ty::Var(_), _) | (_, Ty::Var(_)) => Ok(()),
        // Mismatch
        _ => Err(InferError::TypeMismatch(
            AxString::ax_from_str(&format!("{:?} is not compatible with {:?}", t1, t2))
        )),
    }
}
