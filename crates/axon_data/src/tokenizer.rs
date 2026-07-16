// Copyright (c) 2026 Edison Lepiten / AIEONYX
// axon_data P67-M3 — Sovereign BPE Tokenizer
//
// IAM FLAGSHIP: this is the vocabulary IAM thinks in.
// Every word, every concept, every wisdom principle IAM will ever reason about
// passes through this tokenizer first.
//
// Architecture: byte-level BPE (Byte Pair Encoding)
// - Starts from raw UTF-8 bytes (256 base tokens)
// - Iteratively merges the most frequent adjacent pair
// - Terminates at target vocabulary size
// - Special tokens reserved BEFORE training (IDs 0-31)
// - Deterministic: same corpus + seed → identical tokenizer
// - Output format: .axvocab (sovereign vocabulary file)
//
// Special token registry (IDs 0-15 reserved for IAM):
//   0  <pad>      padding
//   1  <bos>      beginning of sequence
//   2  <eos>      end of sequence
//   3  <unk>      unknown token
//   4  <iam>      IAM brain identity marker
//   5  <intent>   @intent classification boundary (Article 0)
//   6  </intent>  @intent boundary close
//   7  <conf_g>   green confidence (Article 2 — verified)
//   8  <conf_y>   yellow confidence (Article 2 — reasoned)
//   9  <conf_r>   red confidence (Article 2 — unknown, search offered)
//  10  <unknown>  honest unknown marker (Article 2)
//  11  <layer>    knowledge layer boundary
//  12  </layer>   knowledge layer boundary close
//  13  <persona>  personality overlay marker
//  14  </persona> personality overlay close
//  15  <tool>     plugin call marker
//  16-31          future-reserved (IAM expansion)

use std::collections::HashMap;

// ── Special token registry ────────────────────────────────────────────────────

pub const PAD_ID:      u32 = 0;
pub const BOS_ID:      u32 = 1;
pub const EOS_ID:      u32 = 2;
pub const UNK_ID:      u32 = 3;
pub const IAM_ID:      u32 = 4;
pub const INTENT_ID:   u32 = 5;
pub const INTENT_E_ID: u32 = 6;
pub const CONF_G_ID:   u32 = 7;
pub const CONF_Y_ID:   u32 = 8;
pub const CONF_R_ID:   u32 = 9;
pub const UNKNOWN_ID:  u32 = 10;
pub const LAYER_ID:    u32 = 11;
pub const LAYER_E_ID:  u32 = 12;
pub const PERSONA_ID:  u32 = 13;
pub const PERSONA_E_ID:u32 = 14;
pub const TOOL_ID:     u32 = 15;

pub const SPECIAL_TOKEN_COUNT: usize = 32; // 0-31 reserved
pub const BASE_VOCAB_SIZE: usize = 256;    // raw bytes
pub const DEFAULT_VOCAB_SIZE: usize = 8000; // IAM seed target (spec: 8k-16k)

/// IAM special tokens — the constitutional vocabulary.
pub const SPECIAL_TOKENS: &[(&str, u32)] = &[
    ("<pad>",      PAD_ID),
    ("<bos>",      BOS_ID),
    ("<eos>",      EOS_ID),
    ("<unk>",      UNK_ID),
    ("<iam>",      IAM_ID),
    ("<intent>",   INTENT_ID),
    ("</intent>",  INTENT_E_ID),
    ("<conf_g>",   CONF_G_ID),
    ("<conf_y>",   CONF_Y_ID),
    ("<conf_r>",   CONF_R_ID),
    ("<unknown>",  UNKNOWN_ID),
    ("<layer>",    LAYER_ID),
    ("</layer>",   LAYER_E_ID),
    ("<persona>",  PERSONA_ID),
    ("</persona>", PERSONA_E_ID),
    ("<tool>",     TOOL_ID),
];

// ── BPE vocabulary ────────────────────────────────────────────────────────────

/// A BPE merge rule: (left_token, right_token) → merged_token
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Merge {
    pub left: u32,
    pub right: u32,
    pub result: u32,
}

