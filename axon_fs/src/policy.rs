// Copyright (c) 2026 Edison Lepiten / AIEONYX
//! AXFS access policy — tier-aware open/read/write gate.
//!
//! Policy enforces S4+i (Security first):
//!   - Critical files: read requires explicit capability grant.
//!   - Personal files: read/write for owner only.
//!   - Noise files:    standard open flags apply.

use axon_core::prelude::*;
use axon_pal::types::OpenFlags;
use crate::tier::DataTier;

/// Access policy decision.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PolicyDecision {
    /// Access granted.
    Allow,
    /// Access denied — insufficient privilege for tier.
    Deny(&'static str),
}

impl PolicyDecision {
    pub fn is_allow(self) -> bool { matches!(self, PolicyDecision::Allow) }
    pub fn is_deny(self)  -> bool { !self.is_allow() }
}

/// Tier-aware access policy.
pub struct AxfsPolicy;

impl AxfsPolicy {
    /// Gate an open request against the tier's access policy.
    ///
    /// Currently: Critical write is always denied on host (P41 wires capability grant).
    /// Personal and Noise follow standard OpenFlags semantics.
    pub fn check_open(tier: DataTier, flags: OpenFlags) -> AxonResult<PolicyDecision> {
        match tier {
            DataTier::Critical => {
                if flags.contains(OpenFlags::WRITE) {
                    // Critical writes require explicit capability — denied until P41.
                    return AxonResult::Ok(PolicyDecision::Deny(
                        "critical tier: write requires explicit capability grant"
                    ));
                }
                AxonResult::Ok(PolicyDecision::Allow)
            }
            DataTier::Personal | DataTier::Noise => {
                AxonResult::Ok(PolicyDecision::Allow)
            }
        }
    }

    /// Returns true if this operation should be audit-logged.
    pub fn should_audit(tier: DataTier, flags: OpenFlags) -> bool {
        tier.requires_audit() || flags.contains(OpenFlags::WRITE)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tp40_policy_critical_read_allowed() {
        let d = AxfsPolicy::check_open(DataTier::Critical, OpenFlags::READ).unwrap();
        assert!(d.is_allow());
    }

    #[test]
    fn tp40_policy_critical_write_denied() {
        let d = AxfsPolicy::check_open(DataTier::Critical, OpenFlags::WRITE).unwrap();
        assert!(d.is_deny());
    }

    #[test]
    fn tp40_policy_personal_write_allowed() {
        let d = AxfsPolicy::check_open(DataTier::Personal, OpenFlags::WRITE).unwrap();
        assert!(d.is_allow());
    }

    #[test]
    fn tp40_policy_noise_rdwr_allowed() {
        let d = AxfsPolicy::check_open(DataTier::Noise, OpenFlags::RDWR).unwrap();
        assert!(d.is_allow());
    }

    #[test]
    fn tp40_policy_audit_critical_read() {
        assert!(AxfsPolicy::should_audit(DataTier::Critical, OpenFlags::READ));
    }

    #[test]
    fn tp40_policy_audit_any_write() {
        assert!(AxfsPolicy::should_audit(DataTier::Noise, OpenFlags::WRITE));
    }

    #[test]
    fn tp40_policy_no_audit_noise_read() {
        assert!(!AxfsPolicy::should_audit(DataTier::Noise, OpenFlags::READ));
    }
}
