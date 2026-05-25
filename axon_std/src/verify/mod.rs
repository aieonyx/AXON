//! # axon::verify
//!
//! AXON formal verification runtime module.
//!
//! Bridges compile-time `@ensures` annotations to runtime enforcement.
//! The LLM proposes. The developer approves. This module enforces.
//!
//! ## Architecture
//!
//! ```text
//! @ensures result >= 0          ← annotation in AXON source
//!      ↓
//! axon::verify::check_postcondition("result >= 0", result >= 0)
//!      ↓
//! ContractCache::lookup(fn_hash) ← Bio DNA: Epigenetic Memory
//!      ↓ (cache miss)
//! DynamicWitness::generate()     ← P6+ DWC
//!      ↓
//! VerifyResult::Ok(witness)      ← stored in cache
//! ```
//!
//! ## Bio DNA features
//!
//! - **Contract Cache** (Epigenetic Memory) — verified functions cached
//!   by hash. Unchanged functions skip re-verification. ≥70% hit rate
//!   on second build.
//! - **Annotation Preservation** — @ensures, @ai.intent, @immortal_invariant
//!   survive the transpiler intact.
//!
//! ## P6+ features integrated
//!
//! - DWC (Dynamic Witness Contracts) — runtime witness generation
//! - DVG (Dependent Variable Guards) — variable dependency enforcement
//! - IBI (Immutability-by-Inference) — inferred const markers
//! - QCC (Quorum Consensus Contracts) — N-witness enforcement

pub mod cache;
pub mod check;
pub mod guard;
pub mod quorum;
pub mod witness;

pub use check::{check_postcondition, VerificationError, VerifyResult};
pub use cache::{ContractCache, CachedVerification, CacheStats};
pub use witness::{DynamicWitness, WitnessKind};
pub use guard::{DependentGuard, GuardViolation};
pub use quorum::{QuorumGate, QuorumResult};

use std::fmt;

/// A verification event record — emitted when a postcondition is checked.
#[derive(Debug, Clone)]
pub struct VerificationEvent {
    /// The label identifying which @ensures was checked.
    pub label:     &'static str,
    /// Whether the postcondition held.
    pub passed:    bool,
    /// The function hash used for cache keying.
    pub fn_hash:   u64,
}
