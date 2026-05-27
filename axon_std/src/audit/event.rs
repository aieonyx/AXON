//! AuditEvent — the atomic unit of the audit trail.

use serde::{Deserialize, Serialize};

/// Classification of the audited event.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EventKind {
    /// A formal postcondition (@ensures) was checked.
    Postcondition,
    /// An axon::ai SecurityCritical inference call was made.
    InferenceCall,
    /// An unsafe block was executed (P6+ UAT).
    UnsafeBlock,
    /// A seL4 capability was granted or revoked.
    CapabilityGrant,
    /// A consent request was made or decided (Sovereign Consent Doctrine).
    ConsentRequest,
    /// A consent decision was recorded.
    ConsentDecision,
    /// A security property was verified by QuorumGate.
    QuorumVerification,
    /// A custom application-defined event.
    Custom,
}

impl EventKind {
    /// Human-readable label for logging.
    pub const fn label(&self) -> &'static str {
        match self {
            EventKind::Postcondition      => "postcondition",
            EventKind::InferenceCall      => "inference_call",
            EventKind::UnsafeBlock        => "unsafe_block",
            EventKind::CapabilityGrant    => "capability_grant",
            EventKind::ConsentRequest     => "consent_request",
            EventKind::ConsentDecision    => "consent_decision",
            EventKind::QuorumVerification => "quorum_verification",
            EventKind::Custom             => "custom",
        }
    }
}

/// An immutable audit event in the hash chain.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    /// Monotonically increasing event ID (starts at 1).
    pub id:        u64,
    /// Classification of this event.
    pub kind:      EventKind,
    /// Human-readable label identifying the event source.
    pub label:     String,
    /// Arbitrary event payload (serialized as hex string).
    pub payload:   Vec<u8>,
    /// SHA-256 hash of the previous event (genesis = [0u8; 32]).
    pub prev_hash: [u8; 32],
    /// Nanoseconds since Unix epoch (0 if clock unavailable).
    pub timestamp: u64,
}

impl AuditEvent {
    /// Serialize this event to bytes for hashing.
    ///
    /// Format: id(8) || kind(1) || label_len(2) || label || payload_len(4) || payload || timestamp(8)
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut out = Vec::new();
        out.extend_from_slice(&self.id.to_le_bytes());
        out.push(self.kind as u8);
        let label_bytes = self.label.as_bytes();
        out.extend_from_slice(&(label_bytes.len() as u16).to_le_bytes());
        out.extend_from_slice(label_bytes);
        out.extend_from_slice(&(self.payload.len() as u32).to_le_bytes());
        out.extend_from_slice(&self.payload);
        out.extend_from_slice(&self.timestamp.to_le_bytes());
        out
    }

    /// Display the event as a one-line log entry.
    pub fn to_log_line(&self) -> String {
        let prev = self.prev_hash.iter()
            .take(4)
            .map(|b| format!("{b:02x}"))
            .collect::<String>();
        format!(
            "[audit] id={} kind={} label={:?} prev={}... ts={}",
            self.id, self.kind.label(), self.label, prev, self.timestamp
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_event(id: u64) -> AuditEvent {
        AuditEvent {
            id,
            kind:      EventKind::Postcondition,
            label:     "test_event".to_string(),
            payload:   b"payload".to_vec(),
            prev_hash: [0u8; 32],
            timestamp: 1_000_000,
        }
    }

    #[test]
    fn event_to_bytes_deterministic() {
        let e = sample_event(1);
        assert_eq!(e.to_bytes(), e.to_bytes());
    }

    #[test]
    fn event_to_bytes_different_ids_differ() {
        let e1 = sample_event(1);
        let e2 = sample_event(2);
        assert_ne!(e1.to_bytes(), e2.to_bytes());
    }

    #[test]
    fn event_kind_labels_unique() {
        let kinds = [
            EventKind::Postcondition, EventKind::InferenceCall,
            EventKind::UnsafeBlock, EventKind::CapabilityGrant,
            EventKind::ConsentRequest, EventKind::ConsentDecision,
            EventKind::QuorumVerification, EventKind::Custom,
        ];
        let labels: Vec<_> = kinds.iter().map(|k| k.label()).collect();
        let unique: std::collections::HashSet<_> = labels.iter().collect();
        assert_eq!(labels.len(), unique.len());
    }

    #[test]
    fn event_log_line_contains_id() {
        let e = sample_event(42);
        assert!(e.to_log_line().contains("42"));
    }

    #[test]
    fn event_serializes_to_json() {
        let e = sample_event(1);
        let json = serde_json::to_string(&e).unwrap();
        assert!(json.contains("postcondition") || json.contains("Postcondition"));
    }
}