/// The complete sovereign BPE vocabulary.
#[derive(Debug, Clone)]
pub struct BpeVocab {
    /// Token ID → byte sequence
    pub id_to_token: Vec<Vec<u8>>,
    /// Byte sequence → token ID (for fast lookup)
    pub token_to_id: HashMap<Vec<u8>, u32>,
    /// Merge rules in order of application
    pub merges: Vec<Merge>,
    /// Special token map: string → ID
    pub special_tokens: HashMap<String, u32>,
    /// Total vocabulary size
    pub vocab_size: u32,
}

impl BpeVocab {
    /// Build a new vocabulary with base bytes + special tokens reserved.
    pub fn new() -> Self {
        let total = SPECIAL_TOKEN_COUNT + BASE_VOCAB_SIZE;
        // Pre-allocate with empty vecs for ALL slots (special + base bytes)
        let mut id_to_token = vec![vec![]; total];
        let mut token_to_id = HashMap::new();
        let mut special_tokens = HashMap::new();

        // Register special tokens (IDs 0-15)
        for &(name, id) in SPECIAL_TOKENS {
            id_to_token[id as usize] = name.as_bytes().to_vec();
            token_to_id.insert(name.as_bytes().to_vec(), id);
            special_tokens.insert(name.to_string(), id);
        }
        // Fill reserved slots 16-31 with placeholder
        for id in 16..SPECIAL_TOKEN_COUNT as u32 {
            let name = format!("<reserved_{}>", id);
            id_to_token[id as usize] = name.as_bytes().to_vec();
        }

        // Register base byte tokens at IDs 32-287 (in-place, not push)
        for byte in 0u8..=255 {
            let id = (SPECIAL_TOKEN_COUNT as u32 + byte as u32) as usize;
            id_to_token[id] = vec![byte];
            token_to_id.insert(vec![byte], id as u32);
        }

        Self {
            vocab_size: total as u32,
            id_to_token,
            token_to_id,
            merges: vec![],
            special_tokens,
        }
    }

    /// Add a new merge token to the vocabulary.
    pub fn add_merge(&mut self, left: u32, right: u32) -> u32 {
        let new_id = self.vocab_size;
        let mut merged = self.id_to_token[left as usize].clone();
        merged.extend_from_slice(&self.id_to_token[right as usize]);
        self.id_to_token.push(merged.clone());
        self.token_to_id.insert(merged, new_id);
        self.merges.push(Merge { left, right, result: new_id });
        self.vocab_size += 1;
        new_id
    }

    /// Look up a token string → ID.
    pub fn token_id(&self, token: &[u8]) -> Option<u32> {
        self.token_to_id.get(token).copied()
    }

    /// Get token bytes for an ID.
    pub fn token_bytes(&self, id: u32) -> Option<&[u8]> {
        self.id_to_token.get(id as usize).map(|v| v.as_slice())
    }

    /// Check if an ID is a special token.
    pub fn is_special(&self, id: u32) -> bool {
        id < SPECIAL_TOKEN_COUNT as u32
    }

    /// Get special token ID by name.
    pub fn special_id(&self, name: &str) -> Option<u32> {
        self.special_tokens.get(name).copied()
    }

    pub fn size(&self) -> u32 { self.vocab_size }
    pub fn merge_count(&self) -> usize { self.merges.len() }
}

impl Default for BpeVocab { fn default() -> Self { Self::new() } }

// ── BPE trainer ───────────────────────────────────────────────────────────────

/// Sovereign BPE trainer. Processes a corpus and learns merge rules.
/// Same corpus + same target_size → identical vocabulary (deterministic).
pub struct BpeTrainer {
    pub target_size: usize,
    pub min_frequency: usize,
}

impl BpeTrainer {
    pub fn new(target_size: usize) -> Self {
        Self { target_size, min_frequency: 2 }
    }

    pub fn with_min_frequency(mut self, freq: usize) -> Self {
        self.min_frequency = freq;
        self
    }

