//! Constitutional invariant enforcement and witness validation.
//!
//! These functions form the innermost ring of the AXON TCB.
//! Every function is verified by Kani — no exceptions.

use crate::contract::{
    BoundaryInvariant, Contract, EnsuresContract, InvariantTier,
};

/// The result of an invariant enforcement check.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnforcementResult {
    /// The operation is permitted.
    Allow,
    /// The operation is blocked — would violate a constitutional invariant.
    Block,
}

impl EnforcementResult {
    #[inline] pub const fn is_allow(&self) -> bool { matches!(self, EnforcementResult::Allow) }
    #[inline] pub const fn is_block(&self) -> bool { matches!(self, EnforcementResult::Block) }
}

/// Enforce an Immutability-by-Inference (IBI) boundary invariant.
///
/// # Specification
///
/// - Constitutional invariants CANNOT be weakened — EVER.
///   `tier == Constitutional && contract.weakens(invariant)` → `Block`
/// - Non-constitutional invariants may be weakened (Operational, Advisory).
///   All other cases → `Allow`
/// - Never panics for any input.
/// - Pure: no side effects.
///
/// # Kani verification
///
/// `cargo kani --harness enforce_ibi_constitutional_block`
/// `cargo kani --harness enforce_ibi_constitutional_allow`
/// `cargo kani --harness enforce_ibi_operational_allows_weakening`
pub const fn enforce_ibi(
    invariant: &BoundaryInvariant,
    contract:  &Contract,
) -> EnforcementResult {
    // Constitutional invariants: block any weakening, unconditionally.
    if matches!(invariant.tier, InvariantTier::Constitutional)
        && contract.invariant_id == invariant.id
        && contract.weakens_invariant
    {
        return EnforcementResult::Block;
    }
    EnforcementResult::Allow
}

/// Validate all witnesses in an EnsuresContract.
///
/// # Specification
///
/// - Contract has no witnesses → `false` (unwitnessed contracts rejected)
/// - All witnesses valid       → `true`
/// - Any witness invalid       → `false`
/// - Never panics for any input.
/// - Pure: no side effects.
///
/// # Kani verification
///
/// `cargo kani --harness validate_witness_all_valid`
/// `cargo kani --harness validate_witness_one_invalid`
/// `cargo kani --harness validate_witness_empty`
pub fn validate_witness(contract: &EnsuresContract) -> bool {
    if !contract.has_witness() { return false; }
    contract.all_witnesses_valid()
}

// ── Kani proof harnesses ──────────────────────────────────────────────────────
#[cfg(kani)]
mod proofs {
    use super::*;
    use crate::contract::{Witness, WitnessKind};

    // enforce_ibi proofs

    #[kani::proof]
    fn enforce_ibi_constitutional_block() {
        // PROVES: Constitutional invariants ALWAYS block weakening
        // Holds for ALL possible invariant IDs
        let id: u32 = kani::any();
        let invariant = BoundaryInvariant { id, tier: InvariantTier::Constitutional };
        let contract  = Contract { invariant_id: id, weakens_invariant: true };
        let result = enforce_ibi(&invariant, &contract);
        assert!(result.is_block());
    }

    #[kani::proof]
    fn enforce_ibi_constitutional_allow_non_weakening() {
        // PROVES: Constitutional invariants ALLOW non-weakening changes
        let id: u32 = kani::any();
        let invariant = BoundaryInvariant { id, tier: InvariantTier::Constitutional };
        let contract  = Contract { invariant_id: id, weakens_invariant: false };
        let result = enforce_ibi(&invariant, &contract);
        assert!(result.is_allow());
    }

    #[kani::proof]
    fn enforce_ibi_operational_allows_weakening() {
        // PROVES: Operational invariants permit weakening
        let id: u32 = kani::any();
        let invariant = BoundaryInvariant { id, tier: InvariantTier::Operational };
        let contract  = Contract { invariant_id: id, weakens_invariant: true };
        let result = enforce_ibi(&invariant, &contract);
        assert!(result.is_allow());
    }

    #[kani::proof]
    fn enforce_ibi_advisory_allows_weakening() {
        // PROVES: Advisory invariants permit weakening
        let id: u32 = kani::any();
        let invariant = BoundaryInvariant { id, tier: InvariantTier::Advisory };
        let contract  = Contract { invariant_id: id, weakens_invariant: true };
        let result = enforce_ibi(&invariant, &contract);
        assert!(result.is_allow());
    }

    #[kani::proof]
    fn enforce_ibi_different_id_allows() {
        // PROVES: Contract targeting a DIFFERENT invariant ID is always allowed
        let inv_id:  u32 = kani::any();
        let con_id:  u32 = kani::any();
        kani::assume(inv_id != con_id); // different IDs
        let invariant = BoundaryInvariant { id: inv_id, tier: InvariantTier::Constitutional };
        let contract  = Contract { invariant_id: con_id, weakens_invariant: true };
        let result = enforce_ibi(&invariant, &contract);
        assert!(result.is_allow());
    }

    // validate_witness proofs

    #[kani::proof]
    fn validate_witness_empty_contract() {
        // PROVES: empty contract (no witnesses) is always rejected
        let contract = EnsuresContract::empty();
        assert!(!validate_witness(&contract));
    }

    #[kani::proof]
    fn validate_witness_single_valid() {
        // PROVES: single valid witness is accepted
        let mut contract = EnsuresContract::empty();
        contract.add_witness(Witness {
            kind: WitnessKind::Postcondition, valid: true, hash: 42
        });
        assert!(validate_witness(&contract));
    }

    #[kani::proof]
    fn validate_witness_single_invalid() {
        // PROVES: single invalid witness is rejected
        let mut contract = EnsuresContract::empty();
        contract.add_witness(Witness {
            kind: WitnessKind::Postcondition, valid: false, hash: 42
        });
        assert!(!validate_witness(&contract));
    }
}
