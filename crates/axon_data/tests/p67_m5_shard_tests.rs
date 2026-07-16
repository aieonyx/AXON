// Copyright (c) 2026 Edison Lepiten / AIEONYX
// axon_data P67-M5 — Corpus hash registry + .axd shard tests (20 tests)
// IAM FLAGSHIP: every training shard is provable

use axon_data::shard::{
    AxdShard, CorpusRegistry, ShardError,
    sovereign_hash, sovereign_hash_hex,
    build_shards,
    AXD_MAGIC, AXD_VERSION, AXD_HEADER_LEN, HASH_LEN,
};
use axon_data::batch::{pack_sequences, TokenizedEntry, SEQ_LEN};
use axon_data::types::DataTier;

fn make_seq(n_tokens: usize, tier: DataTier) -> axon_data::batch::TrainingSequence {
    let entries = vec![TokenizedEntry::new(
        (100..100+n_tokens as u32).collect(), tier, "test"
    )];
    pack_sequences(&entries).into_iter().next().unwrap()
}

// ── T1: sovereign_hash is 32 bytes ───────────────────────────────────────────
#[test]
fn t1_hash_length() {
    let h = sovereign_hash(b"sovereign");
    assert_eq!(h.len(), HASH_LEN);
}

// ── T2: sovereign_hash is deterministic ──────────────────────────────────────
#[test]
fn t2_hash_deterministic() {
    assert_eq!(sovereign_hash(b"wisdom"), sovereign_hash(b"wisdom"));
}

// ── T3: sovereign_hash different input → different hash ───────────────────────
#[test]
fn t3_hash_different() {
    assert_ne!(sovereign_hash(b"wisdom"), sovereign_hash(b"knowledge"));
}

// ── T4: sovereign_hash_hex is 64 chars ───────────────────────────────────────
#[test]
fn t4_hash_hex_length() {
    assert_eq!(sovereign_hash_hex(b"test").len(), 64);
}

// ── T5: AxdShard from_sequence builds correctly ───────────────────────────────
#[test]
fn t5_shard_from_sequence() {
    let seq = make_seq(100, DataTier::Critical);
    let shard = AxdShard::from_sequence(&seq, 0, "iam-corpus");
    assert_eq!(shard.shard_index, 0);
    assert_eq!(shard.tier, DataTier::Critical);
    assert_eq!(shard.token_count(), SEQ_LEN);
}

// ── T6: shard content hash verifies ──────────────────────────────────────────
#[test]
fn t6_shard_content_hash() {
    let seq = make_seq(50, DataTier::Noise);
    let shard = AxdShard::from_sequence(&seq, 0, "test");
    assert!(shard.verify_content(), "content hash must verify");
}

// ── T7: shard serialize produces correct magic ────────────────────────────────
#[test]
fn t7_shard_magic() {
    let seq = make_seq(10, DataTier::Noise);
    let shard = AxdShard::from_sequence(&seq, 0, "test");
    let bytes = shard.serialize();
    assert_eq!(&bytes[0..4], &AXD_MAGIC);
    assert_eq!(bytes[4], AXD_VERSION);
}

// ── T8: shard serialized size correct ────────────────────────────────────────
#[test]
fn t8_shard_size() {
    let seq = make_seq(10, DataTier::Noise);
    let shard = AxdShard::from_sequence(&seq, 0, "test");
    let bytes = shard.serialize();
    // Header + SEQ_LEN tokens * 4 bytes + 32 footer
    let expected = AXD_HEADER_LEN + SEQ_LEN * 4 + HASH_LEN;
    assert_eq!(bytes.len(), expected);
}

// ── T9: shard deserialize roundtrip ──────────────────────────────────────────
#[test]
fn t9_shard_roundtrip() {
    let seq = make_seq(100, DataTier::Personal);
    let shard = AxdShard::from_sequence(&seq, 7, "wisdom-corpus");
    let bytes = shard.serialize();
    let restored = AxdShard::deserialize(&bytes).unwrap();
    assert_eq!(restored.shard_index, 7);
    assert_eq!(restored.tier, DataTier::Personal);
    assert_eq!(restored.token_count(), SEQ_LEN);
    assert_eq!(restored.content_hash, shard.content_hash);
}

// ── T10: shard deserialize detects bad magic ─────────────────────────────────
#[test]
fn t10_shard_bad_magic() {
    let seq = make_seq(10, DataTier::Noise);
    let shard = AxdShard::from_sequence(&seq, 0, "test");
    let mut bytes = shard.serialize();
    bytes[0] = b'X'; // corrupt magic
    assert!(matches!(AxdShard::deserialize(&bytes), Err(ShardError::InvalidMagic(_))));
}

// ── T11: shard deserialize detects hash mismatch ─────────────────────────────
#[test]
fn t11_shard_hash_mismatch() {
    let seq = make_seq(10, DataTier::Noise);
    let shard = AxdShard::from_sequence(&seq, 0, "test");
    let mut bytes = shard.serialize();
    // Corrupt a token in the body
    let corrupt_pos = AXD_HEADER_LEN + 4;
    bytes[corrupt_pos] ^= 0xFF;
    assert!(matches!(AxdShard::deserialize(&bytes), Err(ShardError::HashMismatch)));
}

// ── T12: shard tier byte correct ─────────────────────────────────────────────
#[test]
fn t12_shard_tier_bytes() {
    for (tier, expected_byte) in [
        (DataTier::Critical, 0u8),
        (DataTier::Personal, 1u8),
        (DataTier::Noise,    2u8),
    ] {
        let seq = make_seq(10, tier);
        let shard = AxdShard::from_sequence(&seq, 0, "test");
        let bytes = shard.serialize();
        assert_eq!(bytes[5], expected_byte);
    }
}

