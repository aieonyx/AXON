// Copyright (c) 2026 Edison Lepiten / AIEONYX
//! Phase 41 integration tests — GENESIS root task bootstrap.

use axon_genesis::{BootInfo, CapRange, UntypedRegion, genesis_main, GenesisState, PdId};

fn standard_bootinfo() -> BootInfo {
    let mut bi = BootInfo::new(CapRange { start: 10, end: 256 }, 0x1000_0000);
    bi.add_untyped(UntypedRegion { cap: 20, paddr: 0x4000_0000, size_bits: 24, is_device: false });
    bi.add_untyped(UntypedRegion { cap: 21, paddr: 0x9000_0000, size_bits: 12, is_device: true  });
    bi
}

#[test]
fn p41_genesis_bootstrap_full() {
    let state = genesis_main(standard_bootinfo()).unwrap();
    assert!(state.bootstrapped);
    assert!(state.broker.grant_count() > 0);
}

#[test]
fn p41_genesis_heap_wired_to_largest_ram() {
    let state = genesis_main(standard_bootinfo()).unwrap();
    let bytes = state.phase3_wire_heap().unwrap();
    assert_eq!(bytes, 1 << 24);
}

#[test]
fn p41_genesis_axfs_wired() {
    let state = genesis_main(standard_bootinfo()).unwrap();
    assert!(state.phase4_wire_axfs().is_ok());
}

#[test]
fn p41_genesis_empty_bootinfo_fails() {
    let bi = BootInfo::new(CapRange { start: 10, end: 256 }, 0x1000_0000);
    assert!(genesis_main(bi).is_err());
}

#[test]
fn p41_genesis_broker_revoke_all_on_pd() {
    let mut state = genesis_main(standard_bootinfo()).unwrap();
    state.broker.revoke_all(PdId::ROOT, 1);
    assert_eq!(state.broker.grant_count(), 0);
}
