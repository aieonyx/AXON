#![allow(clippy::all, dead_code, unused_imports)]
//! AuditChain — SHA-256 hash-chained event log.

use sha2::{Sha256, Digest};
use super::{AuditError, AuditResult, event::{AuditEvent, EventKind}};

/// The result of chain integrity verification.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChainVerification {
    /// All hashes are consistent — chain is intact.
    Valid { event_count: usize },
    /// A hash mismatch was detected.
    Broken { at_id: u64, expected: [u8; 32], actual: [u8; 32] },
    /// The chain is empty.
    Empty,
}

impl ChainVerification {
    /// Returns true if the chain is intact.
    pub fn is_valid(&self) -> bool {
        matches!(self, ChainVerification::Valid { .. })
    }
}

/// A hash-chained audit log.
///
/// Events are appended immutably. Each event's SHA-256 hash is the
/// `prev_hash` of the next event. Tampering with any event breaks
/// all subsequent hashes.
#[derive(Debug, Default)]
pub struct AuditChain {
    events:    Vec<AuditEvent>,
    next_id:   u64,
    last_hash: [u8; 32],
}

impl AuditChain {
    /// Create a new empty chain. The genesis hash is all zeros.
    pub fn new() -> Self {
        Self { events: Vec::new(), next_id: 1, last_hash: [0u8; 32] }
    }

    /// Append an event to the chain.
    ///
    /// The event's `prev_hash` and `id` are set automatically.
    pub fn append(
        &mut self,
        kind:      EventKind,
        label:     impl Into<String>,
        payload:   Vec<u8>,
        timestamp: u64,
    ) -> &AuditEvent {
        let event = AuditEvent {
            id:        self.next_id,
            kind,
            label:     label.into(),
            payload,
            prev_hash: self.last_hash,
            timestamp,
        };

        // Compute SHA-256 of the new event for the next link
        let mut hasher = Sha256::new();
        hasher.update(&event.to_bytes());
        let hash_result = hasher.finalize();
        self.last_hash.copy_from_slice(&hash_result);

        self.next_id += 1;
        self.events.push(event);
        self.events.last().unwrap()
    }

    /// Verify the integrity of the entire chain.
    ///
    /// Recomputes all hashes and checks `prev_hash` fields.
    pub fn verify(&self) -> ChainVerification {
        if self.events.is_empty() {
            return ChainVerification::Empty;
        }

        let mut expected_prev = [0u8; 32]; // genesis

        for event in &self.events {
            if event.prev_hash != expected_prev {
                return ChainVerification::Broken {
                    at_id:    event.id,
                    expected: expected_prev,
                    actual:   event.prev_hash,
                };
            }
            // Compute hash of this event for the next check
            let mut hasher = Sha256::new();
            hasher.update(&event.to_bytes());
            expected_prev.copy_from_slice(&hasher.finalize());
        }

        ChainVerification::Valid { event_count: self.events.len() }
    }

    /// Return all events in the chain.
    pub fn events(&self) -> &[AuditEvent] { &self.events }

    /// Return mutable access to events — for testing tamper detection only.
    #[doc(hidden)]
    pub fn events_mut(&mut self) -> &mut Vec<AuditEvent> { &mut self.events }

    /// Number of events in the chain.
    pub fn len(&self) -> usize { self.events.len() }

    /// True if the chain has no events.
    pub fn is_empty(&self) -> bool { self.events.is_empty() }

    /// SHA-256 of the last event (the chain tip).
    pub fn tip_hash(&self) -> [u8; 32] { self.last_hash }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audit::event::EventKind;

    fn ts() -> u64 { 1_000_000_000 }

    #[test]
    fn chain_empty_on_creation() {
        let c = AuditChain::new();
        assert!(c.is_empty());
        assert_eq!(c.verify(), ChainVerification::Empty);
    }

    #[test]
    fn chain_single_event_valid() {
        let mut c = AuditChain::new();
        c.append(EventKind::Custom, "test", b"data".to_vec(), ts());
        assert_eq!(c.len(), 1);
        assert!(c.verify().is_valid());
    }

    #[test]
    fn chain_multiple_events_valid() {
        let mut c = AuditChain::new();
        for i in 0..5 {
            c.append(EventKind::Custom, format!("event_{i}"), vec![], ts());
        }
        assert_eq!(c.len(), 5);
        assert!(c.verify().is_valid());
    }

    #[test]
    fn chain_ids_are_sequential() {
        let mut c = AuditChain::new();
        c.append(EventKind::Custom, "a", vec![], ts());
        c.append(EventKind::Custom, "b", vec![], ts());
        assert_eq!(c.events()[0].id, 1);
        assert_eq!(c.events()[1].id, 2);
    }

    #[test]
    fn chain_genesis_prev_hash_is_zeros() {
        let mut c = AuditChain::new();
        c.append(EventKind::Custom, "first", vec![], ts());
        assert_eq!(c.events()[0].prev_hash, [0u8; 32]);
    }

    #[test]
    fn chain_tampered_event_breaks_verification() {
        let mut c = AuditChain::new();
        c.append(EventKind::Custom, "a", vec![], ts());
        c.append(EventKind::Custom, "b", vec![], ts());
        // Tamper with event 1's label
        c.events[0].label = "TAMPERED".to_string();
        assert!(!c.verify().is_valid());
    }

    #[test]
    fn chain_tip_hash_changes_with_events() {
        let mut c = AuditChain::new();
        let tip0 = c.tip_hash();
        c.append(EventKind::Custom, "x", vec![], ts());
        let tip1 = c.tip_hash();
        assert_ne!(tip0, tip1);
    }

    #[test]
    fn chain_postcondition_event() {
        let mut c = AuditChain::new();
        c.append(EventKind::Postcondition, "result_nonneg",
                 b"true".to_vec(), ts());
        assert_eq!(c.events()[0].kind, EventKind::Postcondition);
        assert!(c.verify().is_valid());
    }
}
