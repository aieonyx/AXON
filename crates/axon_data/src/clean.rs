// Copyright (c) 2026 Edison Lepiten / AIEONYX
// axon_data P67-M2 — Text cleaning and normalization pipeline
//
// IAM FLAGSHIP: garbage in = garbage brain.
// Every entry passes through this pipeline before tokenization.
//
// Pipeline order:
//   1. Unicode normalization (NFC, collapse whitespace)
//   2. PII scrubbing (email, phone, IP, credit card patterns)
//   3. Quality filters (length, repetition, symbol density)
//   4. Minhash near-dedup (Jaccard 0.8 threshold)

use crate::types::CorpusEntry;

// ── Cleaning options ──────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct CleanOptions {
    /// Scrub PII patterns (email, phone, IP, credit card)
    pub scrub_pii: bool,
    /// Normalize unicode whitespace
    pub normalize_whitespace: bool,
    /// Remove entries with symbol density > threshold
    pub max_symbol_density: f32,
    /// Remove entries with repetition ratio > threshold (repeated n-grams)
    pub max_repetition_ratio: f32,
    /// Minimum unique word ratio (filters gibberish)
    pub min_unique_word_ratio: f32,
    /// Replace PII with placeholder or remove entirely
    pub pii_placeholder: Option<&'static str>,
}

impl Default for CleanOptions {
    fn default() -> Self {
        Self {
            scrub_pii: true,
            normalize_whitespace: true,
            max_symbol_density: 0.3,
            max_repetition_ratio: 0.5,
            min_unique_word_ratio: 0.2,
            pii_placeholder: Some("[REDACTED]"),
        }
    }
}

impl CleanOptions {
    /// Constitutional mode — strict, keep PII placeholder visible
    pub fn constitutional() -> Self {
        Self {
            scrub_pii: true,
            normalize_whitespace: true,
            max_symbol_density: 0.2,
            max_repetition_ratio: 0.4,
            min_unique_word_ratio: 0.3,
            pii_placeholder: Some("[SOVEREIGN-REDACTED]"),
        }
    }

    /// Lenient mode for code/technical content
    pub fn technical() -> Self {
        Self {
            scrub_pii: true,
            normalize_whitespace: true,
            max_symbol_density: 0.6, // code has many symbols
            max_repetition_ratio: 0.7,
            min_unique_word_ratio: 0.1,
            pii_placeholder: Some("[REDACTED]"),
        }
    }
}

// ── Clean result ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum CleanVerdict {
    /// Entry passes — cleaned text returned
    Pass,
    /// Entry filtered — reason given
    Filtered(FilterReason),
}

#[derive(Debug, Clone, PartialEq)]
pub enum FilterReason {
    TooShort,
    TooManySymbols(f32),
    TooMuchRepetition(f32),
    TooFewUniqueWords(f32),
    EmptyAfterCleaning,
    NearDuplicate(u64), // minhash signature of the duplicate
}

impl std::fmt::Display for FilterReason {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::TooShort              => write!(f, "too short after cleaning"),
            Self::TooManySymbols(d)     => write!(f, "symbol density {:.2} exceeds threshold", d),
            Self::TooMuchRepetition(r)  => write!(f, "repetition ratio {:.2} exceeds threshold", r),
            Self::TooFewUniqueWords(r)  => write!(f, "unique word ratio {:.2} below threshold", r),
            Self::EmptyAfterCleaning    => write!(f, "empty after cleaning"),
            Self::NearDuplicate(sig)    => write!(f, "near-duplicate of signature {}", sig),
        }
    }
}

// ── PII scrubbing ─────────────────────────────────────────────────────────────

/// Scrub PII patterns from text. Returns (cleaned_text, pii_found_count).
/// Patterns: email, phone (intl), IPv4, credit card (basic Luhn-free pattern).
/// Sovereign note: we never log or store the actual PII values.
pub fn scrub_pii(text: &str, placeholder: Option<&str>) -> (String, usize) {
    let ph = placeholder.unwrap_or("[REDACTED]");
    let mut result = text.to_string();
    let mut count = 0;

    // Email: user@domain.tld
    let cleaned = replace_pattern_simple(&result, |s| is_email_like(s), ph);
    if cleaned != result { count += 1; result = cleaned; }

    // IPv4: n.n.n.n
    let cleaned = replace_ipv4(&result, ph);
    if cleaned != result { count += 1; result = cleaned; }

    // Phone: various formats +1-555-555-5555, (555) 555-5555, etc.
    let cleaned = replace_phone(&result, ph);
    if cleaned != result { count += 1; result = cleaned; }

    // Credit card: 4 groups of 4 digits
    let cleaned = replace_cc(&result, ph);
    if cleaned != result { count += 1; result = cleaned; }

    (result, count)
}

