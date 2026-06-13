//! axon_sel4 — Sovereign seL4 Syscall Wrappers
//! Copyright (c) 2026 Edison Lepiten / AIEONYX
//! Apache-2.0
//!
//! Provides AXON-native wrappers for all seL4 syscalls.
//! On aarch64-sel4 target: delegates to P23 inline asm! intrinsics.
//! On host: type-safe stubs for unit testing and development.

pub mod ipc;
pub mod cap;
pub mod mem;
pub mod types;
pub mod irq;
