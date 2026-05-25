//! QCC — Quorum Consensus Contracts (P6+ feature).
//!
//! Bio DNA: Quorum sensing — bacteria only activate group behaviors
//! when enough cells signal agreement. QCC applies the same principle:
//! a postcondition is only accepted when N independent witnesses agree.
//!
//! Use in security-critical contexts: capability grants, audit events,
//! cross-domain verification where a single verifier is insufficient.

use super::witness::DynamicWitness;
use super::check::{VerificationError, VerifyResult, fnv64};

/// The result of a quorum check.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QuorumResult {
    /// All required witnesses agree — quorum reached.
    Reached { witnesses: usize },
    /// Not enough witnesses yet.
    Pending { have: usize, need: usize },
    /// A witness disagreed — quorum failed.
    Failed  { dissents: usize },
}

impl QuorumResult {
    /// Returns true if the quorum was reached.
    pub fn is_reached(&self) -> bool {
        matches!(self, QuorumResult::Reached { .. })
    }
}

/// A quorum gate that requires N witnesses before accepting a contract.
///
/// # P6+ QCC
///
/// `@quorum(N)` in AXON requires N independent verifiers to agree on
/// a postcondition before it is accepted. `QuorumGate` enforces this.
///
/// # Examples
///
/// ```rust
/// use axon_std::verify::quorum::QuorumGate;
/// use axon_std::verify::witness::DynamicWitness;
///
/// let mut gate = QuorumGate::new("cap_grant_approved", 2);
/// gate.add_witness(DynamicWitness::security("cap_grant_approved", true));
/// gate.add_witness(DynamicWitness::security("cap_grant_approved", true));
/// assert!(gate.check().is_reached());
/// ```
#[derive(Debug)]
pub struct QuorumGate {
    label:    &'static str,
    required: usize,
    witnesses: Vec<DynamicWitness>,
}

impl QuorumGate {
    /// Create a new gate requiring `n` witnesses.
    pub fn new(label: &'static str, n: usize) -> Self {
        Self { label, required: n, witnesses: Vec::new() }
    }

    /// Add a witness to the gate.
    pub fn add_witness(&mut self, w: DynamicWitness) {
        self.witnesses.push(w);
    }

    /// Check the current quorum state.
    pub fn check(&self) -> QuorumResult {
        let passing  = self.witnesses.iter().filter(|w| w.is_valid()).count();
        let failing  = self.witnesses.iter().filter(|w| !w.is_valid()).count();

        if failing > 0 {
            QuorumResult::Failed { dissents: failing }
        } else if passing >= self.required {
            QuorumResult::Reached { witnesses: passing }
        } else {
            QuorumResult::Pending { have: passing, need: self.required }
        }
    }

    /// Check and return a VerifyResult — passes only when quorum is reached.
    pub fn enforce(&self) -> VerifyResult<()> {
        match self.check() {
            QuorumResult::Reached { .. } => Ok(()),
            QuorumResult::Pending { have, need } => Err(VerificationError {
                label: self.label,
                description: format!("QCC: quorum pending — have {have}/{need} witnesses"),
                fn_hash: fnv64(self.label.as_bytes()),
            }),
            QuorumResult::Failed { dissents } => Err(VerificationError {
                label: self.label,
                description: format!("QCC: quorum failed — {dissents} dissenting witness(es)"),
                fn_hash: fnv64(self.label.as_bytes()),
            }),
        }
    }

    /// Number of witnesses currently registered.
    pub fn witness_count(&self) -> usize { self.witnesses.len() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::witness::DynamicWitness;

    fn passing_witness(label: &'static str) -> DynamicWitness {
        DynamicWitness::postcondition(label, true)
    }
    fn failing_witness(label: &'static str) -> DynamicWitness {
        DynamicWitness::postcondition(label, false)
    }

    #[test]
    fn quorum_pending_with_no_witnesses() {
        let g = QuorumGate::new("test", 2);
        assert_eq!(g.check(), QuorumResult::Pending { have: 0, need: 2 });
    }

    #[test]
    fn quorum_reached_with_enough_witnesses() {
        let mut g = QuorumGate::new("test", 2);
        g.add_witness(passing_witness("test"));
        g.add_witness(passing_witness("test"));
        assert!(g.check().is_reached());
    }

    #[test]
    fn quorum_failed_on_dissent() {
        let mut g = QuorumGate::new("test", 2);
        g.add_witness(passing_witness("test"));
        g.add_witness(failing_witness("test"));
        assert!(matches!(g.check(), QuorumResult::Failed { .. }));
    }

    #[test]
    fn quorum_enforce_ok_when_reached() {
        let mut g = QuorumGate::new("test", 1);
        g.add_witness(passing_witness("test"));
        assert!(g.enforce().is_ok());
    }

    #[test]
    fn quorum_enforce_err_when_pending() {
        let g = QuorumGate::new("test", 3);
        assert!(g.enforce().is_err());
    }

    #[test]
    fn quorum_enforce_err_when_failed() {
        let mut g = QuorumGate::new("test", 2);
        g.add_witness(failing_witness("test"));
        assert!(g.enforce().is_err());
    }

    #[test]
    fn quorum_witness_count() {
        let mut g = QuorumGate::new("test", 2);
        assert_eq!(g.witness_count(), 0);
        g.add_witness(passing_witness("test"));
        assert_eq!(g.witness_count(), 1);
    }
}
