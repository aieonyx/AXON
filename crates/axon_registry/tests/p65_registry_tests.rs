// Copyright (c) 2026 Edison Lepiten / AIEONYX
// P65 — axon_registry tests (20 tests)

use axon_registry::manifest::{Manifest, Version};
use axon_registry::hash::{sha256, sha256_hex};
use axon_registry::index::{PackageIndex, RegistryError};

fn v(major: u32, minor: u32, patch: u32) -> Version {
    Version::new(major, minor, patch)
}

fn make_manifest(name: &str, ver: Version) -> Manifest {
    Manifest::new(name, ver, "Test package", "Edison Lepiten <aieonyx.eu@gmail.com>")
}

// ── T1: Version parse ─────────────────────────────────────────────────────────
#[test]
fn t1_version_parse() {
    let v = Version::parse("1.2.3").unwrap();
    assert_eq!(v.major, 1);
    assert_eq!(v.minor, 2);
    assert_eq!(v.patch, 3);
}

// ── T2: Version to_string ─────────────────────────────────────────────────────
#[test]
fn t2_version_to_string() {
    assert_eq!(v(0, 65, 0).to_string(), "0.65.0");
}

// ── T3: Version ordering ──────────────────────────────────────────────────────
#[test]
fn t3_version_ordering() {
    assert!(v(1, 0, 0) > v(0, 9, 9));
    assert!(v(0, 2, 0) > v(0, 1, 9));
    assert!(v(0, 1, 1) > v(0, 1, 0));
    assert!(v(1, 0, 0) == v(1, 0, 0));
}

// ── T4: Version parse invalid ─────────────────────────────────────────────────
#[test]
fn t4_version_parse_invalid() {
    assert!(Version::parse("1.2").is_none());
    assert!(Version::parse("1.2.x").is_none());
    assert!(Version::parse("").is_none());
}

// ── T5: Manifest validate OK ──────────────────────────────────────────────────
#[test]
fn t5_manifest_validate_ok() {
    let m = make_manifest("axon_core", v(0, 1, 0));
    assert!(m.validate().is_ok());
}

// ── T6: Manifest validate empty name ─────────────────────────────────────────
#[test]
fn t6_manifest_validate_empty_name() {
    let m = make_manifest("", v(0, 1, 0));
    assert!(m.validate().is_err());
}

// ── T7: Manifest validate invalid name chars ──────────────────────────────────
#[test]
fn t7_manifest_validate_invalid_name() {
    let m = make_manifest("my-package", v(0, 1, 0));
    assert!(m.validate().is_err(), "hyphens not allowed in package names");
}

// ── T8: Manifest JSON roundtrip ───────────────────────────────────────────────
#[test]
fn t8_manifest_json_roundtrip() {
    let m = make_manifest("axon_std", v(0, 55, 0));
    let bytes = m.to_json();
    let m2 = Manifest::from_json(&bytes).unwrap();
    assert_eq!(m2.name, "axon_std");
    assert_eq!(m2.version, v(0, 55, 0));
}

// ── T9: SHA-256 known vector (empty string) ───────────────────────────────────
#[test]
fn t9_sha256_empty() {
    // SHA-256("") = e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855
    let h = sha256_hex(b"");
    assert_eq!(h, "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855");
}

// ── T10: SHA-256 known vector ("abc") ────────────────────────────────────────
#[test]
fn t10_sha256_abc() {
    // SHA-256("abc") = ba7816bf8f01cfea414140de5dae2ec73b00361bbef0469f492c347f
    let h = sha256_hex(b"abc");
    assert!(h.starts_with("ba7816bf"), "sha256('abc') prefix mismatch: {}", h);
}

// ── T11: SHA-256 deterministic ───────────────────────────────────────────────
#[test]
fn t11_sha256_deterministic() {
    let data = b"sovereign package content";
    assert_eq!(sha256_hex(data), sha256_hex(data));
}

