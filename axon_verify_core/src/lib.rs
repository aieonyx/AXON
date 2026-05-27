//! # axon_verify_core
//!
//! The AXON constitutional verification kernel.
//!
//! ## Constitutional rules
//!
//! 1. Zero external dependencies — this crate depends only on `core`.
//! 2. Every public function is verified by Kani (bounded model checking).
//! 3. Why3/Z3 proofs are in `proofs/` — committed alongside source.
//! 4. The LLM is NEVER in the Trusted Computing Base.
//! 5. Any modification requires a new proof before merge.
//!
//! ## Scope
//!
//! - `@ensures` postcondition checking
//! - `@witnessed_by` witness validation (DWC)
//! - IBI (Immutability-by-Inference) enforcement
//! - QCC (Quorum Consensus) gate
//! - Constitutional invariant protection

#![no_std]
#![deny(unsafe_code)]  // No unsafe in the TCB — ever

pub mod checker;
pub mod contract;
pub mod enforcer;

pub use checker::{check_ensures, check_dwc, VerifyOutcome};
pub use contract::{
    BoundaryInvariant, Contract, EnsuresContract,
    InvariantTier, Witness, WitnessKind,
};
pub use enforcer::{enforce_ibi, validate_witness, EnforcementResult};
