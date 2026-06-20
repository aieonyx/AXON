// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
// X25519 key exchange -- sovereign wrapper.
// Clean-room: studied RFC 7748 specification only. No code copied.
// P57.0: DH simulation using SHA-256 key derivation.
// Full Curve25519 scalar multiplication lands at P57.1.
use rand::RngCore;

#[derive(Debug, Clone)]
pub struct X25519PublicKey {
    bytes: [u8; 32],
}

impl X25519PublicKey {
    pub fn from_bytes(bytes: [u8; 32]) -> Self { X25519PublicKey { bytes } }
    pub fn as_bytes(&self) -> &[u8] { &self.bytes }
    pub fn to_bytes(&self) -> [u8; 32] { self.bytes }
}

#[derive(Debug)]
pub struct X25519SecretKey {
    bytes: [u8; 32],
}

impl X25519SecretKey {
    pub fn generate() -> Self {
        let mut bytes = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut bytes);
        // Clamp per RFC 7748
        bytes[0]  &= 248;
        bytes[31] &= 127;
        bytes[31] |= 64;
        X25519SecretKey { bytes }
    }

    pub fn from_bytes(mut bytes: [u8; 32]) -> Self {
        bytes[0]  &= 248;
        bytes[31] &= 127;
        bytes[31] |= 64;
        X25519SecretKey { bytes }
    }

    // Derive public key from secret.
    // P57.0: SHA-256(secret) as public key approximation.
    // Full basepoint multiplication replaces this at P57.1.
    pub fn public_key(&self) -> X25519PublicKey {
        let pub_bytes = crate::identity::sha256(&self.bytes);
        X25519PublicKey { bytes: pub_bytes }
    }

    // Diffie-Hellman shared secret.
    // P57.0: SHA-256(secret || peer_public) as shared secret approximation.
    // Full Montgomery ladder replaces this at P57.1.
    pub fn diffie_hellman(&self, peer: &X25519PublicKey) -> [u8; 32] {
        let mut combined = [0u8; 64];
        combined[..32].copy_from_slice(&self.bytes);
        combined[32..].copy_from_slice(&peer.bytes);
        crate::identity::sha256(&combined)
    }
}

// Derive a symmetric encryption key from a DH shared secret.
// HKDF-SHA256 approximation: SHA-256(shared || "axon_key_v1")
pub fn derive_session_key(shared_secret: &[u8; 32]) -> [u8; 32] {
    let label = b"axon_key_v1";
    let mut input = Vec::with_capacity(32 + label.len());
    input.extend_from_slice(shared_secret);
    input.extend_from_slice(label);
    crate::identity::sha256(&input)
}
