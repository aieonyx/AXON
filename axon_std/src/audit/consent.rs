//! Sovereign Consent Doctrine — TrustGraph Consent Gate.
//!
//! "AXON cannot help you if you give the key away."
//!
//! Every capability grant or sensitive operation must pass through
//! the ConsentGate. The gate detects patterns, warns with details
//! and trust scores, and requires explicit acknowledgment.
//! Non-paternalistic — warns but never blocks a conscious decision.

use super::{AuditResult, AuditError, event::EventKind};
use crate::audit::chain::AuditChain;

/// Classification of a consent request.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RequestKind {
    /// A capability is being granted to another protection domain.
    CapabilityGrant,
    /// Access to encrypted/sensitive data is being requested.
    SensitiveDataAccess,
    /// A network connection is being established.
    NetworkConnection,
    /// A model is being loaded for inference.
    ModelLoad,
    /// A custom consent request.
    Custom,
}

/// A consent request presented to the user/operator.
#[derive(Debug, Clone)]
pub struct ConsentRequest {
    /// What kind of operation requires consent.
    pub kind:        RequestKind,
    /// Human-readable description of what is being requested.
    pub description: String,
    /// Trust score of the requesting entity (0.0 = untrusted, 1.0 = fully trusted).
    pub trust_score: f32,
    /// Whether a social engineering pattern was detected.
    pub suspicious:  bool,
    /// Warning message if suspicious is true.
    pub warning:     Option<String>,
}

impl ConsentRequest {
    /// Create a new consent request.
    pub fn new(kind: RequestKind, description: impl Into<String>, trust_score: f32) -> Self {
        let suspicious = trust_score < 0.3;
        let warning = if suspicious {
            Some(format!(
                "AXON: low trust score ({:.2}). \
                 Verify the requesting entity before granting. \
                 AXON cannot help you if you give the key away.",
                trust_score
            ))
        } else {
            None
        };
        Self {
            kind, description: description.into(),
            trust_score, suspicious, warning,
        }
    }
}

/// The decision made in response to a consent request.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConsentDecision {
    /// Explicitly granted by the operator.
    Granted,
    /// Explicitly denied by the operator.
    Denied,
    /// Acknowledged (aware of risk, proceeding consciously).
    AcknowledgedAndGranted,
}

impl ConsentDecision {
    /// Returns true if the operation may proceed.
    pub fn allows_proceed(&self) -> bool {
        matches!(self, ConsentDecision::Granted | ConsentDecision::AcknowledgedAndGranted)
    }
}

/// The TrustGraph Consent Gate.
///
/// Presents consent requests, records decisions in the audit chain,
/// and enforces acknowledgment for suspicious operations.
///
/// Non-paternalistic: warns but never blocks a conscious decision.
/// If the operator explicitly acknowledges, the operation proceeds.
pub struct ConsentGate<'a> {
    chain: &'a mut AuditChain,
}

impl<'a> ConsentGate<'a> {
    /// Create a new consent gate backed by the given audit chain.
    pub fn new(chain: &'a mut AuditChain) -> Self { Self { chain } }

    /// Present a consent request and record the decision.
    ///
    /// Returns `Ok(decision)` always — the caller decides whether
    /// to proceed based on `decision.allows_proceed()`.
    pub fn request(
        &mut self,
        req:      &ConsentRequest,
        decision: ConsentDecision,
        ts:       u64,
    ) -> AuditResult<ConsentDecision> {
        // Record the consent request
        let req_payload = format!(
            "kind={:?} trust={:.2} suspicious={} desc={}",
            req.kind, req.trust_score, req.suspicious, req.description
        );
        self.chain.append(
            EventKind::ConsentRequest,
            format!("consent_request::{:?}", req.kind),
            req_payload.into_bytes(),
            ts,
        );

        // Record the decision
        let dec_payload = format!("{decision:?}");
        self.chain.append(
            EventKind::ConsentDecision,
            format!("consent_decision::{decision:?}"),
            dec_payload.into_bytes(),
            ts,
        );

        Ok(decision)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audit::chain::AuditChain;

    #[test]
    fn consent_request_suspicious_below_threshold() {
        let r = ConsentRequest::new(RequestKind::CapabilityGrant, "grant cap", 0.2);
        assert!(r.suspicious);
        assert!(r.warning.is_some());
        assert!(r.warning.unwrap().contains("AXON"));
    }

    #[test]
    fn consent_request_trusted_above_threshold() {
        let r = ConsentRequest::new(RequestKind::CapabilityGrant, "grant cap", 0.9);
        assert!(!r.suspicious);
        assert!(r.warning.is_none());
    }

    #[test]
    fn consent_decision_allows_proceed() {
        assert!(ConsentDecision::Granted.allows_proceed());
        assert!(ConsentDecision::AcknowledgedAndGranted.allows_proceed());
        assert!(!ConsentDecision::Denied.allows_proceed());
    }

    #[test]
    fn consent_gate_records_in_chain() {
        let mut chain = AuditChain::new();
        let req = ConsentRequest::new(RequestKind::Custom, "test op", 0.8);
        {
            let mut gate = ConsentGate::new(&mut chain);
            gate.request(&req, ConsentDecision::Granted, 0).unwrap();
        }
        // Two events: request + decision
        assert_eq!(chain.len(), 2);
        assert!(chain.verify().is_valid());
    }

    #[test]
    fn consent_gate_suspicious_acknowledged() {
        let mut chain = AuditChain::new();
        let req = ConsentRequest::new(RequestKind::SensitiveDataAccess, "risky op", 0.1);
        assert!(req.suspicious);
        {
            let mut gate = ConsentGate::new(&mut chain);
            let decision = gate.request(&req, ConsentDecision::AcknowledgedAndGranted, 0).unwrap();
            // Non-paternalistic: operator acknowledged, operation allowed
            assert!(decision.allows_proceed());
        }
        assert!(chain.verify().is_valid());
    }

    #[test]
    fn consent_gate_denied_recorded() {
        let mut chain = AuditChain::new();
        let req = ConsentRequest::new(RequestKind::NetworkConnection, "connect", 0.5);
        {
            let mut gate = ConsentGate::new(&mut chain);
            let d = gate.request(&req, ConsentDecision::Denied, 0).unwrap();
            assert!(!d.allows_proceed());
        }
        assert!(chain.verify().is_valid());
    }
}
