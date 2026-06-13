// Copyright (c) 2026 Edison Lepiten / AIEONYX
//! AXFS data tier classification.
//!
//! Three tiers match EdisonDB's sovereignty model:
//!   Critical — encrypted at rest, strict access, audit-logged.
//!   Personal  — encrypted at rest, owner access only.
//!   Noise     — standard access, low-priority storage.
//!
//! Tier is determined at open time by path prefix or explicit annotation.
//! All tiers are equally encrypted at rest — the difference is access policy.

/// AXFS data tier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DataTier {
    /// Highest sensitivity — system keys, credentials, audit logs.
    Critical = 2,
    /// Personal user data — documents, configs, private files.
    Personal = 1,
    /// Low-sensitivity data — cache, temp files, public assets.
    Noise = 0,
}

impl DataTier {
    /// Classify a path by its prefix convention.
    ///
    /// - `/axon/critical/**` → Critical
    /// - `/axon/personal/**` → Personal
    /// - everything else    → Noise
    pub fn from_path(path: &str) -> Self {
        if path.starts_with("/axon/critical/") || path.starts_with("/axon/critical") {
            DataTier::Critical
        } else if path.starts_with("/axon/personal/") || path.starts_with("/axon/personal") {
            DataTier::Personal
        } else {
            DataTier::Noise
        }
    }

    /// Returns true if this tier requires audit logging on access.
    pub fn requires_audit(self) -> bool {
        matches!(self, DataTier::Critical)
    }

    /// Returns true if this tier requires encryption at rest.
    /// (All tiers do — this is here for explicitness.)
    pub const fn requires_encryption(self) -> bool { true }

    /// Human-readable tier name.
    pub const fn name(self) -> &'static str {
        match self {
            DataTier::Critical => "critical",
            DataTier::Personal => "personal",
            DataTier::Noise    => "noise",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tp40_tier_from_path_critical() {
        assert_eq!(DataTier::from_path("/axon/critical/keys/root.key"), DataTier::Critical);
        assert_eq!(DataTier::from_path("/axon/critical"), DataTier::Critical);
    }

    #[test]
    fn tp40_tier_from_path_personal() {
        assert_eq!(DataTier::from_path("/axon/personal/docs/note.txt"), DataTier::Personal);
    }

    #[test]
    fn tp40_tier_from_path_noise() {
        assert_eq!(DataTier::from_path("/tmp/cache.bin"), DataTier::Noise);
        assert_eq!(DataTier::from_path("/var/log/axon.log"), DataTier::Noise);
    }

    #[test]
    fn tp40_tier_ordering() {
        assert!(DataTier::Critical > DataTier::Personal);
        assert!(DataTier::Personal > DataTier::Noise);
    }

    #[test]
    fn tp40_tier_audit_requirement() {
        assert!(DataTier::Critical.requires_audit());
        assert!(!DataTier::Personal.requires_audit());
        assert!(!DataTier::Noise.requires_audit());
    }

    #[test]
    fn tp40_tier_encryption_always_required() {
        assert!(DataTier::Critical.requires_encryption());
        assert!(DataTier::Personal.requires_encryption());
        assert!(DataTier::Noise.requires_encryption());
    }

    #[test]
    fn tp40_tier_names() {
        assert_eq!(DataTier::Critical.name(), "critical");
        assert_eq!(DataTier::Personal.name(), "personal");
        assert_eq!(DataTier::Noise.name(),    "noise");
    }
}
