//! DWC — Dynamic Witness Contracts (P6+ feature).
//!
//! A witness is a proof term generated at runtime when a postcondition
//! passes. Witnesses can be accumulated, compared, and passed to
//! QuorumGate for multi-witness enforcement (QCC).

/// Classification of what the witness certifies.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WitnessKind {
    /// A postcondition (@ensures) was satisfied.
    Postcondition,
    /// A precondition (@requires) was satisfied.
    Precondition,
    /// An invariant (@invariant) was maintained.
    Invariant,
    /// A security property was verified.
    SecurityProperty,
}

/// A runtime proof term generated when a contract is satisfied.
///
/// Witnesses are lightweight — just a label, kind, and hash.
/// They can be passed to [`QuorumGate`][super::quorum::QuorumGate]
/// for multi-party enforcement.
#[derive(Debug, Clone)]
pub struct DynamicWitness {
    /// The contract label this witness certifies.
    pub label:   &'static str,
    /// Classification of the certified property.
    pub kind:    WitnessKind,
    /// FNV-64 hash of the label (for cache key alignment).
    pub hash:    u64,
    /// Whether the certified condition held.
    pub passed:  bool,
}

impl DynamicWitness {
    /// Generate a witness for a postcondition check.
    pub fn postcondition(label: &'static str, passed: bool) -> Self {
        Self {
            label,
            kind:   WitnessKind::Postcondition,
            hash:   super::check::fnv64(label.as_bytes()),
            passed,
        }
    }

    /// Generate a witness for an invariant check.
    pub fn invariant(label: &'static str, passed: bool) -> Self {
        Self {
            label,
            kind:   WitnessKind::Invariant,
            hash:   super::check::fnv64(label.as_bytes()),
            passed,
        }
    }

    /// Generate a witness for a security property check.
    pub fn security(label: &'static str, passed: bool) -> Self {
        Self {
            label,
            kind:   WitnessKind::SecurityProperty,
            hash:   super::check::fnv64(label.as_bytes()),
            passed,
        }
    }

    /// Returns true if this witness certifies a passing condition.
    pub fn is_valid(&self) -> bool { self.passed }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn witness_postcondition_valid() {
        let w = DynamicWitness::postcondition("result_nonneg", true);
        assert!(w.is_valid());
        assert_eq!(w.kind, WitnessKind::Postcondition);
        assert_eq!(w.label, "result_nonneg");
    }

    #[test]
    fn witness_invalid_on_false() {
        let w = DynamicWitness::postcondition("always_false", false);
        assert!(!w.is_valid());
    }

    #[test]
    fn witness_hash_deterministic() {
        let w1 = DynamicWitness::postcondition("label", true);
        let w2 = DynamicWitness::postcondition("label", true);
        assert_eq!(w1.hash, w2.hash);
    }

    #[test]
    fn witness_different_labels_different_hashes() {
        let w1 = DynamicWitness::postcondition("a", true);
        let w2 = DynamicWitness::postcondition("b", true);
        assert_ne!(w1.hash, w2.hash);
    }

    #[test]
    fn witness_security_kind() {
        let w = DynamicWitness::security("cap_grant_verified", true);
        assert_eq!(w.kind, WitnessKind::SecurityProperty);
    }
}
