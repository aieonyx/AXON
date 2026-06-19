// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
//
// axonc — AXONYX sovereign compiler driver.
//
// P55 landmark: the ten sovereign pipeline stages are unified into
// a single compile() call. Determinism proven. Bootstrap documented.
//
// Stage 0 (axonc-rust): P45–P54 Rust bridges, active until axonc.ax
// self-compilation achieves sha256(B1) == sha256(B2) == sha256(B3).
//
// Bootstrap procedure documented in: BOOTSTRAP.md

pub mod driver;
pub mod version;

pub use driver::{compile, compile_elf, CompileError, CompileResult};
pub use version::{AXONC_VERSION, BOOTSTRAP_DATE, BOOTSTRAP_PHASES, SOVEREIGN_STACK};
