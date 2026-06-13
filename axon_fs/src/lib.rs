// Copyright (c) 2026 Edison Lepiten / AIEONYX
//! AXFS — sovereign file system layer.
//!
//! Provides tier-aware file access (Critical/Personal/Noise),
//! policy enforcement, and audit hooks over any PalFs backend.
#![no_std]
#![allow(missing_docs)]
extern crate axon_core;

pub mod fs;
pub mod policy;
pub mod tier;

pub use fs::{Axfs, AxfsHandle};
pub use policy::{AxfsPolicy, PolicyDecision};
pub use tier::DataTier;