// ── T12: Publish and resolve exact version ────────────────────────────────────
#[test]
fn t12_publish_resolve() {
    let mut idx = PackageIndex::new();
    let m = make_manifest("axon_net", v(0, 56, 0));
    let content = b"fn connect() -> i64 { 0 }".to_vec();
    idx.publish(m, content).unwrap();
    let entry = idx.resolve("axon_net", &v(0, 56, 0)).unwrap();
    assert_eq!(entry.manifest.name, "axon_net");
}

// ── T13: Publish auto-fills content hash ─────────────────────────────────────
#[test]
fn t13_publish_auto_hash() {
    let mut idx = PackageIndex::new();
    let m = make_manifest("axon_crypto", v(0, 57, 0));
    let content = b"crypto content".to_vec();
    idx.publish(m, content.clone()).unwrap();
    let entry = idx.resolve("axon_crypto", &v(0, 57, 0)).unwrap();
    let expected = sha256_hex(&content);
    assert_eq!(entry.manifest.content_hash, expected);
}

// ── T14: Duplicate publish returns error ──────────────────────────────────────
#[test]
fn t14_duplicate_publish() {
    let mut idx = PackageIndex::new();
    let content = b"content".to_vec();
    idx.publish(make_manifest("axon_gpu", v(0, 58, 0)), content.clone()).unwrap();
    let result = idx.publish(make_manifest("axon_gpu", v(0, 58, 0)), content);
    assert!(matches!(result, Err(RegistryError::AlreadyExists(_))));
}

// ── T15: Resolve non-existent returns None ────────────────────────────────────
#[test]
fn t15_resolve_not_found() {
    let idx = PackageIndex::new();
    assert!(idx.resolve("nonexistent", &v(1, 0, 0)).is_none());
}

// ── T16: resolve_latest returns highest version ───────────────────────────────
#[test]
fn t16_resolve_latest() {
    let mut idx = PackageIndex::new();
    let content = b"v".to_vec();
    idx.publish(make_manifest("axon_wasm", v(0, 61, 0)), content.clone()).unwrap();
    idx.publish(make_manifest("axon_wasm", v(0, 61, 1)), content.clone()).unwrap();
    idx.publish(make_manifest("axon_wasm", v(0, 60, 0)), content).unwrap();
    let latest = idx.resolve_latest("axon_wasm").unwrap();
    assert_eq!(latest.manifest.version, v(0, 61, 1));
}

// ── T17: list returns sorted keys ────────────────────────────────────────────
#[test]
fn t17_list_sorted() {
    let mut idx = PackageIndex::new();
    let c = b"x".to_vec();
    idx.publish(make_manifest("axon_std", v(0, 55, 0)), c.clone()).unwrap();
    idx.publish(make_manifest("axon_net", v(0, 56, 0)), c.clone()).unwrap();
    let list = idx.list();
    assert_eq!(list.len(), 2);
    // sorted: axon_net before axon_std
    assert!(list[0].starts_with("axon_net"));
    assert!(list[1].starts_with("axon_std"));
}

// ── T18: verify passes for intact package ────────────────────────────────────
#[test]
fn t18_verify_ok() {
    let mut idx = PackageIndex::new();
    let content = b"sovereign content bytes".to_vec();
    idx.publish(make_manifest("axon_layout", v(0, 60, 0)), content).unwrap();
    assert!(idx.verify("axon_layout", &v(0, 60, 0)).unwrap());
}

// ── T19: verify on unknown package returns Err ────────────────────────────────
#[test]
fn t19_verify_not_found() {
    let idx = PackageIndex::new();
    assert!(matches!(
        idx.verify("ghost", &v(1, 0, 0)),
        Err(RegistryError::NotFound(_))
    ));
}

// ── T20: publish with wrong hash returns HashMismatch ────────────────────────
#[test]
fn t20_publish_hash_mismatch() {
    let mut idx = PackageIndex::new();
    let mut m = make_manifest("axon_font", v(0, 62, 0));
    m.content_hash = "a".repeat(64); // wrong hash
    let content = b"font content".to_vec();
    let result = idx.publish(m, content);
    assert!(matches!(result, Err(RegistryError::HashMismatch { .. })));
}