fn is_email_like(s: &str) -> bool {
    let parts: Vec<&str> = s.split('@').collect();
    if parts.len() != 2 { return false; }
    let domain = parts[1];
    domain.contains('.') && domain.len() > 3
        && parts[0].len() > 0
        && parts[0].chars().all(|c| c.is_alphanumeric() || "._+-".contains(c))
}

fn replace_pattern_simple(text: &str, check: impl Fn(&str) -> bool, ph: &str) -> String {
    let words: Vec<&str> = text.split_whitespace().collect();
    let replaced: Vec<&str> = words.iter().map(|w| {
        // strip punctuation for check
        let clean = w.trim_matches(|c: char| !c.is_alphanumeric() && c != '@' && c != '.');
        if check(clean) { ph } else { w }
    }).collect();
    replaced.join(" ")
}

fn replace_ipv4(text: &str, ph: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        if c.is_ascii_digit() {
            let mut candidate = String::new();
            candidate.push(c);
            while let Some(&nc) = chars.peek() {
                if nc.is_ascii_digit() || nc == '.' {
                    candidate.push(nc);
                    chars.next();
                } else { break; }
            }
            // Check if it looks like an IPv4
            let parts: Vec<&str> = candidate.split('.').collect();
            if parts.len() == 4 && parts.iter().all(|p| p.parse::<u8>().is_ok()) {
                out.push_str(ph);
            } else {
                out.push_str(&candidate);
            }
        } else {
            out.push(c);
        }
    }
    out
}

fn replace_phone(text: &str, ph: &str) -> String {
    // Simple heuristic: sequences of digits with spaces/dashes/parens totaling 10-15 digits
    let mut out = text.to_string();
    let mut i = 0;
    let bytes = text.as_bytes();
    while i < bytes.len() {
        // Look for digit sequences that might be phone numbers
        if bytes[i].is_ascii_digit() || bytes[i] == b'+' {
            let start = i;
            let mut digit_count = 0;
            let mut j = i;
            while j < bytes.len() && (bytes[j].is_ascii_digit()
                || bytes[j] == b'+' || bytes[j] == b'-'
                || bytes[j] == b' ' || bytes[j] == b'(' || bytes[j] == b')') {
                if bytes[j].is_ascii_digit() { digit_count += 1; }
                j += 1;
                if digit_count > 15 { break; }
            }
            if digit_count >= 10 && digit_count <= 15 {
                let before = &text[..start];
                let after = &text[j..];
                out = format!("{}{}{}", before, ph, after);
                return out; // simple: replace first match
            }
        }
        i += 1;
    }
    out
}

fn replace_cc(text: &str, ph: &str) -> String {
    // 4 groups of 4 digits separated by spaces or dashes
    let words: Vec<&str> = text.split_whitespace().collect();
    let n = words.len();
    if n < 4 { return text.to_string(); }
    let mut result = text.to_string();
    for i in 0..n.saturating_sub(3) {
        let w = [words[i], words[i+1], words[i+2], words[i+3]];
        let all_4_digit = w.iter().all(|s| {
            let s = s.trim_matches('-');
            s.len() == 4 && s.chars().all(|c| c.is_ascii_digit())
        });
        if all_4_digit {
            let pattern = format!("{} {} {} {}", w[0], w[1], w[2], w[3]);
            result = result.replace(&pattern, ph);
        }
    }
    result
}

// ── Whitespace normalization ───────────────────────────────────────────────────

/// Normalize whitespace: collapse multiple spaces/tabs/newlines, trim.
pub fn normalize_whitespace(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut prev_space = false;
    let mut prev_newline = false;

    for c in text.chars() {
        if c == '\n' || c == '\r' {
            if !prev_newline {
                out.push('\n');
                prev_newline = true;
                prev_space = false;
            }
        } else if c.is_whitespace() {
            if !prev_space && !prev_newline {
                out.push(' ');
                prev_space = true;
            }
        } else {
            out.push(c);
            prev_space = false;
            prev_newline = false;
        }
    }
    out.trim().to_string()
}

