// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
//
// axonc version and bootstrap metadata.
// Immutable after v0.55-bootstrap tag.

/// Compiler version string.
pub const AXONC_VERSION: &str = "0.55.0-bootstrap";

/// Date the bootstrap checkpoint was cut.
pub const BOOTSTRAP_DATE: &str = "2026-06-19";

/// The ten sovereign pipeline crates (P45–P54) that compose axonc.
pub const SOVEREIGN_STACK: &str =
    "axon_std_io    ·axon_std_string    ·axon_build    ·axon_link    ·axon_lex    ·axon_parse    ·axon_hir    ·axon_infer    ·axon_codegen    ·axon_native";

/// Phase range covered by the Rust bridge (stage 0).
pub const BOOTSTRAP_PHASES: &str = "P45-P54";