// ── T13: CorpusRegistry registers shards ─────────────────────────────────────
#[test]
fn t13_registry_register() {
    let mut reg = CorpusRegistry::new("iam-seed");
    let seq = make_seq(100, DataTier::Critical);
    let shard = AxdShard::from_sequence(&seq, 0, "iam-seed");
    reg.register(&shard);
    assert_eq!(reg.shard_count(), 1);
    assert_eq!(reg.total_tokens, SEQ_LEN);
}

// ── T14: CorpusRegistry verify_shard passes ──────────────────────────────────
#[test]
fn t14_registry_verify_pass() {
    let mut reg = CorpusRegistry::new("test");
    let seq = make_seq(50, DataTier::Noise);
    let shard = AxdShard::from_sequence(&seq, 0, "test");
    reg.register(&shard);
    assert!(reg.verify_shard(&shard));
}

// ── T15: CorpusRegistry verify_shard fails on tampered shard ─────────────────
#[test]
fn t15_registry_verify_fail() {
    let mut reg = CorpusRegistry::new("test");
    let seq = make_seq(50, DataTier::Noise);
    let mut shard = AxdShard::from_sequence(&seq, 0, "test");
    reg.register(&shard);
    // Tamper: flip a byte in content_hash
    shard.content_hash[0] ^= 0xFF;
    assert!(!reg.verify_shard(&shard));
}

// ── T16: CorpusRegistry tier_count ───────────────────────────────────────────
#[test]
fn t16_registry_tier_count() {
    let mut reg = CorpusRegistry::new("test");
    for tier in [DataTier::Critical, DataTier::Noise, DataTier::Noise] {
        let seq = make_seq(10, tier.clone());
        let shard = AxdShard::from_sequence(&seq, reg.shard_count() as u32, "test");
        reg.register(&shard);
    }
    assert_eq!(reg.tier_count(&DataTier::Critical), 1);
    assert_eq!(reg.tier_count(&DataTier::Noise), 2);
}

// ── T17: CorpusRegistry manifest_hash is deterministic ───────────────────────
#[test]
fn t17_manifest_hash_deterministic() {
    let mut reg = CorpusRegistry::new("test");
    let seq = make_seq(10, DataTier::Noise);
    let shard = AxdShard::from_sequence(&seq, 0, "test");
    reg.register(&shard);
    assert_eq!(reg.manifest_hash(), reg.manifest_hash());
}

// ── T18: CorpusRegistry serialize produces .axreg ────────────────────────────
#[test]
fn t18_registry_serialize() {
    let mut reg = CorpusRegistry::new("iam-corpus");
    let seq = make_seq(10, DataTier::Critical);
    let shard = AxdShard::from_sequence(&seq, 0, "iam-corpus");
    reg.register(&shard);
    let s = reg.serialize();
    assert!(s.starts_with("#axreg v1"), "must start with #axreg v1");
    assert!(s.contains("iam-corpus"));
    assert!(s.contains("critical"));
    assert!(s.contains("shard 0"));
}

// ── T19: build_shards produces correct shard count ───────────────────────────
#[test]
fn t19_build_shards() {
    let entries: Vec<TokenizedEntry> = (0..5)
        .map(|i| TokenizedEntry::new(
            (i*10..i*10+10).collect(), DataTier::Noise, "test"
        ))
        .collect();
    let seqs = pack_sequences(&entries);
    let seq_count = seqs.len();
    let (shards, registry) = build_shards(&seqs, "test-corpus");
    assert_eq!(shards.len(), seq_count);
    assert_eq!(registry.shard_count(), seq_count);
}

// ── T20: full IAM corpus provenance pipeline ─────────────────────────────────
#[test]
fn t20_full_provenance_pipeline() {
    // Build training sequences
    let entries = vec![
        TokenizedEntry::new((0..200).collect(), DataTier::Critical, "kjv"),
        TokenizedEntry::new((200..400).collect(), DataTier::Critical, "stoics"),
        TokenizedEntry::new((400..500).collect(), DataTier::Personal, "user-notes"),
        TokenizedEntry::new((500..600).collect(), DataTier::Noise,    "wiki"),
    ];
    let seqs = pack_sequences(&entries);
    assert!(!seqs.is_empty());

    // Build shards and registry
    let (shards, registry) = build_shards(&seqs, "iam-seed-v0.1");
    assert_eq!(shards.len(), registry.shard_count());

    // Every shard verifies content hash
    for shard in &shards {
        assert!(shard.verify_content(), "shard {} content must verify", shard.shard_index);
        assert!(registry.verify_shard(shard), "shard {} must be in registry", shard.shard_index);
    }

    // Serialize all shards and deserialize back
    for shard in &shards {
        let bytes = shard.serialize();
        let restored = AxdShard::deserialize(&bytes)
            .expect("shard must deserialize cleanly");
        assert_eq!(restored.shard_index, shard.shard_index);
        assert_eq!(restored.content_hash, shard.content_hash);
        assert_eq!(restored.token_count(), shard.token_count());
    }

    // Manifest hash is stable
    let h1 = registry.manifest_hash();
    let h2 = registry.manifest_hash();
    assert_eq!(h1, h2, "manifest hash must be stable");

    // Registry serializes to .axreg
    let axreg = registry.serialize();
    assert!(axreg.contains("iam-seed-v0.1"));
    assert!(axreg.contains("#axreg v1"));
}
