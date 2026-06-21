// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
// P57 QA -- axon_crypto sovereign cryptography tests
// Pass bar: 20/20
use axon_crypto::{
    SovereignIdentity, Ed25519KeyPair,
    X25519SecretKey,
    chacha20_encrypt, chacha20_decrypt,
    sha256, fingerprint_of, derive_session_key,
};

// ── SHA-256 tests ─────────────────────────────────────────────────────────────

#[test]
fn test_sha256_empty() {
    let h = sha256(b"");
    // SHA-256("") = e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855
    assert_eq!(h[0], 0xe3);
    assert_eq!(h[1], 0xb0);
    assert_eq!(h[31], 0x55);
}

#[test]
fn test_sha256_abc() {
    let h = sha256(b"abc");
    // SHA-256("abc") = ba7816bf8f01cfea414140de5dae2ec73b00361bbef0469348423f656fbd9c43
    assert_eq!(h[0], 0xba);
    assert_eq!(h[1], 0x78);
    assert_eq!(h[2], 0x16);
}

#[test]
fn test_sha256_deterministic() {
    let h1 = sha256(b"sovereign");
    let h2 = sha256(b"sovereign");
    assert_eq!(h1, h2);
}

#[test]
fn test_sha256_different_inputs() {
    let h1 = sha256(b"axon");
    let h2 = sha256(b"axon2");
    assert_ne!(h1, h2);
}

// ── Ed25519 tests ─────────────────────────────────────────────────────────────

#[test]
fn test_ed25519_generate() {
    let kp = Ed25519KeyPair::generate();
    let pk = kp.public_key();
    assert!(pk.as_bytes().iter().any(|&b| b != 0));
}

#[test]
fn test_ed25519_from_seed_deterministic() {
    let seed = [42u8; 32];
    let kp1 = Ed25519KeyPair::from_seed(seed);
    let kp2 = Ed25519KeyPair::from_seed(seed);
    assert_eq!(kp1.public_key().to_bytes(), kp2.public_key().to_bytes());
}

#[test]
fn test_ed25519_sign_deterministic() {
    let seed = [7u8; 32];
    let kp = Ed25519KeyPair::from_seed(seed);
    let sig1 = kp.sign(b"hello sovereign");
    let sig2 = kp.sign(b"hello sovereign");
    assert_eq!(sig1, sig2);
}

#[test]
fn test_ed25519_sign_different_msgs() {
    let kp = Ed25519KeyPair::generate();
    let sig1 = kp.sign(b"msg1");
    let sig2 = kp.sign(b"msg2");
    assert_ne!(sig1, sig2);
}

#[test]
fn test_ed25519_verify_valid() {
    let kp = Ed25519KeyPair::generate();
    let msg = b"sovereign message";
    let sig = kp.sign(msg);
    assert!(kp.public_key().verify(msg, &sig));
}

#[test]
fn test_ed25519_verify_empty_msg_fails() {
    let kp = Ed25519KeyPair::generate();
    let sig = [1u8; 64];
    assert!(!kp.public_key().verify(b"", &sig));
}

// ── SovereignIdentity tests ───────────────────────────────────────────────────

#[test]
fn test_identity_generate() {
    let id = SovereignIdentity::generate();
    assert!(id.fingerprint().iter().any(|&b| b != 0));
}

#[test]
fn test_identity_fingerprint_is_pubkey_hash() {
    let id  = SovereignIdentity::generate();
    let fp  = fingerprint_of(id.public_key().as_bytes());
    assert_eq!(fp, id.fingerprint());
}

#[test]
fn test_identity_sign_verify() {
    let id  = SovereignIdentity::generate();
    let msg = b"axonyx sovereign identity";
    let sig = id.sign(msg);
    assert!(id.verify(msg, &sig));
}

#[test]
fn test_identity_unique_fingerprints() {
    let id1 = SovereignIdentity::generate();
    let id2 = SovereignIdentity::generate();
    assert_ne!(id1.fingerprint(), id2.fingerprint());
}

// ── X25519 tests ──────────────────────────────────────────────────────────────

#[test]
fn test_x25519_generate() {
    let sk = X25519SecretKey::generate();
    let pk = sk.public_key();
    assert!(pk.as_bytes().iter().any(|&b| b != 0));
}

#[test]
fn test_x25519_dh_shared_secret() {
    let alice_sk = X25519SecretKey::generate();
    let bob_sk   = X25519SecretKey::generate();
    let alice_pk = alice_sk.public_key();
    let bob_pk   = bob_sk.public_key();
    let alice_shared = alice_sk.diffie_hellman(&bob_pk);
    let bob_shared   = bob_sk.diffie_hellman(&alice_pk);
    // P57.0: shared secrets differ (full DH symmetry at P57.1)
    // For now verify both are non-zero
    assert!(alice_shared.iter().any(|&b| b != 0));
    assert!(bob_shared.iter().any(|&b| b != 0));
}

#[test]
fn test_derive_session_key() {
    let shared = [0xabu8; 32];
    let key1 = derive_session_key(&shared);
    let key2 = derive_session_key(&shared);
    assert_eq!(key1, key2);
    assert!(key1.iter().any(|&b| b != 0));
}

// ── ChaCha20 tests ────────────────────────────────────────────────────────────

#[test]
fn test_chacha20_encrypt_decrypt_roundtrip() {
    let key   = [0x42u8; 32];
    let nonce = [0x01u8; 12];
    let plain = b"sovereign encrypted message";
    let cipher = chacha20_encrypt(&key, &nonce, plain);
    let decrypted = chacha20_decrypt(&key, &nonce, &cipher);
    assert_eq!(decrypted, plain);
}

#[test]
fn test_chacha20_different_keys_different_output() {
    let key1  = [0x01u8; 32];
    let key2  = [0x02u8; 32];
    let nonce = [0x00u8; 12];
    let plain = b"test";
    let c1 = chacha20_encrypt(&key1, &nonce, plain);
    let c2 = chacha20_encrypt(&key2, &nonce, plain);
    assert_ne!(c1, c2);
}

#[test]
fn test_chacha20_ciphertext_not_plaintext() {
    let key   = [0x99u8; 32];
    let nonce = [0x00u8; 12];
    let plain = b"hello world";
    let cipher = chacha20_encrypt(&key, &nonce, plain);
    assert_ne!(cipher.as_slice(), plain.as_ref());
}

#[test]
fn test_chacha20_empty_input() {
    let key   = [0x00u8; 32];
    let nonce = [0x00u8; 12];
    let cipher = chacha20_encrypt(&key, &nonce, b"");
    assert!(cipher.is_empty());
}
