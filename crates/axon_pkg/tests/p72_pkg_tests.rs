// Copyright (c) 2026 Edison Lepiten / AIEONYX
// axon_pkg P72 — Sovereign .axpkg tests (20 tests)

use axon_pkg::manifest::{AxpkgManifest, Capability, ManifestError, AXPKG_MAGIC};
use axon_pkg::pack::{pack, sign, AxpkgFile};
use axon_pkg::verify::{verify_bytes, VerifyResult, RejectReason};
use axon_pkg::registry::AxpkgRegistry;
use axon_pkg::hash::{sovereign_hash, sovereign_hash_hex};
use axon_crypto::ed25519::Ed25519KeyPair;

fn test_manifest() -> AxpkgManifest {
    AxpkgManifest::new(
        "sovereign-hello",
        "0.1.0",
        "Edison Lepiten / AIEONYX",
        "Hello sovereign world",
        "main.ax",
        vec![],
    )
}

fn test_script() -> Vec<u8> {
    b"print \"hello sovereign\"\n".to_vec()
}

fn test_keypair() -> Ed25519KeyPair {
    Ed25519KeyPair::from_seed([0x42u8; 32])
}

// ── T1: manifest validation — valid ──────────────────────────────────────────
#[test]
fn t1_manifest_valid() {
    assert!(test_manifest().validate().is_ok());
}

// ── T2: manifest validation — empty name ─────────────────────────────────────
#[test]
fn t2_manifest_empty_name() {
    let mut m = test_manifest();
    m.name = String::new();
    assert!(matches!(m.validate(), Err(ManifestError::EmptyName)));
}

// ── T3: manifest validation — bad entry extension ────────────────────────────
#[test]
fn t3_manifest_bad_entry() {
    let mut m = test_manifest();
    m.entry = "main.rs".to_string();
    assert!(matches!(m.validate(), Err(ManifestError::InvalidEntry(_))));
}

// ── T4: manifest validation — invalid name chars ─────────────────────────────
#[test]
fn t4_manifest_invalid_name() {
    let mut m = test_manifest();
    m.name = "My App!".to_string();
    assert!(matches!(m.validate(), Err(ManifestError::InvalidName(_))));
}

// ── T5: capability declarations ───────────────────────────────────────────────
#[test]
fn t5_capability_declarations() {
    let m = AxpkgManifest::new(
        "awp-app", "0.1.0", "Ed", "desc", "main.ax",
        vec![Capability::Awp, Capability::FsRead],
    );
    assert!(m.has_capability(&Capability::Awp));
    assert!(m.has_capability(&Capability::FsRead));
    assert!(!m.has_capability(&Capability::FsWrite));
}

// ── T6: capability roundtrip ──────────────────────────────────────────────────
#[test]
fn t6_capability_roundtrip() {
    for (s, cap) in [
        ("awp", Capability::Awp),
        ("fs:read", Capability::FsRead),
        ("fs:write", Capability::FsWrite),
    ] {
        assert_eq!(Capability::from_str(s), Some(cap.clone()));
        assert_eq!(cap.as_str(), s);
    }
}

// ── T7: pack produces AxpkgFile ───────────────────────────────────────────────
#[test]
fn t7_pack_basic() {
    let pkg = pack(test_manifest(), test_script()).unwrap();
    assert!(!pkg.signed);
    assert_eq!(pkg.script, test_script());
    assert!(!pkg.content_hash.iter().all(|&b| b == 0));
}

// ── T8: pack content hash is deterministic ────────────────────────────────────
#[test]
fn t8_pack_deterministic() {
    let p1 = pack(test_manifest(), test_script()).unwrap();
    let p2 = pack(test_manifest(), test_script()).unwrap();
    assert_eq!(p1.content_hash, p2.content_hash);
}

// ── T9: sign produces valid signature ────────────────────────────────────────
#[test]
fn t9_sign() {
    let mut pkg = pack(test_manifest(), test_script()).unwrap();
    assert!(!pkg.signed);
    sign(&mut pkg, &test_keypair());
    assert!(pkg.signed);
    assert!(!pkg.signature.iter().all(|&b| b == 0));
    assert_eq!(pkg.manifest.signer_pubkey_hex.len(), 64);
}

// ── T10: serialize produces valid magic ───────────────────────────────────────
#[test]
fn t10_serialize_magic() {
    let mut pkg = pack(test_manifest(), test_script()).unwrap();
    sign(&mut pkg, &test_keypair());
    let bytes = pkg.serialize().unwrap();
    assert_eq!(&bytes[0..4], &AXPKG_MAGIC);
    assert_eq!(bytes[4], 1u8); // version
}

// ── T11: verify_bytes accepts valid signed package ────────────────────────────
#[test]
fn t11_verify_signed_accepted() {
    let mut pkg = pack(test_manifest(), test_script()).unwrap();
    sign(&mut pkg, &test_keypair());
    let bytes = pkg.serialize().unwrap();
    let (restored, result) = verify_bytes(&bytes, true);
    assert_eq!(result, VerifyResult::Accepted);
    assert!(restored.is_some());
    assert_eq!(restored.unwrap().script, test_script());
}

// ── T12: verify_bytes rejects unsigned in production mode ────────────────────
#[test]
fn t12_verify_unsigned_rejected() {
    let pkg = pack(test_manifest(), test_script()).unwrap();
    let bytes = pkg.serialize().unwrap();
    let (_, result) = verify_bytes(&bytes, true); // require_signed=true
    assert_eq!(result, VerifyResult::Rejected(RejectReason::UnsignedPackage));
}

