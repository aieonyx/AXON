// Copyright (c) 2026 Edison Lepiten / AIEONYX
//! Phase 40 integration tests — AXFS sovereign file system layer.

use axon_fs::{Axfs, DataTier, AxfsPolicy, PolicyDecision};
use axon_pal::types::{AxonPath, OpenFlags};
use axon_pal_linux::LinuxPal;

type AxfsLinux = Axfs<LinuxPal>;

#[test]
fn p40_axfs_tier_classification() {
    assert_eq!(AxfsLinux::tier_of(&AxonPath::new("/axon/critical/key")),  DataTier::Critical);
    assert_eq!(AxfsLinux::tier_of(&AxonPath::new("/axon/personal/doc")),  DataTier::Personal);
    assert_eq!(AxfsLinux::tier_of(&AxonPath::new("/tmp/cache.bin")),      DataTier::Noise);
}

#[test]
fn p40_axfs_open_noise_write_allowed() {
    let path = AxonPath::new("/tmp/axon_axfs_test_noise.txt");
    let h = AxfsLinux::open(&path, OpenFlags::WRITE.or(OpenFlags::CREATE).or(OpenFlags::TRUNCATE)).unwrap();
    assert!(h.is_valid());
    let tier = h.tier;
    assert_eq!(tier, DataTier::Noise);
    AxfsLinux::close(h).unwrap();
    AxfsLinux::remove(&path).unwrap();
}

#[test]
fn p40_axfs_open_critical_write_denied() {
    let path = AxonPath::new("/axon/critical/secret.key");
    let r = AxfsLinux::open(&path, OpenFlags::WRITE);
    assert!(r.is_err());
}

#[test]
fn p40_axfs_stat_with_tier() {
    let path = AxonPath::new("/tmp/axon_axfs_stat_test.txt");
    let h = AxfsLinux::open(&path, OpenFlags::WRITE.or(OpenFlags::CREATE).or(OpenFlags::TRUNCATE)).unwrap();
    AxfsLinux::close(h).unwrap();
    let (stat, tier) = AxfsLinux::stat(&path).unwrap();
    assert!(stat.is_file);
    assert_eq!(tier, DataTier::Noise);
    AxfsLinux::remove(&path).unwrap();
}

#[test]
fn p40_axfs_mkdir_remove() {
    let path = AxonPath::new("/tmp/axon_axfs_mkdir_test");
    let _ = AxfsLinux::remove(&path);
    AxfsLinux::mkdir(&path, 0o755).unwrap();
    assert!(AxfsLinux::exists(&path));
    AxfsLinux::remove(&path).unwrap();
    assert!(!AxfsLinux::exists(&path));
}

#[test]
fn p40_axfs_policy_audit_flags() {
    assert!(AxfsPolicy::should_audit(DataTier::Critical, OpenFlags::READ));
    assert!(AxfsPolicy::should_audit(DataTier::Noise,    OpenFlags::WRITE));
    assert!(!AxfsPolicy::should_audit(DataTier::Noise,   OpenFlags::READ));
}

#[test]
fn p40_axfs_tier_ordering() {
    assert!(DataTier::Critical > DataTier::Personal);
    assert!(DataTier::Personal > DataTier::Noise);
}
