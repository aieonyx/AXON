// Copyright (c) 2026 Edison Lepiten / AIEONYX
//! GENESIS capability broker — CB-01 through CB-10.
//!
//! The Capability Broker is the single mediating component for all
//! seL4 capability grants in AXON. No capability may be granted
//! without passing through the broker.
//!
//! CB-01: Grant endpoint capability to PD
//! CB-02: Grant notification capability to PD
//! CB-03: Grant untyped memory capability to PD
//! CB-04: Grant IRQ handler capability to PD
//! CB-05: Grant page capability to PD
//! CB-06: Revoke capability from PD
//! CB-07: Audit capability grant (hash-chained log)
//! CB-08: Validate PD identity before grant
//! CB-09: Enforce capability scope limits
//! CB-10: Emergency revocation of all PD capabilities

use axon_sel4::types::Cap;
use axon_sel4::cap::{CNodeSlot, cnode_copy, cnode_delete, rights};

/// Maximum PDs tracked by the broker.
pub const MAX_PDS: usize = 32;
/// Maximum capabilities per PD.
pub const MAX_CAPS_PER_PD: usize = 16;

/// Protection Domain identity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PdId(pub u32);

impl PdId {
    pub const ROOT: PdId = PdId(0);
}

/// Capability type tracked by the broker.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CapType {
    Endpoint,
    Notification,
    Untyped,
    IrqHandler,
    Page,
}

/// A capability grant record.
#[derive(Debug, Clone, Copy)]
pub struct CapGrant {
    pub pd:       PdId,
    pub cap:      Cap,
    pub cap_type: CapType,
    pub slot:     Cap,
}

/// Capability broker error.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BrokerError {
    PdNotFound,
    CapLimitExceeded,
    InvalidCapType,
    RevocationFailed,
    SeL4Error(u64),
}

/// The GENESIS Capability Broker.
pub struct CapabilityBroker {
    /// Sparse grant table — slots reused after revocation.
    grants: [Option<CapGrant>; MAX_PDS],
    /// High-water mark for iteration (never decremented).
    hwm: usize,
}

impl CapabilityBroker {
    pub const fn new() -> Self {
        Self { grants: [None; MAX_PDS], hwm: 0 }
    }

    /// Find a free slot — reuses revoked slots before extending hwm.
    fn find_free_slot(&self) -> Option<usize> {
        // Prefer reusing a revoked slot within hwm
        for i in 0..self.hwm {
            if self.grants[i].is_none() { return Some(i); }
        }
        // Extend hwm if space remains
        if self.hwm < MAX_PDS { Some(self.hwm) } else { None }
    }

    /// Count active grants for a given PD — used to enforce per-PD limit.
    fn pd_grant_count(&self, pd: PdId) -> usize {
        self.grants[..self.hwm]
            .iter()
            .filter(|g| matches!(g, Some(g) if g.pd == pd))
            .count()
    }

    /// CB-01/02/03/04/05: Grant a capability to a PD.
    pub fn grant(
        &mut self,
        pd: PdId,
        cap: Cap,
        cap_type: CapType,
        dest_slot: Cap,
        cnode_root: Cap,
    ) -> Result<(), BrokerError> {
        // CB-09: Enforce per-PD capability budget.
        if self.pd_grant_count(pd) >= MAX_CAPS_PER_PD {
            return Err(BrokerError::CapLimitExceeded);
        }
        // Find a reusable or new slot.
        let slot_idx = self.find_free_slot().ok_or(BrokerError::CapLimitExceeded)?;
        // CB-08: Validate PD identity (stub — P42 wires real identity check)
        let src = CNodeSlot { root: cnode_root, index: cap, depth: 64 };
        let dst = CNodeSlot { root: cnode_root, index: dest_slot, depth: 64 };
        let err = cnode_copy(dst, src, rights::ALL);
        if err != 0 { return Err(BrokerError::SeL4Error(err)); }
        // CB-07: Audit the grant
        self.audit_grant(pd, cap, cap_type, dest_slot);
        self.grants[slot_idx] = Some(CapGrant { pd, cap, cap_type, slot: dest_slot });
        if slot_idx == self.hwm { self.hwm += 1; }
        Ok(())
    }

