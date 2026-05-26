//! Integration tests — axon::verify formal verification runtime.
use axon_std::verify::{
    check_postcondition, cache_hit_rate, cache_stats, cache_invalidate,
    ContractCache, DynamicWitness, WitnessKind,
    DependentGuard, QuorumGate, QuorumResult,
};

// ── check_postcondition ───────────────────────────────────────────────────────
#[test] fn verify_postcondition_ok() { assert!(check_postcondition("always_true", true).is_ok()); }
#[test] fn verify_postcondition_err() { assert!(check_postcondition("always_false", false).is_err()); }
#[test] fn verify_error_label_preserved() {
    let e = check_postcondition("my_label", false).unwrap_err();
    assert_eq!(e.label, "my_label");
}
#[test] fn verify_error_e411_in_display() {
    let e = check_postcondition("x", false).unwrap_err();
    assert!(format!("{e}").contains("E411"));
}

// ── ContractCache ─────────────────────────────────────────────────────────────
#[test] fn verify_cache_miss_then_hit() {
    let mut c = ContractCache::new();
    c.record(0xAA, true); // miss
    c.record(0xAA, true); // hit
    assert_eq!(c.stats().misses, 1); assert_eq!(c.stats().hits, 1);
}
#[test] fn verify_cache_violation_count() {
    let mut c = ContractCache::new();
    c.record(0x01, true); c.record(0x02, false); c.record(0x03, false);
    assert_eq!(c.stats().violations, 2);
}
#[test] fn verify_cache_hit_rate_target() {
    // Simulate: 1 miss + 2 hits = 66.7% → approaching 70% target on second build
    let mut c = ContractCache::new();
    c.record(0xFF, true); c.record(0xFF, true); c.record(0xFF, true);
    let rate = c.stats().hit_rate();
    assert!(rate > 60.0);
}
#[test] fn verify_cache_invalidate() {
    let mut c = ContractCache::new();
    c.record(0xAA, true); assert_eq!(c.len(), 1);
    c.invalidate(); assert!(c.is_empty());
}
#[test] fn verify_cache_global_stats_accessible() {
    let stats = cache_stats(); let _ = stats.hit_rate();
}

// ── DynamicWitness (DWC) ─────────────────────────────────────────────────────
#[test] fn verify_witness_postcondition_valid() {
    let w = DynamicWitness::postcondition("result_nonneg", true);
    assert!(w.is_valid()); assert_eq!(w.kind, WitnessKind::Postcondition);
}
#[test] fn verify_witness_invalid_on_false() {
    assert!(!DynamicWitness::postcondition("x", false).is_valid());
}
#[test] fn verify_witness_hash_deterministic() {
    let h1 = DynamicWitness::postcondition("label", true).hash;
    let h2 = DynamicWitness::postcondition("label", true).hash;
    assert_eq!(h1, h2);
}
#[test] fn verify_witness_security_kind() {
    let w = DynamicWitness::security("cap_grant", true);
    assert_eq!(w.kind, WitnessKind::SecurityProperty);
}

// ── DependentGuard (DVG) ─────────────────────────────────────────────────────
#[test] fn verify_guard_blocks_before_satisfy() {
    let g = DependentGuard::new("output", "input_validated");
    assert!(g.check_access().is_err());
}
#[test] fn verify_guard_allows_after_satisfy() {
    let mut g = DependentGuard::new("output", "input_validated");
    g.satisfy_dependency(); assert!(g.check_access().is_ok());
}

// ── QuorumGate (QCC) ─────────────────────────────────────────────────────────
#[test] fn verify_quorum_requires_n_witnesses() {
    let g = QuorumGate::new("cap_grant", 3);
    assert_eq!(g.check(), QuorumResult::Pending { have: 0, need: 3 });
}
#[test] fn verify_quorum_reached() {
    let mut g = QuorumGate::new("cap_grant", 2);
    g.add_witness(DynamicWitness::postcondition("x", true));
    g.add_witness(DynamicWitness::postcondition("x", true));
    assert!(g.check().is_reached()); assert!(g.enforce().is_ok());
}
#[test] fn verify_quorum_fails_on_dissent() {
    let mut g = QuorumGate::new("x", 2);
    g.add_witness(DynamicWitness::postcondition("x", true));
    g.add_witness(DynamicWitness::postcondition("x", false));
    assert!(matches!(g.check(), QuorumResult::Failed { .. }));
}

// ── Full pipeline: postcondition → cache → witness ────────────────────────────
#[test] fn verify_full_pipeline() {
    // Simulate an @ensures check feeding cache and producing a witness
    let result = check_postcondition("result_bounded", 42 > 0);
    assert!(result.is_ok());
    let w = DynamicWitness::postcondition("result_bounded", true);
    let mut gate = QuorumGate::new("result_bounded", 1);
    gate.add_witness(w);
    assert!(gate.enforce().is_ok());
    assert!(cache_stats().lookups > 0);
}
