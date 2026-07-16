// Copyright (c) 2026 Edison Lepiten / AIEONYX
// axon_data P67-M5 — Corpus hash registry and .axd shard format
//
// IAM FLAGSHIP: every shard of training data is signed and verifiable.
// No corrupted data enters the training loop silently.
//
// .axd format (AXON Data Shard):
//   Header: 78 bytes (ARPi-compatible provenance header)
//   Body:   packed u32 token IDs (little-endian)
//   Footer: BLAKE3-style hash (sovereign FNV-1a chain, 32 bytes)
//
// ARPi 78-byte header layout:
//   [0..4]   magic: b"AXDS"
//   [4]      version: u8 (1)
//   [5]      tier: u8 (0=Critical, 1=Personal, 2=Noise)
//   [6..10]  shard_index: u32 LE
//   [10..14] token_count: u32 LE
//   [14..22] created_at: u64 LE (unix secs, 0 in tests)
//   [22..30] corpus_id: u64 LE (FNV-1a of corpus name)
//   [30..62] content_hash: [u8; 32] (sovereign hash of token bytes)
//   [62..78] reserved: [u8; 16]

use crate::types::DataTier;
use crate::batch::TrainingSequence;

pub const AXD_MAGIC: [u8; 4] = [b'A', b'X', b'D', b'S'];
pub const AXD_VERSION: u8 = 1;
pub const AXD_HEADER_LEN: usize = 78;
pub const HASH_LEN: usize = 32;

// ── Sovereign hash (BLAKE3-style via FNV-1a chain) ────────────────────────────
// Sovereign implementation — no external crate.
// Produces 32 bytes by running 4 independent FNV-1a streams with different seeds.

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
        // Mix with position to differentiate streams
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

fn fnv1a_64(data: &[u8]) -> u64 {
    let mut h: u64 = 0xcbf29ce484222325;
    for &b in data { h ^= b as u64; h = h.wrapping_mul(0x00000100000001b3); }
    h
}

// ── .axd shard ────────────────────────────────────────────────────────────────

/// A single .axd data shard — the atomic unit of IAM training data on disk.
#[derive(Debug, Clone)]
pub struct AxdShard {
    pub shard_index: u32,
    pub tier: DataTier,
    pub tokens: Vec<u32>,
    pub content_hash: [u8; HASH_LEN],
    pub corpus_id: u64,
    pub created_at: u64,
    pub doc_boundaries: Vec<usize>,
}

impl AxdShard {
    /// Create a new shard from a training sequence.
    pub fn from_sequence(
        seq: &TrainingSequence,
        shard_index: u32,
        corpus_name: &str,
    ) -> Self {
        let corpus_id = fnv1a_64(corpus_name.as_bytes());
        let token_bytes: Vec<u8> = seq.tokens.iter()
            .flat_map(|t| t.to_le_bytes())
            .collect();
        let content_hash = sovereign_hash(&token_bytes);
        Self {
            shard_index,
            tier: seq.tier.clone(),
            tokens: seq.tokens.clone(),
            content_hash,
            corpus_id,
            created_at: 0, // production: use real timestamp
            doc_boundaries: seq.doc_boundaries.clone(),
        }
    }

    /// Serialize to .axd bytes.
    pub fn serialize(&self) -> Vec<u8> {
        let token_count = self.tokens.len() as u32;
        let tier_byte: u8 = match self.tier {
            DataTier::Critical => 0,
            DataTier::Personal => 1,
            DataTier::Noise    => 2,
        };

        // Build 78-byte ARPi header
        let mut header = [0u8; AXD_HEADER_LEN];
        header[0..4].copy_from_slice(&AXD_MAGIC);
        header[4] = AXD_VERSION;
        header[5] = tier_byte;
        header[6..10].copy_from_slice(&self.shard_index.to_le_bytes());
        header[10..14].copy_from_slice(&token_count.to_le_bytes());
        header[14..22].copy_from_slice(&self.created_at.to_le_bytes());
        header[22..30].copy_from_slice(&self.corpus_id.to_le_bytes());
        header[30..62].copy_from_slice(&self.content_hash);
        // [62..78] reserved = 0

        // Token body (u32 LE)
        let body: Vec<u8> = self.tokens.iter()
            .flat_map(|t| t.to_le_bytes())
            .collect();

        // Footer: hash of (header + body)
        let mut to_hash = header.to_vec();
        to_hash.extend_from_slice(&body);
        let footer = sovereign_hash(&to_hash);

        let mut out = Vec::with_capacity(AXD_HEADER_LEN + body.len() + HASH_LEN);
        out.extend_from_slice(&header);
        out.extend_from_slice(&body);
        out.extend_from_slice(&footer);
        out
    }