    /// CB-06: Revoke a specific capability from a PD.
    pub fn revoke(&mut self, pd: PdId, slot: Cap, cnode_root: Cap) -> Result<(), BrokerError> {
        let pos = self.grants[..self.hwm]
            .iter()
            .position(|g| matches!(g, Some(g) if g.pd == pd && g.slot == slot));
        let pos = pos.ok_or(BrokerError::PdNotFound)?;
        let err = cnode_delete(cnode_root, slot, 64);
        if err != 0 { return Err(BrokerError::RevocationFailed); }
        self.grants[pos] = None;
        Ok(())
    }

    /// CB-10: Emergency revocation of all capabilities for a PD.
    pub fn revoke_all(&mut self, pd: PdId, cnode_root: Cap) {
        for slot in self.grants[..self.hwm].iter_mut() {
            if let Some(g) = slot {
                if g.pd == pd {
                    let _ = cnode_delete(cnode_root, g.slot, 64);
                    *slot = None;
                }
            }
        }
    }

    /// Number of active grants across all PDs.
    pub fn grant_count(&self) -> usize {
        self.grants[..self.hwm].iter().filter(|g| g.is_some()).count()
    }

    /// CB-07: Audit hook — P41 stub, P42 wires to hash-chained audit log.
    fn audit_grant(&self, _pd: PdId, _cap: Cap, _cap_type: CapType, _slot: Cap) {
        // Stub — wired to axon_std audit chain in P42
    }
}

impl Default for CapabilityBroker {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tp41_broker_grant_and_count() {
        let mut broker = CapabilityBroker::new();
        broker.grant(PdId(1), 10, CapType::Endpoint, 20, 1).unwrap();
        broker.grant(PdId(1), 11, CapType::Notification, 21, 1).unwrap();
        assert_eq!(broker.grant_count(), 2);
    }

    #[test]
    fn tp41_broker_revoke() {
        let mut broker = CapabilityBroker::new();
        broker.grant(PdId(2), 10, CapType::Endpoint, 20, 1).unwrap();
        assert_eq!(broker.grant_count(), 1);
        broker.revoke(PdId(2), 20, 1).unwrap();
        assert_eq!(broker.grant_count(), 0);
    }

    #[test]
    fn tp41_broker_revoke_all() {
        let mut broker = CapabilityBroker::new();
        broker.grant(PdId(3), 10, CapType::Endpoint,     20, 1).unwrap();
        broker.grant(PdId(3), 11, CapType::Notification, 21, 1).unwrap();
        broker.grant(PdId(4), 12, CapType::Page,         22, 1).unwrap();
        broker.revoke_all(PdId(3), 1);
        // PD(4) grant still active
        assert_eq!(broker.grant_count(), 1);
    }

    #[test]
    fn tp41_broker_revoke_unknown_pd_fails() {
        let mut broker = CapabilityBroker::new();
        assert_eq!(broker.revoke(PdId(99), 10, 1), Err(BrokerError::PdNotFound));
    }

    #[test]
    #[test]
    fn tp41_broker_per_pd_cap_limit() {
        let mut broker = CapabilityBroker::new();
        // Fill one PD to its per-PD limit
        for i in 0..MAX_CAPS_PER_PD as u64 {
            broker.grant(PdId(1), i + 10, CapType::Endpoint, i + 100, 1).unwrap();
        }
        // Next grant to same PD must fail
        let err = broker.grant(PdId(1), 999, CapType::Endpoint, 999, 1);
        assert_eq!(err, Err(BrokerError::CapLimitExceeded));
        // Different PD can still grant
        assert!(broker.grant(PdId(2), 50, CapType::Endpoint, 200, 1).is_ok());
    }
    #[test]
    fn tp41_broker_slot_reuse_after_revoke() {
        let mut broker = CapabilityBroker::new();
        // Fill to per-PD limit
        for i in 0..MAX_CAPS_PER_PD as u64 {
            broker.grant(PdId(3), i + 10, CapType::Endpoint, i + 100, 1).unwrap();
        }
        // Revoke one
        broker.revoke(PdId(3), 100, 1).unwrap();
        // Should be able to grant again
        assert!(broker.grant(PdId(3), 77, CapType::Endpoint, 177, 1).is_ok());
    }

    #[test]
    fn tp41_pd_root_constant() {
        assert_eq!(PdId::ROOT, PdId(0));
    }
}
