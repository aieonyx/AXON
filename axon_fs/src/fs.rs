// Copyright (c) 2026 Edison Lepiten / AIEONYX
//! AXFS — sovereign file system facade.
//!
//! Wraps PalFs with tier classification, policy enforcement, and audit hooks.
//! The underlying storage is provided by any PalFs implementation.

use axon_core::prelude::*;
use axon_pal::traits::PalFs;
use axon_pal::types::{AxonPath, FileStat, OpenFlags, RawFd};
use crate::tier::DataTier;
use crate::policy::{AxfsPolicy, PolicyDecision};

/// An open AXFS file handle — wraps RawFd with tier metadata.
#[derive(Debug)]
pub struct AxfsHandle {
    pub fd:    RawFd,
    pub tier:  DataTier,
    pub flags: OpenFlags,
}

impl AxfsHandle {
    pub fn is_valid(&self) -> bool { !self.fd.is_invalid() }
}

/// AXFS sovereign file system — generic over any PalFs backend.
pub struct Axfs<P: PalFs> {
    _pal: core::marker::PhantomData<P>,
}

impl<P: PalFs> Axfs<P> {
    pub const fn new() -> Self { Self { _pal: core::marker::PhantomData } }

    /// Open a file with tier-aware policy enforcement.
    pub fn open(path: &AxonPath, flags: OpenFlags) -> AxonResult<AxfsHandle> {
        let tier = DataTier::from_path(path.as_str());
        // Policy gate
        let decision = axon_try!(AxfsPolicy::check_open(tier, flags));
        if let PolicyDecision::Deny(reason) = decision {
            return AxonResult::Err(AxonError::permission_denied(reason));
        }
        // Audit hook — P41 wires this to the audit chain
        if AxfsPolicy::should_audit(tier, flags) {
            Self::audit_log(path, flags, tier);
        }
        let fd = axon_try!(P::open(path, flags));
        AxonResult::Ok(AxfsHandle { fd, tier, flags })
    }

    /// Close a file handle.
    pub fn close(handle: AxfsHandle) -> AxonResult<()> {
        P::close(handle.fd)
    }

    /// Stat a path — returns FileStat with tier metadata.
    pub fn stat(path: &AxonPath) -> AxonResult<(FileStat, DataTier)> {
        let tier = DataTier::from_path(path.as_str());
        let stat = axon_try!(P::stat(path));
        AxonResult::Ok((stat, tier))
    }

    /// Create a directory.
    pub fn mkdir(path: &AxonPath, mode: u32) -> AxonResult<()> {
        P::mkdir(path, mode)
    }

    /// Remove a file or directory — policy-gated on tier.
    pub fn remove(path: &AxonPath) -> AxonResult<()> {
        let tier = DataTier::from_path(path.as_str());
        // Critical files require explicit capability to delete.
        let decision = axon_try!(AxfsPolicy::check_open(tier, OpenFlags::WRITE));
        if let PolicyDecision::Deny(reason) = decision {
            return AxonResult::Err(AxonError::permission_denied(reason));
        }
        P::remove(path)
    }

    /// Rename/move a path — policy-gated on source and destination tier.
    pub fn rename(from: &AxonPath, to: &AxonPath) -> AxonResult<()> {
        // Gate on both source and destination tier.
        for path in &[from, to] {
            let tier = DataTier::from_path(path.as_str());
            let decision = axon_try!(AxfsPolicy::check_open(tier, OpenFlags::WRITE));
            if let PolicyDecision::Deny(reason) = decision {
                return AxonResult::Err(AxonError::permission_denied(reason));
            }
        }
        P::rename(from, to)
    }

    /// Check existence.
    pub fn exists(path: &AxonPath) -> bool {
        P::exists(path)
    }

    /// Tier of a path without opening it.
    pub fn tier_of(path: &AxonPath) -> DataTier {
        DataTier::from_path(path.as_str())
    }

    /// Audit log hook — P41 replaces this with hash-chained audit chain.
    fn audit_log(_path: &AxonPath, _flags: OpenFlags, _tier: DataTier) {
        // Stub — wired to axon_verify_core audit chain in P41
    }
}

impl<P: PalFs> Default for Axfs<P> {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tp40_axfs_tier_of_paths() {
        assert_eq!(Axfs::<axon_pal::stub::StubPal>::tier_of(
            &AxonPath::new("/axon/critical/key")), DataTier::Critical);
        assert_eq!(Axfs::<axon_pal::stub::StubPal>::tier_of(
            &AxonPath::new("/axon/personal/doc")), DataTier::Personal);
        assert_eq!(Axfs::<axon_pal::stub::StubPal>::tier_of(
            &AxonPath::new("/tmp/cache")), DataTier::Noise);
    }

    #[test]
    fn tp40_axfs_open_critical_write_denied() {
        type AxfsLinux = Axfs<axon_pal::stub::StubPal>;
        let r = AxfsLinux::open(
            &AxonPath::new("/axon/critical/secret"),
            OpenFlags::WRITE,
        );
        assert!(r.is_err());
    }
}
