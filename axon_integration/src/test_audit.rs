//! Integration tests — axon::audit hash-chained trail.
use axon_std::audit::{
    AuditChain, ChainVerification, EventKind,
    MemorySink, AuditSink,
    ConsentGate, ConsentDecision, ConsentRequest, RequestKind,
};

fn ts() -> u64 { 1_000_000_000 }

// ── AuditChain ────────────────────────────────────────────────────────────────
#[test] fn audit_chain_empty_is_valid() {
    assert_eq!(AuditChain::new().verify(), ChainVerification::Empty);
}
#[test] fn audit_chain_single_event_valid() {
    let mut c = AuditChain::new();
    c.append(EventKind::Custom, "test", b"payload".to_vec(), ts());
    assert!(c.verify().is_valid());
}
#[test] fn audit_chain_100_events_valid() {
    let mut c = AuditChain::new();
    for i in 0u64..100 { c.append(EventKind::Custom, format!("event_{i}"), vec![], ts()+i); }
    assert_eq!(c.len(), 100); assert!(c.verify().is_valid());
}
#[test] fn audit_chain_ids_sequential() {
    let mut c = AuditChain::new();
    for _ in 0..5 { c.append(EventKind::Custom, "x", vec![], ts()); }
    for (i, e) in c.events().iter().enumerate() { assert_eq!(e.id, i as u64 + 1); }
}
#[test] fn audit_chain_genesis_zeros() {
    let mut c = AuditChain::new();
    c.append(EventKind::Postcondition, "first", vec![], ts());
    assert_eq!(c.events()[0].prev_hash, [0u8; 32]);
}
#[test] fn audit_chain_tamper_breaks_verification() {
    let mut c = AuditChain::new();
    c.append(EventKind::Custom, "a", vec![], ts());
    c.append(EventKind::Custom, "b", vec![], ts());
    c.events_mut()[0].label = "TAMPERED".to_string();
    assert!(!c.verify().is_valid());
}
#[test] fn audit_chain_tip_hash_changes() {
    let mut c = AuditChain::new();
    let t0 = c.tip_hash();
    c.append(EventKind::Custom, "x", vec![], ts());
    assert_ne!(t0, c.tip_hash());
}
#[test] fn audit_all_event_kinds() {
    let kinds = [EventKind::Postcondition, EventKind::InferenceCall,
        EventKind::UnsafeBlock, EventKind::CapabilityGrant,
        EventKind::ConsentRequest, EventKind::ConsentDecision,
        EventKind::QuorumVerification, EventKind::Custom];
    let mut c = AuditChain::new();
    for k in kinds { c.append(k, "x", vec![], ts()); }
    assert!(c.verify().is_valid());
}

// ── MemorySink ────────────────────────────────────────────────────────────────
#[test] fn audit_memory_sink_stores() {
    let mut s = MemorySink::new(); let mut c = AuditChain::new();
    let e = c.append(EventKind::Custom, "test", vec![], ts());
    s.emit(e).unwrap(); assert_eq!(s.len(), 1);
}
#[test] fn audit_memory_sink_clear() {
    let mut s = MemorySink::new(); let mut c = AuditChain::new();
    for _ in 0..5 { let e = c.append(EventKind::Custom, "x", vec![], ts()); s.emit(e).unwrap(); }
    assert_eq!(s.len(), 5); s.clear(); assert!(s.is_empty());
}

// ── ConsentGate (Sovereign Consent Doctrine) ──────────────────────────────────
#[test] fn audit_consent_grant_recorded() {
    let mut chain = AuditChain::new();
    let req = ConsentRequest::new(RequestKind::CapabilityGrant, "grant seL4 cap", 0.9);
    assert!(!req.suspicious);
    let d = { let mut g = ConsentGate::new(&mut chain); g.request(&req, ConsentDecision::Granted, ts()).unwrap() };
    assert!(d.allows_proceed()); assert_eq!(chain.len(), 2); assert!(chain.verify().is_valid());
}
#[test] fn audit_consent_suspicious_warns() {
    let req = ConsentRequest::new(RequestKind::SensitiveDataAccess, "risky op", 0.1);
    assert!(req.suspicious); assert!(req.warning.is_some());
    assert!(req.warning.as_ref().unwrap().contains("AXON"));
}
#[test] fn audit_consent_acknowledged_allows_proceed() {
    let mut chain = AuditChain::new();
    let req = ConsentRequest::new(RequestKind::Custom, "aware op", 0.2);
    let d = { let mut g = ConsentGate::new(&mut chain); g.request(&req, ConsentDecision::AcknowledgedAndGranted, ts()).unwrap() };
    assert!(d.allows_proceed()); assert!(chain.verify().is_valid());
}
#[test] fn audit_consent_denied_recorded() {
    let mut chain = AuditChain::new();
    let req = ConsentRequest::new(RequestKind::NetworkConnection, "connect", 0.5);
    let d = { let mut g = ConsentGate::new(&mut chain); g.request(&req, ConsentDecision::Denied, ts()).unwrap() };
    assert!(!d.allows_proceed()); assert!(chain.verify().is_valid());
}

// ── Full audit pipeline: verify → audit ──────────────────────────────────────
#[test] fn audit_verify_to_audit_pipeline() {
    use axon_std::verify::check_postcondition;
    let mut chain = AuditChain::new();
    let result = check_postcondition("result_positive", 42 > 0);
    assert!(result.is_ok());
    chain.append(EventKind::Postcondition, "result_positive", b"true".to_vec(), ts());
    assert!(chain.verify().is_valid());
}
