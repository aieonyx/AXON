//! End-to-end pipeline tests — verifies complete Phase 6 stack integration.
use axon_core::prelude::*;
use axon_alloc::prelude::*;
use axon_std::{audit::{AuditChain, EventKind, MemorySink, AuditSink},
               verify::{check_postcondition, DynamicWitness, QuorumGate}};
use axon_pal_linux::LinuxPal;
use axon_pal::traits::{PalFs, PalTime};
use axon_pal::types::{AxonPath, OpenFlags};

// ── Pipeline 1: computation → verify → audit ──────────────────────────────────
#[test] fn pipeline_computation_verify_audit() {
    // Simulate: AXON program computes, verifies @ensures, emits audit trail
    fn sovereign_add(a: AxonInt, b: AxonInt) -> AxonResult<AxonInt> {
        let result = a + b;
        match check_postcondition("result_equals_sum", result == a + b) {
            Ok(_)  => AxonResult::Ok(result),
            Err(e) => AxonResult::Err(AxonError::verification(e.label)),
        }
    }
    let r = sovereign_add(10, 32);
    assert_eq!(r, AxonResult::Ok(42));

    // Emit to audit chain
    let mut chain = AuditChain::new();
    chain.append(EventKind::Postcondition, "result_equals_sum", b"passed".to_vec(), 0);
    assert!(chain.verify().is_valid());
}

// ── Pipeline 2: PAL → alloc → verify ──────────────────────────────────────────
#[test] fn pipeline_pal_alloc_verify() {
    // Read process args via std, store in AxonVec, verify count
    let args: AxonVec<String> = std::env::args().collect();
    let count = args.len() as i64;
    let ok = check_postcondition("args_count_positive", count > 0);
    assert!(ok.is_ok());
}

// ── Pipeline 3: filesystem → alloc → audit ────────────────────────────────────
#[test] fn pipeline_fs_alloc_audit() {
    let path = AxonPath::new("/tmp/axon_pipeline_test");
    let _ = LinuxPal::remove(&path);

    // Write data
    let fd = LinuxPal::open(&path, OpenFlags::WRITE.or(OpenFlags::CREATE)).unwrap();
    use axon_pal::traits::PalIo;
    LinuxPal::write_all(fd, b"axon integration").unwrap();
    LinuxPal::close(fd).unwrap();

    // Verify size
    let st = LinuxPal::stat(&path).unwrap();
    assert_eq!(st.size, 16);

    // Store result in AxonHashMap and verify
    let mut results: AxonHashMap<&str, u64> = AxonHashMap::new();
    results.insert("file_size", st.size);
    let size = results["file_size"];
    check_postcondition("file_size_correct", size == 16).unwrap();

    // Audit the operation
    let mut chain = AuditChain::new();
    chain.append(EventKind::Postcondition, "file_size_correct",
                 size.to_le_bytes().to_vec(), 0);
    assert!(chain.verify().is_valid());

    LinuxPal::remove(&path).unwrap();
}

// ── Pipeline 4: quorum → audit → verify ──────────────────────────────────────
#[test] fn pipeline_quorum_audit_verify() {
    // Multi-witness verification with audit trail
    let mut chain = AuditChain::new();
    let mut gate = QuorumGate::new("sovereign_op_approved", 2);

    // Two independent witnesses
    gate.add_witness(DynamicWitness::security("sovereign_op_approved", true));
    gate.add_witness(DynamicWitness::postcondition("sovereign_op_approved", true));

    let result = gate.enforce();
    assert!(result.is_ok());

    // Record quorum in audit chain
    chain.append(EventKind::QuorumVerification, "sovereign_op_approved",
                 b"2 witnesses".to_vec(), 0);
    assert!(chain.verify().is_valid());
}

// ── Pipeline 5: alloc stress → verify → audit ─────────────────────────────────
#[test] fn pipeline_alloc_stress_verify_audit() {
    // Fill a large HashMap, verify invariants, audit summary
    let mut m: AxonHashMap<u64,u64> = AxonHashMap::with_capacity(500);
    for i in 0..500u64 { m.insert(i, i * i); }
    let sum: u64 = m.values().copied().sum();

    check_postcondition("map_len_correct", m.len() == 500).unwrap();
    check_postcondition("sum_positive", sum > 0).unwrap();

    let mut chain = AuditChain::new();
    chain.append(EventKind::Postcondition, "map_len_correct", b"500".to_vec(), 0);
    chain.append(EventKind::Postcondition, "sum_positive", sum.to_le_bytes().to_vec(), 0);
    assert!(chain.verify().is_valid()); assert_eq!(chain.len(), 2);
}

// ── Pipeline 6: time → verify → audit ────────────────────────────────────────
#[test] fn pipeline_time_verify_audit() {
    let t1 = LinuxPal::now_monotonic().unwrap();
    // Simulate some work
    let mut v: AxonVec<u64> = AxonVec::new();
    for i in 0..1000u64 { v.push(i * i); }
    let t2 = LinuxPal::now_monotonic().unwrap();

    check_postcondition("time_monotonic", t2 >= t1).unwrap();
    check_postcondition("work_done", v.len() == 1000).unwrap();

    let mut chain = AuditChain::new();
    chain.append(EventKind::Postcondition, "time_monotonic", vec![], 0);
    chain.append(EventKind::Postcondition, "work_done", b"1000".to_vec(), 0);
    assert!(chain.verify().is_valid());
}
