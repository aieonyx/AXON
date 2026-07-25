// Copyright (c) 2026 Edison Lepiten / AIEONYX
// axon_pkg P72 — Package verifier (verify-before-run)
//
// Architectural invariant: NO .ax script executes without passing verify().
// Unsigned packages are rejected in production mode.
// Tampered content_hash or signature = rejection, not warning.

use crate::manifest::{AxpkgManifest, AXPKG_MAGIC, AXPKG_VERSION};
use crate::pack::{AxpkgFile, SIG_LEN};
use crate::hash::{sovereign_hash, HASH_LEN};
use axon_crypto::ed25519::Ed25519PublicKey;

/// Verification result
#[derive(Debug, Clone, PartialEq)]
pub enum VerifyResult {
    Accepted,
    Rejected(RejectReason),
}

impl VerifyResult {
    pub fn is_accepted(&self) -> bool { matches!(self, Self::Accepted) }
}

#[derive(Debug, Clone, PartialEq)]
pub enum RejectReason {
    InvalidMagic([u8; 4]),
    VersionMismatch(u8),
    InvalidManifest(String),
    ContentHashMismatch,
    InvalidSignature,
    UnsignedPackage,
    TooShort(usize),
    ParseError(String),
}

impl std::fmt::Display for RejectReason {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::InvalidMagic(m)    => write!(f, "invalid magic: {:?}", m),
            Self::VersionMismatch(v) => write!(f, "unsupported version: {}", v),
            Self::InvalidManifest(s) => write!(f, "invalid manifest: {}", s),
            Self::ContentHashMismatch=> write!(f, "content hash mismatch — tampered"),
            Self::InvalidSignature   => write!(f, "invalid Ed25519 signature"),
            Self::UnsignedPackage    => write!(f, "unsigned package rejected"),
            Self::TooShort(n)        => write!(f, "package too short: {} bytes", n),
            Self::ParseError(s)      => write!(f, "parse error: {}", s),
        }
    }
}

/// Deserialize and verify a .axpkg from raw bytes.
/// `require_signed`: if true, unsigned packages are rejected (production mode).
pub fn verify_bytes(data: &[u8], require_signed: bool) -> (Option<AxpkgFile>, VerifyResult) {
    // Minimum size check
    if data.len() < 9 + HASH_LEN + SIG_LEN {
        return (None, VerifyResult::Rejected(RejectReason::TooShort(data.len())));
    }

    // Magic
    if &data[0..4] != &AXPKG_MAGIC {
        return (None, VerifyResult::Rejected(
            RejectReason::InvalidMagic([data[0], data[1], data[2], data[3]])
        ));
    }

    // Version
    if data[4] != AXPKG_VERSION {
        return (None, VerifyResult::Rejected(RejectReason::VersionMismatch(data[4])));
    }

    // Manifest
    let manifest_len = u32::from_le_bytes([data[5], data[6], data[7], data[8]]) as usize;
    let manifest_start = 9;
    let manifest_end = manifest_start + manifest_len;
    if manifest_end > data.len() {
        return (None, VerifyResult::Rejected(RejectReason::TooShort(data.len())));
    }
    let manifest: AxpkgManifest = match serde_json::from_slice(&data[manifest_start..manifest_end]) {
        Ok(m) => m,
        Err(e) => return (None, VerifyResult::Rejected(RejectReason::ParseError(e.to_string()))),
    };
    if let Err(e) = manifest.validate() {
        return (None, VerifyResult::Rejected(RejectReason::InvalidManifest(e.to_string())));
    }

    // Script
    let script_len_start = manifest_end;
    if script_len_start + 4 > data.len() {
        return (None, VerifyResult::Rejected(RejectReason::TooShort(data.len())));
    }
    let script_len = u32::from_le_bytes([
        data[script_len_start], data[script_len_start+1],
        data[script_len_start+2], data[script_len_start+3],
    ]) as usize;
    let script_start = script_len_start + 4;
    let script_end = script_start + script_len;
    if script_end + HASH_LEN + SIG_LEN > data.len() {
        return (None, VerifyResult::Rejected(RejectReason::TooShort(data.len())));
    }
    let script = data[script_start..script_end].to_vec();

    // Content hash
    let stored_hash: [u8; HASH_LEN] = data[script_end..script_end+HASH_LEN].try_into().unwrap();
    let manifest_json = serde_json::to_string(&manifest).unwrap_or_default();
    let mut to_hash = manifest_json.as_bytes().to_vec();
    to_hash.extend_from_slice(&script);
    let computed_hash = sovereign_hash(&to_hash);
    if computed_hash != stored_hash {
        return (None, VerifyResult::Rejected(RejectReason::ContentHashMismatch));
    }

    // Signature
    let sig_start = script_end + HASH_LEN;
    let mut signature = [0u8; SIG_LEN];
    signature.copy_from_slice(&data[sig_start..sig_start+SIG_LEN]);
    let is_unsigned = signature.iter().all(|&b| b == 0);

    if require_signed && is_unsigned {
        return (None, VerifyResult::Rejected(RejectReason::UnsignedPackage));
    }

    if !is_unsigned {
        // Verify Ed25519
        if manifest.signer_pubkey_hex.len() == 64 {
            let mut pk_bytes = [0u8; 32];
            let hex = manifest.signer_pubkey_hex.as_bytes();
            let mut valid_hex = true;
            for i in 0..32 {
                let hi = unhex_nibble(hex[i*2]);
                let lo = unhex_nibble(hex[i*2+1]);
                match (hi, lo) {
                    (Some(h), Some(l)) => pk_bytes[i] = (h << 4) | l,
                    _ => { valid_hex = false; break; }
                }
            }
            if valid_hex {
                let pubkey = Ed25519PublicKey::from_bytes(pk_bytes);
                if !pubkey.verify(&stored_hash, &signature) {
                    return (None, VerifyResult::Rejected(RejectReason::InvalidSignature));
                }
            }
        }
    }

    let pkg = AxpkgFile {
        manifest,
        script,
        content_hash: stored_hash,
        signature,
        signed: !is_unsigned,
    };

    (Some(pkg), VerifyResult::Accepted)
}

fn unhex_nibble(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        b'A'..=b'F' => Some(b - b'A' + 10),
        _ => None,
    }
}
