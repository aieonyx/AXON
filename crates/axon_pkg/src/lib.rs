// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
// axon_pkg P72 — Sovereign .axpkg package format
//
// .axpkg is the signed distribution unit for AXONYX apps on aiXos Phoenix.
// Same discipline as .iam: Ed25519 signature + sovereign hash before any
// execution. Verify-before-run is architecturally enforced — no bypass path.
//
// .axpkg wire format:
//   [0..4]   magic: b"AXPK"
//   [4]      version: u8 (1)
//   [5..9]   manifest_len: u32 LE
//   [9..N]   manifest JSON (AxpkgManifest)
//   [N..N+4] script_len: u32 LE
//   [N+4..M] script bytes (.ax source)
//   [M..M+32] content_hash: [u8; 32] (sovereign hash of manifest+script)
//   [M+32..M+96] signature: [u8; 64] (Ed25519 over content_hash)
//
// Capability declarations (v0): interpreter gates — deny by default.
// axon_interp rejects awp statements unless "awp" capability is declared.

pub mod manifest;
pub mod pack;
pub mod verify;
pub mod registry;
pub mod hash;
