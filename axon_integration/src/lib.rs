//! # AXON Phase 6 Integration Test Suite
//!
//! 300+ tests verifying the complete Phase 6 stdlib stack:
//! axon_core → axon_alloc → axon_pal → axon_pal_linux → axon_std

// ── Module declarations ───────────────────────────────────────────────────────
#[cfg(test)] mod test_core;
#[cfg(test)] mod test_alloc;
#[cfg(test)] mod test_pal;
#[cfg(test)] mod test_verify;
#[cfg(test)] mod test_audit;
#[cfg(test)] mod test_ai;
#[cfg(test)] mod test_pipeline;
