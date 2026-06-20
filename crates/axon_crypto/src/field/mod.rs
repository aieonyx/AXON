// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
// field/mod.rs -- GF(2^255-19) field arithmetic module.
// M1 of P57.1: field element type and all operations.

pub mod fe25519;
pub mod fe_ops;

pub use fe25519::Fe25519;
