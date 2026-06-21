// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
// axon_crypto -- sovereign cryptographic primitives.
// P57.0: Ed25519 identity, X25519 key exchange, ChaCha20 encryption, SHA-256.
// P57.1: Full curve arithmetic replaces approximations.
pub mod chacha20;
pub mod ed25519;
pub mod identity;
pub mod x25519;
pub use chacha20::{chacha20_encrypt, chacha20_decrypt, ChaCha20};
pub use ed25519::{Ed25519KeyPair, Ed25519PublicKey};
pub use identity::{SovereignIdentity, fingerprint_of, sha256};
pub use x25519::{X25519SecretKey, X25519PublicKey, derive_session_key};
pub mod sha512;
pub mod field;
pub mod curve;
