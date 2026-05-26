//! # AXON Phase 6 Integration Test Suite
//!
//! 300+ tests verifying the complete Phase 6 stdlib stack:
//! axon_core → axon_alloc → axon_pal → axon_pal_linux → axon_std

// ── Module declarations ───────────────────────────────────────────────────────
mod test_core;
mod test_alloc;
mod test_pal;
mod test_verify;
mod test_audit;
mod test_ai;
mod test_pipeline;