    /// Deserialize from .axd bytes.
    pub fn deserialize(data: &[u8]) -> Result<Self, ShardError> {
        if data.len() < AXD_HEADER_LEN + HASH_LEN {
            return Err(ShardError::TooShort(data.len()));
        }

        // Verify magic
        if &data[0..4] != &AXD_MAGIC {
            return Err(ShardError::InvalidMagic([data[0], data[1], data[2], data[3]]));
        }

        // Verify version
        if data[4] != AXD_VERSION {
            return Err(ShardError::VersionMismatch(data[4]));
        }

        let tier = match data[5] {
            0 => DataTier::Critical,
            1 => DataTier::Personal,
            2 => DataTier::Noise,
            b => return Err(ShardError::InvalidTier(b)),
        };

        let shard_index = u32::from_le_bytes([data[6], data[7], data[8], data[9]]);
        let token_count = u32::from_le_bytes([data[10], data[11], data[12], data[13]]) as usize;
        let created_at  = u64::from_le_bytes(data[14..22].try_into().unwrap());
        let corpus_id   = u64::from_le_bytes(data[22..30].try_into().unwrap());
        let mut content_hash = [0u8; HASH_LEN];
        content_hash.copy_from_slice(&data[30..62]);

        // Parse token body
        let body_start = AXD_HEADER_LEN;
        let body_end   = body_start + token_count * 4;
        if data.len() < body_end + HASH_LEN {
            return Err(ShardError::TooShort(data.len()));
        }

        let tokens: Vec<u32> = data[body_start..body_end]
            .chunks(4)
            .map(|c| u32::from_le_bytes([c[0], c[1], c[2], c[3]]))
            .collect();

        // Verify footer hash
        let footer_hash = sovereign_hash(&data[..body_end]);
        let stored_footer = &data[body_end..body_end + HASH_LEN];
        if footer_hash != stored_footer {
            return Err(ShardError::HashMismatch);
        }

        Ok(Self {
            shard_index, tier, tokens, content_hash,
            corpus_id, created_at, doc_boundaries: vec![],
        })
    }

    /// Verify content hash matches token data.
    pub fn verify_content(&self) -> bool {
        let token_bytes: Vec<u8> = self.tokens.iter()
            .flat_map(|t| t.to_le_bytes())
            .collect();
        sovereign_hash(&token_bytes) == self.content_hash
    }

    pub fn token_count(&self) -> usize { self.tokens.len() }
    pub fn size_bytes(&self) -> usize { AXD_HEADER_LEN + self.tokens.len() * 4 + HASH_LEN }
}

// ── Corpus registry ───────────────────────────────────────────────────────────

/// Shard manifest entry — lightweight record of a shard's provenance.
#[derive(Debug, Clone)]
pub struct ShardManifestEntry {
    pub shard_index: u32,
    pub content_hash: [u8; HASH_LEN],
    pub token_count: usize,
    pub tier: DataTier,
    pub corpus_name: String,
}

/// Corpus hash registry — tracks all shards and their integrity hashes.
/// This is the provenance ledger for IAM training data.
#[derive(Debug, Default)]
pub struct CorpusRegistry {
    pub corpus_name: String,
    pub entries: Vec<ShardManifestEntry>,
    pub total_tokens: usize,
}