// ── T13: verify_bytes accepts unsigned in dev mode ────────────────────────────
#[test]
fn t13_verify_unsigned_dev_mode() {
    let pkg = pack(test_manifest(), test_script()).unwrap();
    let bytes = pkg.serialize().unwrap();
    let (restored, result) = verify_bytes(&bytes, false); // require_signed=false
    assert_eq!(result, VerifyResult::Accepted);
    assert!(restored.is_some());
}

// ── T14: verify_bytes detects content tampering ───────────────────────────────
#[test]
fn t14_verify_tampered_content() {
    let mut pkg = pack(test_manifest(), test_script()).unwrap();
    sign(&mut pkg, &test_keypair());
    let mut bytes = pkg.serialize().unwrap();
    // Corrupt a script byte
    let script_offset = 9 + serde_json::to_string(&pkg.manifest).unwrap().len() + 4;
    bytes[script_offset] ^= 0xFF;
    let (_, result) = verify_bytes(&bytes, true);
    assert!(matches!(result, VerifyResult::Rejected(RejectReason::ContentHashMismatch)));
}

// ── T15: verify_bytes detects bad magic ───────────────────────────────────────
#[test]
fn t15_verify_bad_magic() {
    let mut pkg = pack(test_manifest(), test_script()).unwrap();
    sign(&mut pkg, &test_keypair());
    let mut bytes = pkg.serialize().unwrap();
    bytes[0] = b'X';
    let (_, result) = verify_bytes(&bytes, true);
    assert!(matches!(result, VerifyResult::Rejected(RejectReason::InvalidMagic(_))));
}

// ── T16: registry register + lookup ──────────────────────────────────────────
#[test]
fn t16_registry_register() {
    let mut reg = AxpkgRegistry::new();
    let pkg = pack(test_manifest(), test_script()).unwrap();
    reg.register(&pkg.manifest, &pkg.content_hash, "/axfs/apps/sovereign-hello/");
    assert_eq!(reg.package_count(), 1);
    let entry = reg.lookup("sovereign-hello").unwrap();
    assert_eq!(entry.version, "0.1.0");
}

// ── T17: registry capability check ───────────────────────────────────────────
#[test]
fn t17_registry_capability() {
    let mut reg = AxpkgRegistry::new();
    let m = AxpkgManifest::new(
        "awp-app", "0.1.0", "Ed", "desc", "main.ax",
        vec![Capability::Awp],
    );
    let pkg = pack(m, test_script()).unwrap();
    reg.register(&pkg.manifest, &pkg.content_hash, "/axfs/apps/awp-app/");
    assert!(reg.has_capability("awp-app", &Capability::Awp));
    assert!(!reg.has_capability("awp-app", &Capability::FsWrite));
}

// ── T18: registry unregister ─────────────────────────────────────────────────
#[test]
fn t18_registry_unregister() {
    let mut reg = AxpkgRegistry::new();
    let pkg = pack(test_manifest(), test_script()).unwrap();
    reg.register(&pkg.manifest, &pkg.content_hash, "/axfs/apps/sovereign-hello/");
    assert!(reg.unregister("sovereign-hello"));
    assert_eq!(reg.package_count(), 0);
    assert!(reg.lookup("sovereign-hello").is_none());
}

// ── T19: sovereign_hash deterministic ────────────────────────────────────────
#[test]
fn t19_hash_deterministic() {
    let h1 = sovereign_hash(b"axonyx sovereign");
    let h2 = sovereign_hash(b"axonyx sovereign");
    assert_eq!(h1, h2);
    assert_ne!(h1, sovereign_hash(b"different"));
    assert_eq!(sovereign_hash_hex(b"test").len(), 64);
}

// ── T20: full sovereign app distribution pipeline ────────────────────────────
#[test]
fn t20_full_pipeline() {
    // Build a sovereign AWP app
    let manifest = AxpkgManifest::new(
        "sovereign-awp-ping",
        "0.1.0",
        "Edison Lepiten / AIEONYX",
        "Sends an AWP ping on aiXos Phoenix",
        "ping.ax",
        vec![Capability::Awp],
    );
    let script = b"print \"sovereign AWP ping\"\nawp ping\n".to_vec();

    // Pack
    let mut pkg = pack(manifest, script.clone()).unwrap();
    assert!(!pkg.signed);

    // Sign with sovereign keypair
    let keypair = Ed25519KeyPair::from_seed([0xABu8; 32]);
    sign(&mut pkg, &keypair);
    assert!(pkg.signed);

    // Serialize
    let bytes = pkg.serialize().unwrap();
    assert!(!bytes.is_empty());
    assert_eq!(&bytes[0..4], &AXPKG_MAGIC);

    // Verify (production mode — require signed)
    let (restored, result) = verify_bytes(&bytes, true);
    assert_eq!(result, VerifyResult::Accepted, "signed package must be accepted");
    let restored = restored.unwrap();
    assert_eq!(restored.script, script);
    assert!(restored.manifest.has_capability(&Capability::Awp));

    // Register in local registry
    let mut registry = AxpkgRegistry::new();
    registry.register(&restored.manifest, &restored.content_hash, "/axfs/apps/sovereign-awp-ping/");
    assert_eq!(registry.package_count(), 1);
    assert!(registry.has_capability("sovereign-awp-ping", &Capability::Awp));

    // Serialize registry
    let reg_json = registry.serialize();
    assert!(reg_json.contains("sovereign-awp-ping"));
    assert!(reg_json.contains("0.1.0"));
}
