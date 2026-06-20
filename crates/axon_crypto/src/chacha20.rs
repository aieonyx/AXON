// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
// ChaCha20 stream cipher -- sovereign implementation.
// Clean-room: studied RFC 7539 specification only. No code copied.
pub struct ChaCha20 {
    state: [u32; 16],
}

impl ChaCha20 {
    const CONSTANTS: [u32; 4] = [0x61707865, 0x3320646e, 0x79622d32, 0x6b206574];

    pub fn new(key: &[u8; 32], nonce: &[u8; 12], counter: u32) -> Self {
        let mut state = [0u32; 16];
        state[0] = Self::CONSTANTS[0];
        state[1] = Self::CONSTANTS[1];
        state[2] = Self::CONSTANTS[2];
        state[3] = Self::CONSTANTS[3];
        for i in 0..8 {
            state[4+i] = u32::from_le_bytes(key[i*4..(i+1)*4].try_into().unwrap());
        }
        state[12] = counter;
        state[13] = u32::from_le_bytes(nonce[0..4].try_into().unwrap());
        state[14] = u32::from_le_bytes(nonce[4..8].try_into().unwrap());
        state[15] = u32::from_le_bytes(nonce[8..12].try_into().unwrap());
        ChaCha20 { state }
    }

    fn qr(s: &mut [u32; 16], a: usize, b: usize, c: usize, d: usize) {
        s[a] = s[a].wrapping_add(s[b]); s[d] ^= s[a]; s[d] = s[d].rotate_left(16);
        s[c] = s[c].wrapping_add(s[d]); s[b] ^= s[c]; s[b] = s[b].rotate_left(12);
        s[a] = s[a].wrapping_add(s[b]); s[d] ^= s[a]; s[d] = s[d].rotate_left(8);
        s[c] = s[c].wrapping_add(s[d]); s[b] ^= s[c]; s[b] = s[b].rotate_left(7);
    }

    fn block(&self) -> [u8; 64] {
        let mut s = self.state;
        for _ in 0..10 {
            Self::qr(&mut s, 0, 4, 8,  12);
            Self::qr(&mut s, 1, 5, 9,  13);
            Self::qr(&mut s, 2, 6, 10, 14);
            Self::qr(&mut s, 3, 7, 11, 15);
            Self::qr(&mut s, 0, 5, 10, 15);
            Self::qr(&mut s, 1, 6, 11, 12);
            Self::qr(&mut s, 2, 7, 8,  13);
            Self::qr(&mut s, 3, 4, 9,  14);
        }
        for i in 0..16 { s[i] = s[i].wrapping_add(self.state[i]); }
        let mut out = [0u8; 64];
        for i in 0..16 {
            out[i*4..(i+1)*4].copy_from_slice(&s[i].to_le_bytes());
        }
        out
    }

    pub fn encrypt(&mut self, data: &[u8]) -> Vec<u8> {
        let mut out = Vec::with_capacity(data.len());
        let mut pos = 0;
        while pos < data.len() {
            let block = self.block();
            self.state[12] = self.state[12].wrapping_add(1);
            let end = (pos + 64).min(data.len());
            for (i, &byte) in data[pos..end].iter().enumerate() {
                out.push(byte ^ block[i]);
            }
            pos += 64;
        }
        out
    }

    pub fn decrypt(&mut self, data: &[u8]) -> Vec<u8> { self.encrypt(data) }
}

pub fn chacha20_encrypt(key: &[u8; 32], nonce: &[u8; 12], data: &[u8]) -> Vec<u8> {
    ChaCha20::new(key, nonce, 1).encrypt(data)
}

pub fn chacha20_decrypt(key: &[u8; 32], nonce: &[u8; 12], data: &[u8]) -> Vec<u8> {
    ChaCha20::new(key, nonce, 1).decrypt(data)
}