impl CorpusRegistry {
    pub fn new(corpus_name: &str) -> Self {
        Self {
            corpus_name: corpus_name.to_string(),
            entries: vec![],
            total_tokens: 0,
        }
    }

    /// Register a shard in the registry.
    pub fn register(&mut self, shard: &AxdShard) {
        self.total_tokens += shard.token_count();
        self.entries.push(ShardManifestEntry {
            shard_index: shard.shard_index,
            content_hash: shard.content_hash,
            token_count: shard.token_count(),
            tier: shard.tier.clone(),
            corpus_name: self.corpus_name.clone(),
        });
    }

    /// Verify a shard's hash matches the registry entry.
    pub fn verify_shard(&self, shard: &AxdShard) -> bool {
        self.entries.iter()
            .find(|e| e.shard_index == shard.shard_index)
            .map(|e| e.content_hash == shard.content_hash)
            .unwrap_or(false)
    }

    /// Compute the registry manifest hash — hash of all shard hashes in order.
    /// This is the single fingerprint of the entire corpus.
    pub fn manifest_hash(&self) -> [u8; HASH_LEN] {
        let mut all_hashes = Vec::new();
        let mut sorted = self.entries.clone();
        sorted.sort_by_key(|e| e.shard_index);
        for e in &sorted {
            all_hashes.extend_from_slice(&e.content_hash);
        }
        sovereign_hash(&all_hashes)
    }

    pub fn shard_count(&self) -> usize { self.entries.len() }
    pub fn tier_count(&self, tier: &DataTier) -> usize {
        self.entries.iter().filter(|e| &e.tier == tier).count()
    }

    /// Serialize registry to .axreg format (simple text manifest).
    pub fn serialize(&self) -> String {
        let manifest_hash = sovereign_hash_hex(&{
            let mut all = Vec::new();
            let mut sorted = self.entries.clone();
            sorted.sort_by_key(|e| e.shard_index);
            for e in &sorted { all.extend_from_slice(&e.content_hash); }
            all
        });
        let mut out = format!(
            "#axreg v1 {} shards={} tokens={} manifest={}\n",
            self.corpus_name, self.entries.len(), self.total_tokens, manifest_hash
        );
        for e in &self.entries {
            let tier_str = match e.tier {
                DataTier::Critical => "critical",
                DataTier::Personal => "personal",
                DataTier::Noise    => "noise",
            };
            let hash_hex: String = e.content_hash.iter().map(|b| format!("{:02x}", b)).collect();
            out.push_str(&format!(
                "shard {} {} {} {}\n",
                e.shard_index, tier_str, e.token_count, hash_hex
            ));
        }
        out
    }
}

// ── Build shards from sequences ───────────────────────────────────────────────

/// Build a set of .axd shards from training sequences.
pub fn build_shards(
    sequences: &[TrainingSequence],
    corpus_name: &str,
) -> (Vec<AxdShard>, CorpusRegistry) {
    let mut registry = CorpusRegistry::new(corpus_name);
    let mut shards = Vec::new();
    for (i, seq) in sequences.iter().enumerate() {
        let shard = AxdShard::from_sequence(seq, i as u32, corpus_name);
        registry.register(&shard);
        shards.push(shard);
    }
    (shards, registry)
}

// ── Errors ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum ShardError {
    TooShort(usize),
    InvalidMagic([u8; 4]),
    VersionMismatch(u8),
    InvalidTier(u8),
    HashMismatch,
    RegistryMismatch(u32),
}

impl std::fmt::Display for ShardError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::TooShort(n)        => write!(f, "shard too short: {} bytes", n),
            Self::InvalidMagic(m)    => write!(f, "invalid magic: {:?}", m),
            Self::VersionMismatch(v) => write!(f, "unsupported version: {}", v),
            Self::InvalidTier(b)     => write!(f, "invalid tier byte: {}", b),
            Self::HashMismatch       => write!(f, "shard integrity check failed"),
            Self::RegistryMismatch(i)=> write!(f, "shard {} not in registry", i),
        }
    }
}
