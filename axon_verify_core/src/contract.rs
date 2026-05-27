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