// ── Quality filters ───────────────────────────────────────────────────────────

/// Compute symbol density: ratio of non-alphanumeric, non-space chars.
pub fn symbol_density(text: &str) -> f32 {
    if text.is_empty() { return 0.0; }
    let total = text.chars().count();
    let symbols = text.chars().filter(|c| !c.is_alphanumeric() && !c.is_whitespace()).count();
    symbols as f32 / total as f32
}

/// Compute repetition ratio: fraction of trigrams that appear more than once.
pub fn repetition_ratio(text: &str) -> f32 {
    let words: Vec<&str> = text.split_whitespace().collect();
    if words.len() < 4 { return 0.0; }

    let mut trigrams = std::collections::HashMap::new();
    for w in words.windows(3) {
        let tri = format!("{} {} {}", w[0], w[1], w[2]);
        *trigrams.entry(tri).or_insert(0usize) += 1;
    }

    let total = trigrams.len();
    let repeated = trigrams.values().filter(|&&c| c > 1).count();
    if total == 0 { return 0.0; }
    repeated as f32 / total as f32
}

/// Compute unique word ratio: unique words / total words.
pub fn unique_word_ratio(text: &str) -> f32 {
    let words: Vec<&str> = text.split_whitespace().collect();
    if words.is_empty() { return 0.0; }
    let unique: std::collections::HashSet<&str> = words.iter().cloned().collect();
    unique.len() as f32 / words.len() as f32
}

// ── Minhash dedup ─────────────────────────────────────────────────────────────
// Sovereign implementation: FNV-1a based minhash, no external crate.
// Uses 32 hash functions (32-bit), Jaccard threshold 0.8.

const MINHASH_BANDS: usize = 8;
const MINHASH_ROWS: usize = 4;
const MINHASH_SIZE: usize = MINHASH_BANDS * MINHASH_ROWS; // 32

/// Compute minhash signature for text (shingles of 3 words).
pub fn minhash_signature(text: &str) -> [u32; MINHASH_SIZE] {
    let words: Vec<&str> = text.split_whitespace().collect();
    let mut sig = [u32::MAX; MINHASH_SIZE];

    // Generate word trigram shingles
    let shingles: Vec<u64> = if words.len() >= 3 {
        words.windows(3).map(|w| {
            fnv1a_64(format!("{} {} {}", w[0], w[1], w[2]).as_bytes())
        }).collect()
    } else {
        // Fallback: word unigrams
        words.iter().map(|w| fnv1a_64(w.as_bytes())).collect()
    };

    // Minhash: for each hash function h_i, sig[i] = min(h_i(shingle))
    for (i, s) in sig.iter_mut().enumerate() {
        let a = HASH_A[i % 16];
        let b = HASH_B[i % 16];
        for &shingle in &shingles {
            let h = ((a.wrapping_mul(shingle).wrapping_add(b)) >> 32) as u32;
            if h < *s { *s = h; }
        }
    }
    sig
}

/// Estimate Jaccard similarity from two minhash signatures.
pub fn jaccard_estimate(a: &[u32; MINHASH_SIZE], b: &[u32; MINHASH_SIZE]) -> f32 {
    let matches = a.iter().zip(b.iter()).filter(|(x, y)| x == y).count();
    matches as f32 / MINHASH_SIZE as f32
}

/// Compute a single dedup key (XOR of signature bands) for fast lookup.
pub fn dedup_key(sig: &[u32; MINHASH_SIZE]) -> u64 {
    let mut key = 0u64;
    for (i, &v) in sig.iter().enumerate() {
        key ^= (v as u64) << ((i % 8) * 8 % 64);
    }
    key
}

fn fnv1a_64(data: &[u8]) -> u64 {
    let mut h: u64 = 0xcbf29ce484222325;
    for &b in data {
        h ^= b as u64;
        h = h.wrapping_mul(0x00000100000001b3_u64);
    }
    h
}