    /// Train a BPE vocabulary on a corpus string.
    /// Returns a trained BpeVocab with merges learned from the corpus.
    pub fn train(&self, corpus: &str) -> BpeVocab {
        let mut vocab = BpeVocab::new();
        if corpus.is_empty() { return vocab; }

        // Tokenize corpus into byte sequences (words split on whitespace)
        // Each word is represented as a sequence of byte token IDs
        let words = tokenize_to_words(corpus);

        // Build initial word sequences using base byte tokens
        let mut word_sequences: Vec<(Vec<u32>, usize)> = words.into_iter()
            .map(|(word, freq)| {
                let seq: Vec<u32> = word.bytes()
                    .map(|b| SPECIAL_TOKEN_COUNT as u32 + b as u32)
                    .collect();
                (seq, freq)
            })
            .collect();

        // Iteratively merge most frequent pairs
        while vocab.vocab_size < self.target_size as u32 {
            // Count pair frequencies
            let pair_freqs = count_pairs(&word_sequences);
            if pair_freqs.is_empty() { break; }

            // Find most frequent pair (deterministic: break ties by pair value)
            let best = pair_freqs.iter()
                .filter(|(_, &f)| f >= self.min_frequency)
                .max_by_key(|(&(l, r), &f)| (f, u64::MAX - l as u64, u64::MAX - r as u64));

            let &(left, right) = match best {
                Some((pair, _)) => pair,
                None => break,
            };

            // Add merge to vocabulary
            let new_id = vocab.add_merge(left, right);

            // Apply merge to all word sequences
            for (seq, _) in word_sequences.iter_mut() {
                apply_merge(seq, left, right, new_id);
            }
        }

        vocab
    }
}

// ── BPE encoder ───────────────────────────────────────────────────────────────

/// Encode text to token IDs using a trained vocabulary.
pub struct BpeEncoder<'a> {
    pub vocab: &'a BpeVocab,
}

