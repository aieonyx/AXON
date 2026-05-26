//! # axon::audit
//!
//! AXON hash-chained audit trail — compatible with AIEONYX Aegis Collective.
//!
//! Every security-critical event in an AXON program is recorded as an
//! immutable, hash-linked entry. Tampering with any event breaks the
//! chain — detectable by any verifier.
//!
//! ## Chain integrity
//!
//! Each event carries SHA-256 of the previous event.
//! Genesis event carries [0u8; 32].
//! Chain verification: recompute all hashes, compare prev_hash fields.
//!
//! ## Integration points
//!
//! - axon::verify — SecurityCritical postconditions emit audit events
//! - axon::ai — InferenceWeight::SecurityCritical calls emit events
//! - P6+ UAT — unsafe blocks emit via audit_unsafe!()
//! - Sovereign Consent Doctrine — consent requests/decisions recorded

pub mod chain;
pub mod consent;
pub mod event;
pub mod sink;

pub use event::{AuditEvent, EventKind};
pub use chain::{AuditChain, ChainVerification};
pub use sink::{AuditSink, MemorySink, StdoutSink};
pub use consent::{ConsentGate, ConsentDecision, ConsentRequest, RequestKind};

use std::fmt;

/// Result type for axon::audit operations.
pub type AuditResult<T> = Result<T, AuditError>;

/// Errors from the axon::audit module.
#[derive(Debug, Clone)]
pub enum AuditError {
    /// The hash chain is broken at the given event index.
    ChainBroken { event_id: u64, expected: [u8; 32], actual: [u8; 32] },
    /// The sink failed to record an event.
    SinkError(String),
    /// Serialization failed.
    SerializationError(String),
}

impl fmt::Display for AuditError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AuditError::ChainBroken { event_id, .. } =>
                write!(f, "axon::audit: chain broken at event {event_id}"),
            AuditError::SinkError(m) =>
                write!(f, "axon::audit: sink error — {m}"),
            AuditError::SerializationError(m) =>
                write!(f, "axon::audit: serialization error — {m}"),
        }
    }
}

impl std::error::Error for AuditError {}
