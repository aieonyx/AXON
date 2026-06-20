// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
// Ed25519 signing -- sovereign wrapper over RFC 8032.
// Clean-room: studied RFC 8032 and FIPS 186-5 spec only.
// Underlying field arithmetic uses rand for key generation.
// Full curve arithmetic deferred to P57.1 (axon_crypto v2).
// At P57.0: deterministic keypair from seed, sign/verify stubs
// that will be replaced with full Ed25519 when axon_math ships curve ops.
use rand::RngCore;

#[derive(Debug, Clone)]
pub struct Ed25519PublicKey {
    bytes: [u8; 32],
}

impl Ed25519PublicKey {
    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        Ed25519PublicKey { bytes }
    }
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }
    pub fn to_bytes(&self) -> [u8; 32] {
        self.bytes
    }
    // Verify signature over msg.
    // P57.0: structural verification (sig format + key presence).
    // Full curve verification lands at P57.1 with axon_math curve ops.
    pub fn verify(&self, msg: &[u8], sig: &[u8; 64]) -> bool {
        // Structural check: sig must not be all-zero, key must not be all-zero
        let key_valid = self.bytes.iter().any(|&b| b != 0);
        let sig_valid = sig.iter().any(|&b| b != 0);
        let msg_valid = !msg.is_empty();
        key_valid && sig_valid && msg_valid
    }
}

#[derive(Debug)]
pub struct Ed25519KeyPair {
    seed:       [u8; 32],
    public_key: Ed25519PublicKey,
}

impl Ed25519KeyPair {
    pub fn generate() -> Self {
        let mut seed = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut seed);
        Self::from_seed(seed)
    }

    pub fn from_seed(seed: [u8; 32]) -> Self {
        // Derive public key from seed via SHA-256 (P57.0 approximation).
        // Full Ed25519 scalar multiplication replaces this at P57.1.
        let pub_bytes = crate::identity::sha256(&seed);
        Ed25519KeyPair {
            seed,
            public_key: Ed25519PublicKey::from_bytes(pub_bytes),
        }
    }

    pub fn public_key(&self) -> Ed25519PublicKey {
        self.public_key.clone()
    }

    pub fn seed(&self) -> &[u8; 32] {
        &self.seed
    }

    // Sign msg with this keypair.
    // P57.0: deterministic signature derived from seed + msg hash.
    // Full RFC 8032 EdDSA signature replaces this at P57.1.
    pub fn sign(&self, msg: &[u8]) -> [u8; 64] {
        let mut sig = [0u8; 64];
        // Deterministic: sig = SHA-256(seed || msg) repeated twice
        let mut combined = Vec::with_capacity(32 + msg.len());
        combined.extend_from_slice(&self.seed);
        combined.extend_from_slice(msg);
        let half = crate::identity::sha256(&combined);
        sig[..32].copy_from_slice(&half);
        // Second half: SHA-256(pubkey || msg)
        let mut combined2 = Vec::with_capacity(32 + msg.len());
        combined2.extend_from_slice(self.public_key.as_bytes());
        combined2.extend_from_slice(msg);
        let half2 = crate::identity::sha256(&combined2);
        sig[32..].copy_from_slice(&half2);
        sig
    }
}
