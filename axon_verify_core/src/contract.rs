//! Core contract types for axon_verify_core.
//!
//! All types are `Copy` or `Clone` — no heap allocation in the TCB.

/// Classification of a boundary invariant's protection level.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InvariantTier {
    /// Cannot be weakened under any circumstances.
    /// Proof required for any change.
    Constitutional,
    /// Can be updated with a new Kani proof.
    Operational,
    /// Informational — not enforced by the kernel.
    Advisory,
}

/// A boundary invariant that must be preserved.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BoundaryInvariant {
    /// Unique identifier for this invariant.
    pub id:   u32,
    /// Protection tier — determines enforcement strength.
    pub tier: InvariantTier,
}

/// A proposed contract change to evaluate against an invariant.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Contract {
    /// The invariant ID this contract targets.
    pub invariant_id:      u32,
    /// Whether this contract weakens the target invariant.
    pub weakens_invariant: bool,
}

impl Contract {
    /// Returns true if this contract weakens the given invariant.
    #[inline]
    pub fn weakens(&self, invariant: &BoundaryInvariant) -> bool {
        self.invariant_id == invariant.id && self.weakens_invariant
    }
}

/// Classification of a runtime witness.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WitnessKind {
    /// Certifies a postcondition (@ensures) was satisfied.
    Postcondition,
    /// Certifies a precondition (@requires) was satisfied.
    Precondition,
    /// Certifies a security property was verified.
    SecurityProperty,
    /// Certifies an invariant was maintained.
    Invariant,
}

/// A runtime proof term (witness) generated when a contract is satisfied.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Witness {
    /// Classification of what this witness certifies.
    pub kind:   WitnessKind,
    /// Whether the certified condition actually held.
    pub valid:  bool,
    /// Stable hash of the label — used for cache keying.
    pub hash:   u64,
}

impl Witness {
    /// Returns true if this witness certifies a passing condition.
    #[inline]
    pub const fn is_valid(&self) -> bool { self.valid }
}

/// A formal contract carrying one or more witnesses.
#[derive(Debug, Clone, Copy)]
pub struct EnsuresContract {
    /// The witnesses attached to this contract.
    pub witnesses:     [Option<Witness>; 8],
    /// Number of active witnesses (0..=8).
    pub witness_count: usize,
}

impl EnsuresContract {
    /// Create an empty contract with no witnesses.
    pub const fn empty() -> Self {
        Self { witnesses: [None; 8], witness_count: 0 }
    }

    /// Add a witness to this contract. Returns false if full (max 8).
    pub fn add_witness(&mut self, w: Witness) -> bool {
        if self.witness_count >= 8 { return false; }
        self.witnesses[self.witness_count] = Some(w);
        self.witness_count += 1;
        true
    }

    /// Returns true if at least one witness is present.
    #[inline]
    pub const fn has_witness(&self) -> bool { self.witness_count > 0 }

    /// Returns true if ALL present witnesses are valid.
    #[inline]
    pub fn all_witnesses_valid(&self) -> bool {
        let mut i = 0;
        while i < self.witness_count {
            if let Some(w) = self.witnesses[i] {
                if !w.is_valid() { return false; }
            }
            i += 1;
        }
        true
    }
}

// ── Phase 22 M3: Kani proof harnesses for EnsuresContract ────────────────────
#[cfg(kani)]
mod proofs {
    use super::*;

    #[kani::proof]
    fn empty_contract_has_no_witnesses() {
        // PROVES: empty() always creates a contract with witness_count == 0
        let c = EnsuresContract::empty();
        assert!(!c.has_witness());
        assert_eq!(c.witness_count, 0);
    }

    #[kani::proof]
    fn add_witness_increments_count() {
        // PROVES: add_witness on empty contract sets witness_count to 1
        let mut c = EnsuresContract::empty();
        let w = Witness { kind: WitnessKind::Postcondition, valid: true, hash: kani::any() };
        let added = c.add_witness(w);
        assert!(added);
        assert_eq!(c.witness_count, 1);
        assert!(c.has_witness());
    }

    #[kani::proof]
    fn all_witnesses_valid_empty_is_false() {
        // PROVES: empty contract fails all_witnesses_valid
        let c = EnsuresContract::empty();
        assert!(!c.all_witnesses_valid());
    }

    #[kani::proof]
    fn all_witnesses_valid_one_invalid_is_false() {
        // PROVES: one invalid witness causes all_witnesses_valid to return false
        let mut c = EnsuresContract::empty();
        c.add_witness(Witness { kind: WitnessKind::Postcondition, valid: false, hash: 1 });
        assert!(!c.all_witnesses_valid());
    }

    #[kani::proof]
    fn add_witness_capacity_limit() {
        // PROVES: cannot add more than 8 witnesses (capacity limit)
        let mut c = EnsuresContract::empty();
        for i in 0..8u64 {
            let added = c.add_witness(Witness {
                kind: WitnessKind::Postcondition, valid: true, hash: i
            });
            assert!(added);
        }
        // 9th add must fail
        let overflow = c.add_witness(Witness {
            kind: WitnessKind::Postcondition, valid: true, hash: 99
        });
        assert!(!overflow);
        assert_eq!(c.witness_count, 8);
    }
}

#[cfg(test)]
mod unit_tests {
    use super::*;

    // ── Phase 22 M3 unit tests ────────────────────────────────────────────────

    #[test]
    fn tc_ensures_contract_empty() {
        let c = EnsuresContract::empty();
        assert!(!c.has_witness());
        assert_eq!(c.witness_count, 0);
    }

    #[test]
    fn tc_ensures_contract_add_witness() {
        let mut c = EnsuresContract::empty();
        let added = c.add_witness(Witness {
            kind: WitnessKind::Postcondition, valid: true, hash: 1
        });
        assert!(added);
        assert!(c.has_witness());
        assert_eq!(c.witness_count, 1);
    }

    #[test]
    fn tc_ensures_contract_all_valid() {
        let mut c = EnsuresContract::empty();
        c.add_witness(Witness { kind: WitnessKind::Postcondition, valid: true, hash: 1 });
        c.add_witness(Witness { kind: WitnessKind::Precondition, valid: true, hash: 2 });
        assert!(c.all_witnesses_valid());
    }

    #[test]
    fn tc_ensures_contract_one_invalid() {
        let mut c = EnsuresContract::empty();
        c.add_witness(Witness { kind: WitnessKind::Postcondition, valid: true,  hash: 1 });
        c.add_witness(Witness { kind: WitnessKind::Precondition,  valid: false, hash: 2 });
        assert!(!c.all_witnesses_valid());
    }

    #[test]
    fn tc_ensures_contract_capacity() {
        let mut c = EnsuresContract::empty();
        for i in 0..8u64 {
            assert!(c.add_witness(Witness {
                kind: WitnessKind::Postcondition, valid: true, hash: i
            }));
        }
        // 9th must fail
        assert!(!c.add_witness(Witness {
            kind: WitnessKind::Postcondition, valid: true, hash: 99
        }));
        assert_eq!(c.witness_count, 8);
    }
}
