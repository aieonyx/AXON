// Copyright (c) 2026 Edison Lepiten / AIEONYX
//! GENESIS — AXON seL4 root task: capability bootstrap and sovereignty enforcement.
//!
//! GENESIS is the first process seL4 hands control to after boot.
//! It owns the Capability Broker, BootInfo parsing, and the
//! sovereignty bootstrap sequence.
#![no_std]
#![allow(missing_docs)]
extern crate axon_core;

pub mod bootinfo;
pub mod capability;
pub mod genesis;

pub use bootinfo::{BootInfo, CapRange, UntypedRegion};
pub use capability::{CapabilityBroker, PdId, CapType, BrokerError};
pub use genesis::{GenesisState, genesis_main};
