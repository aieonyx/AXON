// Copyright (c) 2026 Edison Lepiten / AIEONYX
//! GENESIS — root task entry point and sovereignty bootstrap.
//!
//! GENESIS is the first process seL4 hands control to after boot.
//! It enforces the AXON sovereignty semantics:
//!   1. Parse BootInfo — discover untyped memory and initial caps
//!   2. Bootstrap the Capability Broker
//!   3. Wire the sovereign heap (axon_alloc) to untyped memory
//!   4. Wire AXFS tier enforcement
//!   5. Spawn Protection Domains for each AXON service
//!   6. Enter the main dispatch loop
//!
//! On aarch64-seL4: this is the actual entry point (_start → genesis_main).
//! On host: genesis_main() is a simulation for integration testing.

use axon_core::prelude::*;
use crate::bootinfo::BootInfo;
use crate::capability::{CapabilityBroker, PdId, CapType};

/// GENESIS system state — holds all sovereignty-critical state.
pub struct GenesisState {
    /// Capability broker — mediates all cap grants.
    pub broker:   CapabilityBroker,
    /// Boot info from seL4 kernel.
    pub bootinfo: BootInfo,
    /// Whether GENESIS has completed bootstrap.
    pub bootstrapped: bool,
}

impl GenesisState {
    pub fn new(bootinfo: BootInfo) -> Self {
        Self {
            broker: CapabilityBroker::new(),
            bootinfo,
            bootstrapped: false,
        }
    }

    /// Phase 1: Parse BootInfo and validate system resources.
    pub fn phase1_parse_bootinfo(&self) -> AxonResult<()> {
        if self.bootinfo.untyped_count == 0 {
            return AxonResult::Err(AxonError::invalid_state("no untyped regions in BootInfo"));
        }
        if self.bootinfo.empty.is_empty() {
            return AxonResult::Err(AxonError::invalid_state("no empty CNode slots"));
        }
        AxonResult::Ok(())
    }

    /// Phase 2: Bootstrap the Capability Broker with initial caps.
    pub fn phase2_bootstrap_broker(&mut self) -> AxonResult<()> {
        // Grant root PD access to IRQ control (slot 4 per seL4 convention)
        if self.broker.grant(PdId::ROOT, 4, CapType::IrqHandler, 4, 1).is_err() {
            return AxonResult::Err(AxonError::invalid_state("broker: IRQ grant failed"));
        }
        AxonResult::Ok(())
    }

    /// Phase 3: Wire sovereign heap to largest untyped region.
    pub fn phase3_wire_heap(&self) -> AxonResult<u64> {
        // Find largest non-device untyped region
        let largest = self.bootinfo.untyped_regions()
            .filter(|r| !r.is_device)
            .max_by_key(|r| r.size_bits);
        match largest {
            Some(r) => AxonResult::Ok(r.size_bytes()),
            None    => AxonResult::Err(AxonError::not_found("no RAM untyped region")),
        }
    }

    /// Phase 4: Wire AXFS tier enforcement — mark critical paths.
    pub fn phase4_wire_axfs(&self) -> AxonResult<()> {
        // Stub — AXFS DataTier::from_path already active
        // P42: wire audit chain to hash-chained log here
        AxonResult::Ok(())
    }

    /// Full bootstrap sequence.
    pub fn bootstrap(&mut self) -> AxonResult<()> {
        axon_try!(self.phase1_parse_bootinfo());
        axon_try!(self.phase2_bootstrap_broker());
        axon_try!(self.phase3_wire_heap());
        axon_try!(self.phase4_wire_axfs());
        self.bootstrapped = true;
        AxonResult::Ok(())
    }
}

/// GENESIS main entry point — called by _start on aarch64-seL4.
/// On host: called directly by integration tests.
pub fn genesis_main(bootinfo: BootInfo) -> AxonResult<GenesisState> {
    let mut state = GenesisState::new(bootinfo);
    axon_try!(state.bootstrap());
    AxonResult::Ok(state)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bootinfo::BootInfo;

    fn make_bootinfo() -> BootInfo {
        let mut bi = BootInfo::new(CapRange { start: 10, end: 256 }, 0x1000_0000);
        bi.add_untyped(UntypedRegion {
            cap: 20, paddr: 0x4000_0000, size_bits: 24, is_device: false,
        });
        bi.add_untyped(UntypedRegion {
            cap: 21, paddr: 0x9000_0000, size_bits: 12, is_device: true,
        });
        bi
    }

    #[test]
    fn tp41_genesis_bootstrap_succeeds() {
        let bi = make_bootinfo();
        let result = genesis_main(bi);
        assert!(result.is_ok());
        assert!(result.unwrap().bootstrapped);
    }

    #[test]
    fn tp41_genesis_phase1_no_untyped_fails() {
        let bi = BootInfo::new(CapRange { start: 10, end: 256 }, 0x1000_0000);
        let state = GenesisState::new(bi);
        assert!(state.phase1_parse_bootinfo().is_err());
    }

    #[test]
    fn tp41_genesis_phase1_no_slots_fails() {
        let mut bi = BootInfo::new(CapRange { start: 10, end: 10 }, 0x1000_0000);
        bi.add_untyped(UntypedRegion { cap: 20, paddr: 0, size_bits: 20, is_device: false });
        let state = GenesisState::new(bi);
        assert!(state.phase1_parse_bootinfo().is_err());
    }

    #[test]
    fn tp41_genesis_phase3_wire_heap() {
        let bi = make_bootinfo();
        let state = GenesisState::new(bi);
        let bytes = state.phase3_wire_heap().unwrap();
        assert_eq!(bytes, 1 << 24); // 16MB
    }

    #[test]
    fn tp41_genesis_broker_grant_count_after_bootstrap() {
        let bi = make_bootinfo();
        let state = genesis_main(bi).unwrap();
        assert!(state.broker.grant_count() > 0);
    }
}
