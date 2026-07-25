// Copyright (c) 2026 Edison Lepiten / AIEONYX
// axon_pkg P72 — Package builder (pack + sign)

use crate::manifest::{AxpkgManifest, AXPKG_MAGIC, AXPKG_VERSION};
use crate::hash::{sovereign_hash, HASH_LEN};
use axon_crypto::ed25519::Ed25519KeyPair;

pub const SIG_LEN: usize = 64;

/// A built .axpkg ready for distribution.
#[derive(Debug, Clone)]
pub struct AxpkgFile {
    pub manifest: AxpkgManifest,
    pub script: Vec<u8>,
    pub content_hash: [u8; HASH_LEN],
    pub signature: [u8; SIG_LEN],
    pub signed: bool,
}

impl AxpkgFile {
    /// Serialize to .axpkg bytes.
    pub fn serialize(&self) -> Result<Vec<u8>, PackError> {
        let manifest_json = serde_json::to_string(&self.manifest)
            .map_err(|e| PackError::SerializeError(e.to_string()))?;
        let manifest_bytes = manifest_json.as_bytes();
        let manifest_len = manifest_bytes.len() as u32;
        let script_len = self.script.len() as u32;

        let mut out = Vec::new();
        out.extend_from_slice(&AXPKG_MAGIC);
        out.push(AXPKG_VERSION);
        out.extend_from_slice(&manifest_len.to_le_bytes());
        out.extend_from_slice(manifest_bytes);
        out.extend_from_slice(&script_len.to_le_bytes());
        out.extend_from_slice(&self.script);
        out.extend_from_slice(&self.content_hash);
        out.extend_from_slice(&self.signature);
        Ok(out)
    }

    pub fn size_bytes(&self) -> usize {
        9 + // magic(4) + version(1) + manifest_len(4)
        serde_json::to_string(&self.manifest).unwrap_or_default().len() +
        4 + // script_len
        self.script.len() +
        HASH_LEN + SIG_LEN
    }
}

/// Pack a .ax script into an unsigned .axpkg.
pub fn pack(manifest: AxpkgManifest, script: Vec<u8>) -> Result<AxpkgFile, PackError> {
    manifest.validate()
        .map_err(|e| PackError::ManifestError(e.to_string()))?;

    let manifest_json = serde_json::to_string(&manifest)
        .map_err(|e| PackError::SerializeError(e.to_string()))?;

    // Content hash = sovereign_hash(manifest_json + script)
    let mut to_hash = manifest_json.as_bytes().to_vec();
    to_hash.extend_from_slice(&script);
    let content_hash = sovereign_hash(&to_hash);

    Ok(AxpkgFile {
        manifest,
        script,
        content_hash,
        signature: [0u8; SIG_LEN], // unsigned stub
        signed: false,
    })
}

/// Sign a packed .axpkg with an Ed25519 keypair.
pub fn sign(pkg: &mut AxpkgFile, keypair: &Ed25519KeyPair) {
    // Update manifest fields first, then recompute hash over final manifest
    let pubkey_hex: String = keypair.public_key().to_bytes()
        .iter().map(|b| format!("{:02x}", b)).collect();
    pkg.manifest.signer_pubkey_hex = pubkey_hex;
    pkg.manifest.signed = true;

    // Recompute content hash over updated manifest + script
    let manifest_json = serde_json::to_string(&pkg.manifest).unwrap_or_default();
    let mut to_hash = manifest_json.as_bytes().to_vec();
    to_hash.extend_from_slice(&pkg.script);
    pkg.content_hash = crate::hash::sovereign_hash(&to_hash);

    // Sign the final hash
    let sig = keypair.sign(&pkg.content_hash);
    pkg.signature = sig;
    pkg.signed = true;
}

#[derive(Debug, Clone, PartialEq)]
pub enum PackError {
    ManifestError(String),
    SerializeError(String),
    InvalidScript(String),
}

impl std::fmt::Display for PackError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::ManifestError(s)  => write!(f, "manifest error: {}", s),
            Self::SerializeError(s) => write!(f, "serialize error: {}", s),
            Self::InvalidScript(s)  => write!(f, "invalid script: {}", s),
        }
    }
}
