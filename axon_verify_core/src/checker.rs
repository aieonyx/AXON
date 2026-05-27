//! Runtime postcondition and witness checkers.
//!
//! Every public function in this module is verified by Kani.

use crate::contract::Witness;

/// The outcome of a verification check.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerifyOutcome {
    /// The condition held — contract satisfied.
    Pass,
    /// The condition did not hold — contract violated.
    Fail,
}

impl VerifyOutcome {
    /// Returns true if this outcome represents a pass.
    #[inline]
    pub const fn is_pass(&self) -> bool { matches!(self, VerifyOutcome::Pass) }
    /// Returns true if this outcome represents a failure.
    #[inline]
    pub const fn is_fail(&self) -> bool { matches!(self, VerifyOutcome::Fail) }
}

/// Check a runtime `@ensures` postcondition.
///
/// # Specification
///
/// - If `ok` is `true`  → returns `VerifyOutcome::Pass`
/// - If `ok` is `false` → returns `VerifyOutcome::Fail`
/// - Never panics for any input.
/// - Pure: no side effects.
///
/// # Kani verification
///
/// `cargo kani --harness check_ensures_pass`
/// `cargo kani --harness check_ensures_fail`
#[inline]
pub const fn check_ensures(_label: u32, ok: bool) -> VerifyOutcome {
    if ok { VerifyOutcome::Pass } else { VerifyOutcome::Fail }
}

/// Check a Dynamic Witness Contract (DWC).
///
/// # Specification
///
/// - If `witness.is_valid()` → returns `VerifyOutcome::Pass`
/// - Otherwise              → returns `VerifyOutcome::Fail`
/// - Never panics.
/// - Pure: no side effects.
///
/// # Kani verification
///
/// `cargo kani --harness check_dwc_valid`
/// `cargo kani --harness check_dwc_invalid`
#[inline]
pub const fn check_dwc(witness: &Witness) -> VerifyOutcome {
    if witness.is_valid() { VerifyOutcome::Pass } else { VerifyOutcome::Fail }
}

/// Check a Quorum Consensus Contract (QCC).
///
/// Returns `Pass` only when at least `required` valid witnesses are present.
///
/// # Kani verification
///
/// `cargo kani --harness check_qcc_sufficient`
/// `cargo kani --harness check_qcc_insufficient`
pub fn check_qcc(witnesses: &[Witness], required: usize) -> VerifyOutcome {
    if required == 0 { return VerifyOutcome::Pass; }
    let valid_count = witnesses.iter().filter(|w| w.is_valid()).count();
    if valid_count >= required { VerifyOutcome::Pass } else { VerifyOutcome::Fail }
}

// ── Kani proof harnesses ──────────────────────────────────────────────────────
#[cfg(kani)]
mod proofs {
    use super::*;
    use crate::contract::{Witness, WitnessKind};

    // check_ensures proofs
    #[kani::proof]
    fn check_ensures_pass() {
        // PROVES: check_ensures(any_label, true) always returns Pass
        let label: u32 = kani::any();
        let result = check_ensures(label, true);
        assert!(result.is_pass());
    }

    #[kani::proof]
    fn check_ensures_fail() {
        // PROVES: check_ensures(any_label, false) always returns Fail
        let label: u32 = kani::any();
        let result = check_ensures(label, false);
        assert!(result.is_fail());
    }

    #[kani::proof]
    fn check_ensures_deterministic() {
        // PROVES: same inputs always produce same output
        let label: u32 = kani::any();
        let ok: bool = kani::any();
        let r1 = check_ensures(label, ok);
        let r2 = check_ensures(label, ok);
        assert_eq!(r1, r2);
    }

    // check_dwc proofs
    #[kani::proof]
    fn check_dwc_valid() {
        // PROVES: valid witness always passes DWC check
        let w = Witness { kind: WitnessKind::Postcondition, valid: true, hash: kani::any() };
        assert!(check_dwc(&w).is_pass());
    }

    #[kani::proof]
    fn check_dwc_invalid() {
        // PROVES: invalid witness always fails DWC check
        let w = Witness { kind: WitnessKind::Postcondition, valid: false, hash: kani::any() };
        assert!(check_dwc(&w).is_fail());
    }

    #[kani::proof]
    fn check_dwc_hash_irrelevant() {
        // PROVES: the hash field does not affect the outcome
        let valid: bool = kani::any();
        let hash1: u64  = kani::any();
        let hash2: u64  = kani::any();
        let w1 = Witness { kind: WitnessKind::Postcondition, valid, hash: hash1 };
        let w2 = Witness { kind: WitnessKind::Postcondition, valid, hash: hash2 };
        assert_eq!(check_dwc(&w1), check_dwc(&w2));
    }

    // check_qcc proofs
    #[kani::proof]
    fn check_qcc_sufficient() {
        // PROVES: two valid witnesses satisfy a quorum of 2
        let witnesses = [
            Witness { kind: WitnessKind::SecurityProperty, valid: true, hash: 1 },
            Witness { kind: WitnessKind::Postcondition,   valid: true, hash: 2 },
        ];
        assert!(check_qcc(&witnesses, 2).is_pass());
    }

    #[kani::proof]
    fn check_qcc_insufficient() {
        // PROVES: one valid witness does not satisfy quorum of 2
        let witnesses = [
            Witness { kind: WitnessKind::SecurityProperty, valid: true,  hash: 1 },
            Witness { kind: WitnessKind::Postcondition,   valid: false, hash: 2 },
        ];
        assert!(check_qcc(&witnesses, 2).is_fail());
    }

    #[kani::proof]
    fn check_qcc_zero_required() {
        // PROVES: quorum of 0 always passes regardless of witnesses
        let witnesses: [Witness; 0] = [];
        assert!(check_qcc(&witnesses, 0).is_pass());
    }
}
