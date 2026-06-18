// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
//
// axon_awp_ffi — AXON AWP Verifier FFI Bridge
//
// Exposes a C-compatible boundary so Onyxia (Tauri/Rust) can call into AXON
// for sovereign AWP packet verification without coupling to the AXON compiler
// internals.
//
// Architecture:
//   Onyxia (awp_handler.rs)
//     └─ axon_verify_awp_packet()   [extern "C", this file]
//          └─ verify_stub()          [deterministic stub — AXON-STUB-001]
//               └─ [future] real P45 AWP mesh verifier when axon-live feature enabled
//
// AXON-STUB-001: verification logic is a deterministic protocol-aware stub.
// Replace with P45 AWP mesh verifier once axon_awp_mesh crate is stabilised.
// Feature flag: --features axon-live

/// Five-layer ARPi verification result.
/// Mirrors the ARPi bar layer order in Onyxia chrome.
#[repr(C)]
pub struct AxonVerifyResult {
    /// L1 — Schema: packet structure valid
    pub l1_schema: bool,
    /// L2 — Identity: sovereign identity present
    pub l2_identity: bool,
    /// L3 — Mutual Auth: handshake verified
    pub l3_auth: bool,
    /// L4 — Scope: capability scope within bounds
    pub l4_scope: bool,
    /// L5 — Anomaly: no anomalous patterns detected
    pub l5_anomaly: bool,
    /// Overall verdict: all five layers passed
    pub verified: bool,
    /// Human-readable status (null-terminated, max 127 bytes)
    pub status: [u8; 128],
}

impl AxonVerifyResult {
    fn sovereign(msg: &str) -> Self {
        let mut status = [0u8; 128];
        let bytes = msg.as_bytes();
        let len = bytes.len().min(127);
        status[..len].copy_from_slice(&bytes[..len]);
        Self {
            l1_schema: true,
            l2_identity: true,
            l3_auth: true,
            l4_scope: true,
            l5_anomaly: true,
            verified: true,
            status,
        }
    }

    fn legacy(msg: &str) -> Self {
        let mut status = [0u8; 128];
        let bytes = msg.as_bytes();
        let len = bytes.len().min(127);
        status[..len].copy_from_slice(&bytes[..len]);
        Self {
            l1_schema: true,
            l2_identity: false,
            l3_auth: false,
            l4_scope: false,
            l5_anomaly: true,
            verified: false,
            status,
        }
    }

    fn failed(msg: &str) -> Self {
        let mut status = [0u8; 128];
        let bytes = msg.as_bytes();
        let len = bytes.len().min(127);
        status[..len].copy_from_slice(&bytes[..len]);
        Self {
            l1_schema: false,
            l2_identity: false,
            l3_auth: false,
            l4_scope: false,
            l5_anomaly: false,
            verified: false,
            status,
        }
    }
}

/// Verify an AWP packet against the five-layer ARPi stack.
///
/// # Safety
/// `packet` must be a valid pointer to `len` bytes, or null.
/// Returns a stack-allocated `AxonVerifyResult` — no heap allocation.
///
/// # AXON-STUB-001
/// Current implementation is a deterministic protocol-aware stub.
/// The stub inspects packet prefix bytes to distinguish AWP / HTTPS / empty
/// packets and returns appropriate layer states without cryptographic verification.
/// Replace with real P45 mesh verifier when `axon-live` feature is enabled.
#[no_mangle]
pub extern "C" fn axon_verify_awp_packet(
    packet: *const u8,
    len: usize,
) -> AxonVerifyResult {
    #[cfg(feature = "axon-live")]
    {
        // AXON-LIVE: delegate to real P45 AWP mesh verifier
        // TODO: call axon_awp_mesh::verify(packet, len) once P45 is stable
        let _ = (packet, len);
        return AxonVerifyResult::failed("axon-live: P45 verifier not yet linked");
    }

    #[cfg(not(feature = "axon-live"))]
    {
        // AXON-STUB-001: deterministic protocol-aware stub
        verify_stub(packet, len)
    }
}

/// Stub verifier — inspects packet bytes to determine protocol type.
/// AWP packets: all five layers verified (sovereign).
/// HTTPS packets: L1 + L5 only (legacy connection).
/// Empty / null: all layers failed.
#[cfg(not(feature = "axon-live"))]
fn verify_stub(packet: *const u8, len: usize) -> AxonVerifyResult {
    if packet.is_null() || len == 0 {
        return AxonVerifyResult::failed("null packet");
    }

    // SAFETY: caller guarantees packet points to len valid bytes
    let bytes = unsafe { std::slice::from_raw_parts(packet, len.min(8)) };

    // AWP magic prefix: b"AWP/1.0" (0x41 0x57 0x50 0x2F 0x31 0x2E 0x30)
    if bytes.len() >= 7 && &bytes[..7] == b"AWP/1.0" {
        return AxonVerifyResult::sovereign("AWP/1.0 sovereign packet verified");
    }

    // awp:// URI prefix used by Onyxia internal pages
    if bytes.len() >= 6 && &bytes[..6] == b"awp://" {
        return AxonVerifyResult::sovereign("AWP sovereign internal page");
    }

    // HTTPS legacy connection
    if bytes.len() >= 5 && &bytes[..5] == b"https" {
        return AxonVerifyResult::legacy("HTTPS legacy connection — L1/L5 only");
    }

    // HTTP (unencrypted)
    if bytes.len() >= 4 && &bytes[..4] == b"http" {
        let mut result = AxonVerifyResult::failed("HTTP unencrypted — all layers failed");
        result.l1_schema = true; // schema is valid, just insecure
        return result;
    }

    AxonVerifyResult::failed("unknown protocol")
}

/// Rust-friendly wrapper — takes a URL string and returns verification result.
/// Called by Onyxia's awp_handler.rs via direct Rust linkage (not C FFI).
pub fn verify_url(url: &str) -> AxonVerifyResult {
    let bytes = url.as_bytes();
    axon_verify_awp_packet(bytes.as_ptr(), bytes.len())
}

/// Return a human-readable status string from a result.
pub fn result_status(r: &AxonVerifyResult) -> &str {
    let end = r.status.iter().position(|&b| b == 0).unwrap_or(127);
    std::str::from_utf8(&r.status[..end]).unwrap_or("unknown")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_awp_sovereign() {
        let url = "awp://aegis";
        let r = verify_url(url);
        assert!(r.l1_schema);
        assert!(r.l2_identity);
        assert!(r.l3_auth);
        assert!(r.l4_scope);
        assert!(r.l5_anomaly);
        assert!(r.verified);
    }

    #[test]
    fn test_https_legacy() {
        let url = "https://example.com";
        let r = verify_url(url);
        assert!(r.l1_schema);
        assert!(!r.l2_identity);
        assert!(!r.l3_auth);
        assert!(!r.l4_scope);
        assert!(r.l5_anomaly);
        assert!(!r.verified);
    }

    #[test]
    fn test_http_insecure() {
        let url = "http://example.com";
        let r = verify_url(url);
        assert!(r.l1_schema);
        assert!(!r.l2_identity);
        assert!(!r.verified);
    }

    #[test]
    fn test_null_packet() {
        let r = axon_verify_awp_packet(std::ptr::null(), 0);
        assert!(!r.verified);
        assert!(!r.l1_schema);
    }

    #[test]
    fn test_awp_magic_prefix() {
        let packet = b"AWP/1.0\x00some-payload";
        let r = axon_verify_awp_packet(packet.as_ptr(), packet.len());
        assert!(r.verified);
        assert_eq!(result_status(&r), "AWP/1.0 sovereign packet verified");
    }
}