impl<'a> BpeEncoder<'a> {
    pub fn new(vocab: &'a BpeVocab) -> Self { Self { vocab } }

    /// Encode text → token IDs.
    /// Special tokens in the text are matched first before BPE encoding.
    pub fn encode(&self, text: &str) -> Vec<u32> {
        let mut ids = Vec::new();

        // Split on special tokens first
        let segments = split_on_special_tokens(text, &self.vocab.special_tokens);

        for seg in segments {
            match seg {
                Segment::Special(id) => ids.push(id),
                Segment::Text(t) => {
                    // BPE encode each word
                    for word in t.split_whitespace() {
                        let word_ids = self.encode_word(word.as_bytes());
                        if !ids.is_empty() { ids.push(SPECIAL_TOKEN_COUNT as u32 + b' ' as u32); }
                        ids.extend(word_ids);
                    }
                }
            }
        }
        ids
    }

    /// Encode a single word (byte sequence) by applying learned merges.
    fn encode_word(&self, bytes: &[u8]) -> Vec<u32> {
        let mut seq: Vec<u32> = bytes.iter()
            .map(|&b| SPECIAL_TOKEN_COUNT as u32 + b as u32)
            .collect();

        // Apply merges in order
        for merge in &self.vocab.merges {
            apply_merge(&mut seq, merge.left, merge.right, merge.result);
        }
        seq
    }

    /// Decode token IDs → UTF-8 string (best effort).
    pub fn decode(&self, ids: &[u32]) -> String {
        let mut bytes = Vec::new();
        for &id in ids {
            if id < SPECIAL_TOKEN_COUNT as u32 {
                // Special token: emit its name
                if let Some(tok) = self.vocab.id_to_token.get(id as usize) {
                    bytes.extend_from_slice(tok);
                }
            } else if let Some(tok) = self.vocab.token_bytes(id) {
                bytes.extend_from_slice(tok);
            }
        }
        String::from_utf8_lossy(&bytes).to_string()
    }
}

// ── .axvocab format ───────────────────────────────────────────────────────────
// Sovereign vocabulary serialization format.
// Line 0: "#axvocab v1 <vocab_size> <merge_count>"
// Lines 1-N: merge rules "left_id right_id result_id"
// Lines N+1: special token declarations "special <name> <id>"

pub fn serialize_vocab(vocab: &BpeVocab) -> String {
    let mut out = String::new();
    out.push_str(&format!("#axvocab v1 {} {}\n", vocab.vocab_size, vocab.merge_count()));

    // Special tokens
    for (name, &id) in &vocab.special_tokens {
        out.push_str(&format!("special {} {}\n", name, id));
    }

    // Merge rules
    for merge in &vocab.merges {
        out.push_str(&format!("merge {} {} {}\n", merge.left, merge.right, merge.result));
    }
    out
}

pub fn deserialize_vocab(data: &str) -> Result<BpeVocab, String> {
    let mut lines = data.lines();

    // Header
    let header = lines.next().ok_or("empty .axvocab")?;
    if !header.starts_with("#axvocab v1") {
        return Err(format!("invalid .axvocab header: {}", header));
    }

    let mut vocab = BpeVocab::new();

    for line in lines {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') { continue; }

        let parts: Vec<&str> = line.splitn(4, ' ').collect();
        match parts[0] {
            "special" => {
                // Already registered in new() — skip
            }
            "merge" if parts.len() == 4 => {
                let l: u32 = parts[1].parse().map_err(|e| format!("bad merge left: {}", e))?;
                let r: u32 = parts[2].parse().map_err(|e| format!("bad merge right: {}", e))?;
                let _res: u32 = parts[3].parse().map_err(|e| format!("bad merge result: {}", e))?;
                vocab.add_merge(l, r);
            }
            _ => {}
        }
    }
    Ok(vocab)
}

// ── Internal helpers ──────────────────────────────────────────────────────────

/// Tokenize corpus into (word, frequency) pairs.
fn tokenize_to_words(corpus: &str) -> HashMap<String, usize> {
    let mut freqs = HashMap::new();
    for word in corpus.split_whitespace() {
        // Normalize: lowercase for frequency counting
        let w = word.to_lowercase();
        let w = w.trim_matches(|c: char| !c.is_alphanumeric());
        if !w.is_empty() {
            *freqs.entry(w.to_string()).or_insert(0) += 1;
        }
    }
    freqs
}

/// Count adjacent pair frequencies across all word sequences.
fn count_pairs(sequences: &[(Vec<u32>, usize)]) -> HashMap<(u32, u32), usize> {
    let mut freqs = HashMap::new();
    for (seq, count) in sequences {
        for window in seq.windows(2) {
            *freqs.entry((window[0], window[1])).or_insert(0) += count;
        }
    }
    freqs
}

/// Apply a merge rule to a token sequence in place.
fn apply_merge(seq: &mut Vec<u32>, left: u32, right: u32, result: u32) {
    let mut i = 0;
    let mut new_seq = Vec::with_capacity(seq.len());
    while i < seq.len() {
        if i + 1 < seq.len() && seq[i] == left && seq[i+1] == right {
            new_seq.push(result);
            i += 2;
        } else {
            new_seq.push(seq[i]);
            i += 1;
        }
    }
    *seq = new_seq;
}

/// Segment type for special token splitting.
enum Segment {
    Special(u32),
    Text(String),
}

/// Split text on special token boundaries.
fn split_on_special_tokens(text: &str, specials: &HashMap<String, u32>) -> Vec<Segment> {
    let mut segments = Vec::new();
    let mut remaining = text;

    while !remaining.is_empty() {
        // Find earliest special token match
        let mut earliest: Option<(usize, usize, u32)> = None; // (start, end, id)
        for (name, &id) in specials {
            if let Some(pos) = remaining.find(name.as_str()) {
                let end = pos + name.len();
                if earliest.is_none() || pos < earliest.unwrap().0 {
                    earliest = Some((pos, end, id));
                }
            }
        }
        match earliest {
            Some((start, end, id)) => {
                if start > 0 {
                    segments.push(Segment::Text(remaining[..start].to_string()));
                }
                segments.push(Segment::Special(id));
                remaining = &remaining[end..];
            }
            None => {
                segments.push(Segment::Text(remaining.to_string()));
                break;
            }
        }
    }
    segments
}
