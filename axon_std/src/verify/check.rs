#![allow(clippy::all, dead_code, unused_imports)]
//! Runtime postcondition enforcement — check_postcondition() and ensures!().
//!
//! Bio DNA: Annotation Preservation — @ensures annotations survive the
//! transpiler and become runtime checks via this module.

use super::cache::CONTRACT_CACHE;
use crate::ai::weight::InferenceWeight;

/// Result type for all axon::verify operations.
pub type VerifyResult<T> = Result<T, VerificationError>;

/// A postcondition violation at runtime.
#[derive(Debug, Clone)]
pub struct VerificationError {
    /// The @ensures label that was violated.
    pub label:       &'static str,
    /// Human-readable description of the violation.
    pub description: String,
    /// The function hash at time of violation.
    pub fn_hash:     u64,
}

impl std::fmt::Display for VerificationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f,
            "error[E411]: @ensures violated\n  → label: {}\n  → {}\n  → fn_hash: {:#018x}",
            self.label, self.description, self.fn_hash
        )
    }
}

impl std::error::Error for VerificationError {}

/// Check a runtime postcondition from an `@ensures` annotation.
///
/// Returns `Ok(())` if the condition holds, `Err(VerificationError)` otherwise.
/// Results are stored in the [`ContractCache`][super::cache::ContractCache]
/// for replay without re-evaluation.
///
/// # Examples
///
/// ```rust
/// use axon_std::verify::check_postcondition;
///
/// fn safe_div(a: i64, b: i64) -> i64 {
///     assert!(b != 0);
///     let result = a / b;
///     check_postcondition("result_bounded", result.abs() <= a.abs()).unwrap();
///     result
/// }
/// assert_eq!(safe_div(10, 2), 5);
/// ```
pub fn check_postcondition(label: &'static str, ok: bool) -> VerifyResult<()> {
    // Compute a simple hash for cache keying
    let fn_hash = fnv64(label.as_bytes());

    // Update cache
    CONTRACT_CACHE.with(|cache| {
        cache.borrow_mut().record(fn_hash, ok);
    });

    if ok {
        Ok(())
    } else {
        Err(VerificationError {
            label,
            description: format!("postcondition '{label}' evaluated to false"),
            fn_hash,
        })
    }
}

/// Check a postcondition with an explicit function hash (for cache keying).
///
/// Use this when the function identity needs to be tied to its source hash
/// rather than just its label — prevents cache collisions across refactors.
pub fn check_postcondition_hashed(
    label:   &'static str,
    ok:      bool,
    fn_hash: u64,
) -> VerifyResult<()> {
    CONTRACT_CACHE.with(|cache| {
        cache.borrow_mut().record(fn_hash, ok);
    });

    if ok {
        Ok(())
    } else {
        Err(VerificationError {
            label,
            description: format!("postcondition '{label}' violated (fn_hash={fn_hash:#018x})"),
            fn_hash,
        })
    }
}

/// FNV-64 hash — same algorithm as axon_core::FnvHasher, inlined here
/// to avoid a dependency cycle (axon_core → axon_std would be circular).
pub(crate) fn fnv64(data: &[u8]) -> u64 {
    let mut h: u64 = 14_695_981_039_346_656_037;
    for &byte in data {
        h ^= byte as u64;
        h = h.wrapping_mul(1_099_511_628_211);
    }
    h
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_ok_returns_unit() {
        assert!(check_postcondition("always_true", true).is_ok());
    }

    #[test]
    fn check_err_returns_violation() {
        let r = check_postcondition("always_false", false);
        assert!(r.is_err());
        let e = r.unwrap_err();
        assert_eq!(e.label, "always_false");
        assert!(e.description.contains("always_false"));
    }

    #[test]
    fn check_hashed_ok() {
        assert!(check_postcondition_hashed("label", true, 0xDEAD_BEEF).is_ok());
    }

    #[test]
    fn check_hashed_err() {
        let r = check_postcondition_hashed("label", false, 0xDEAD_BEEF);
        assert!(r.is_err());
        assert!(r.unwrap_err().description.contains("dead"));
    }

    #[test]
    fn fnv64_deterministic() {
        assert_eq!(fnv64(b"axon"), fnv64(b"axon"));
        assert_ne!(fnv64(b"axon"), fnv64(b"AXON"));
    }

    #[test]
    fn verification_error_display() {
        let e = VerificationError {
            label: "test_label",
            description: "test desc".into(),
            fn_hash: 0xABCD,
        };
        let s = format!("{e}");
        assert!(s.contains("E411"));
        assert!(s.contains("test_label"));
    }
}
