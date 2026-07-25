// Copyright (c) 2026 Edison Lepiten / AIEONYX
// axon_pkg P72 — Sovereign hash (reuses axon_data discipline)

pub const HASH_LEN: usize = 32;

/// Sovereign hash: 4-stream FNV-1a, 32 bytes. Same as axon_data::shard.
pub fn sovereign_hash(data: &[u8]) -> [u8; HASH_LEN] {
    const SEEDS: [u64; 4] = [
        0xcbf29ce484222325,
        0x9e3779b97f4a7c15,
        0x6c62272e07bb0142,
        0x517cc1b727220a95,
    ];
    let mut out = [0u8; HASH_LEN];
    for (i, &seed) in SEEDS.iter().enumerate() {
        let mut h = seed;
        for &b in data {
            h ^= b as u64;
            h = h.wrapping_mul(0x00000100000001b3);
        }
        h ^= (i as u64).wrapping_mul(0xbf58476d1ce4e5b9);
        h = h.wrapping_mul(0x94d049bb133111eb);
        h ^= h >> 31;
        out[i*8..(i+1)*8].copy_from_slice(&h.to_le_bytes());
    }
    out
}

pub fn sovereign_hash_hex(data: &[u8]) -> String {
    sovereign_hash(data).iter().map(|b| format!("{:02x}", b)).collect()
}