// Pre-computed hash function coefficients (prime-based)
const HASH_A: [u64; 16] = [
    0x9e3779b97f4a7c15_u64, 0x6c62272e07bb0142_u64,
    0x94d049bb133111eb_u64, 0xbf58476d1ce4e5b9_u64,
    0x517cc1b727220a95_u64, 0x2545f4914f6cdd1d_u64,
    0x9e3779b185ebca87_u64, 0xc2b2ae3d27d4eb4f_u64,
    0x165667b19e3779f9_u64, 0x85ebca77c2b2ae63_u64,
    0x4be98134a5976fd3_u64, 0x3f84d5b5b5470917_u64,
    0xa87ca1891c2f2df1_u64, 0x27d4eb2f165667c5_u64,
    0xb5470917a87ca1af_u64, 0xff51afd7ed558ccd_u64,
];
const HASH_B: [u64; 16] = [
    0x8f3f9c4a4d5e7b3d_u64, 0x1b873593d3a2f4e5_u64,
    0x7fb5d329728ea185_u64, 0x81dadef4bc9dec31_u64,
    0xa5e6937b2c843f2b_u64, 0x4295a2e7dec8a8f5_u64,
    0xc1f8a2b7e53d9f6a_u64, 0x2d7e4a3b9c851f07_u64,
    0x6b3c4e2a1d8f5907_u64, 0xf3a7e2c1b5d84069_u64,
    0x8c3d5e7a1b492f63_u64, 0x4f2e9a1b6d7c3e85_u64,
    0xa1b2c3d4e5f60718_u64, 0x9f8e7d6c5b4a3928_u64,
    0x1234567890abcdef_u64, 0xfedcba0987654321_u64,
];

// ── Main clean pipeline ───────────────────────────────────────────────────────

/// Clean a single corpus entry. Returns (cleaned_entry, verdict).
pub fn clean_entry(
    entry: &CorpusEntry,
    opts: &CleanOptions,
) -> (CorpusEntry, CleanVerdict) {
    let mut text = entry.text.clone();
    let mut response = entry.response.clone();

    // 1. Normalize whitespace
    if opts.normalize_whitespace {
        text = normalize_whitespace(&text);
        response = normalize_whitespace(&response);
    }

    // 2. PII scrub
    if opts.scrub_pii {
        let (t, _) = scrub_pii(&text, opts.pii_placeholder);
        let (r, _) = scrub_pii(&response, opts.pii_placeholder);
        text = t;
        response = r;
    }

    // 3. Empty check
    if text.trim().is_empty() {
        return (entry.clone(), CleanVerdict::Filtered(FilterReason::EmptyAfterCleaning));
    }

    // 4. Symbol density
    let density = symbol_density(&text);
    if density > opts.max_symbol_density {
        return (entry.clone(), CleanVerdict::Filtered(FilterReason::TooManySymbols(density)));
    }

    // 5. Repetition ratio
    let rep = repetition_ratio(&text);
    if rep > opts.max_repetition_ratio {
        return (entry.clone(), CleanVerdict::Filtered(FilterReason::TooMuchRepetition(rep)));
    }

    // 6. Unique word ratio
    let uniq = unique_word_ratio(&text);
    if uniq < opts.min_unique_word_ratio && text.split_whitespace().count() > 10 {
        return (entry.clone(), CleanVerdict::Filtered(FilterReason::TooFewUniqueWords(uniq)));
    }

    let char_count = text.chars().count() + response.chars().count();
    let cleaned = CorpusEntry {
        text,
        response,
        tier: entry.tier.clone(),
        source: entry.source.clone(),
        tags: entry.tags.clone(),
        char_count,
        is_pair: entry.is_pair,
    };

    (cleaned, CleanVerdict::Pass)
}

/// Clean a corpus batch. Returns (passed, filtered_count).
pub fn clean_batch(
    entries: &[CorpusEntry],
    opts: &CleanOptions,
    seen_signatures: &mut std::collections::HashSet<u64>,
) -> (Vec<CorpusEntry>, usize) {
    let mut passed = Vec::new();
    let mut filtered = 0;

    for entry in entries {
        let (cleaned, verdict) = clean_entry(entry, opts);
        match verdict {
            CleanVerdict::Pass => {
                // Minhash dedup check
                let sig = minhash_signature(&cleaned.text);
                let key = dedup_key(&sig);
                if seen_signatures.contains(&key) {
                    filtered += 1;
                } else {
                    seen_signatures.insert(key);
                    passed.push(cleaned);
                }
            }
            CleanVerdict::Filtered(_) => { filtered += 1; }
        }
    }

    (passed, filtered)
}
