// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
//
// Sovereign signing stub.
// At P48: deterministic stub using FNV-64 spread across 64 bytes.
// Full Ed25519 signing wired at P57 (axon_crypto).
// The .axsig section is reserved in the ELF layout now so P57 is a
// drop-in replacement — no layout change required.

use crate::error::{LinkError, LinkResult};
use axon_std_string::AxString;

/// 64-byte sovereign signature placeholder.
/// Real Ed25519 signature arrives at P57.
#[derive(Debug, Clone, PartialEq)]
pub struct SovereignSig(pub [u8; 64]);

/// FNV-64 hash spread across 64 bytes — deterministic stub.
/// Same input always produces same output — bootstrap verification safe.
pub fn sign_stub(data: &[u8]) -> SovereignSig {
    const FNV_OFFSET: u64 = 14695981039346656037;
    const FNV_PRIME:  u64 = 1099511628211;

    let mut hash = FNV_OFFSET;
    for &byte in data {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(FNV_PRIME);
    }

    // Spread hash across 64 bytes by re-hashing with position salt
    let mut sig = [0u8; 64];
    for i in 0..8u64 {
        let mut h = hash.wrapping_add(i.wrapping_mul(FNV_PRIME));
        h = h.wrapping_mul(FNV_PRIME);
        let bytes = h.to_le_bytes();
        sig[(i * 8) as usize..(i * 8 + 8) as usize].copy_from_slice(&bytes);
    }
    SovereignSig(sig)
}

/// Append a sovereign signature to a binary file as .axsig trailer.
/// Format: 4-byte magic `AXSG` + 64-byte signature.
pub fn sig_append(bin_path: &str, sig: &SovereignSig) -> LinkResult<()> {
    let mut trailer = Vec::with_capacity(68);
    trailer.extend_from_slice(b"AXSG");
    trailer.extend_from_slice(&sig.0);

    // Read existing binary
    let mut data = std::fs::read(bin_path).map_err(|e| {
        LinkError::SigningError(AxString::ax_from_str(&e.to_string()))
    })?;

    // Append trailer
    data.extend_from_slice(&trailer);

    std::fs::write(bin_path, &data).map_err(|e| {
        LinkError::SigningError(AxString::ax_from_str(&e.to_string()))
    })
}

/// Verify the .axsig trailer is present in a binary.
pub fn sig_verify_present(bin_path: &str) -> LinkResult<bool> {
    let data = std::fs::read(bin_path).map_err(|e| {
        LinkError::SigningError(AxString::ax_from_str(&e.to_string()))
    })?;
    if data.len() < 68 {
        return Ok(false);
    }
    let trailer_start = data.len() - 68;
    Ok(&data[trailer_start..trailer_start + 4] == b"AXSG")
}
