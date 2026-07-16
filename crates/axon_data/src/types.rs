// Copyright (c) 2026 Edison Lepiten / AIEONYX
// axon_data P67-M1 — Core corpus types for IAM data pipeline
//
// IAM FLAGSHIP: every type here feeds into IAM training.
// DataTier mirrors EdisonDB: Critical (constitutional), Personal (IAM persona),
// Noise (general world knowledge).

use serde::{Deserialize, Serialize};

/// Data tier — mirrors EdisonDB DataTier, governs corpus access and training weight.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DataTier {
    /// Constitutional data — immutable IAM articles, sovereignty corpus
    Critical,
    /// Personal data — IAM persona, ItsMe layers, user-specific knowledge
    Personal,
    /// General knowledge — world knowledge, reasoning pairs, public corpus
    Noise,
}

impl DataTier {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Critical => "critical",
            Self::Personal => "personal",
            Self::Noise    => "noise",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "critical" => Some(Self::Critical),
            "personal" => Some(Self::Personal),
            "noise"    => Some(Self::Noise),
            _          => None,
        }
    }

    /// Training weight multiplier for this tier.
    /// Constitutional data is weighted 3x — it must dominate the foundation.
    pub fn training_weight(&self) -> f32 {
        match self {
            Self::Critical => 3.0,
            Self::Personal => 2.0,
            Self::Noise    => 1.0,
        }
    }
}

/// A single corpus entry — the atomic unit of IAM training data.
/// Sourced from either plain text or JSONL pair format.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorpusEntry {
    /// Question / prompt (for QA pairs) or document chunk (for plain text)
    pub text: String,
    /// Answer / completion — empty string for plain text documents
    pub response: String,
    /// Data tier governing training weight and access policy
    pub tier: DataTier,
    /// Source identifier (file path, URL, or corpus name)
    pub source: String,
    /// Semantic tags for curriculum routing
    pub tags: Vec<String>,
    /// Character count (pre-tokenization length estimate)
    pub char_count: usize,
    /// Whether this entry came from a QA pair (true) or plain text (false)
    pub is_pair: bool,
}

impl CorpusEntry {
    /// Build from a plain text chunk.
    pub fn from_text(text: String, source: &str, tier: DataTier) -> Self {
        let char_count = text.chars().count();
        Self {
            text,
            response: String::new(),
            tier,
            source: source.to_string(),
            tags: vec![],
            char_count,
            is_pair: false,
        }
    }

    /// Build from a QA pair.
    pub fn from_pair(
        text: String,
        response: String,
        source: &str,
        tier: DataTier,
        tags: Vec<String>,
    ) -> Self {
        let char_count = text.chars().count() + response.chars().count();
        Self { text, response, tier, source: source.to_string(), tags, char_count, is_pair: true }
    }

    /// Total character count (question + response).
    pub fn total_chars(&self) -> usize {
        self.char_count
    }

    /// Estimated token count (rough: chars / 4, conservative).
    pub fn estimated_tokens(&self) -> usize {
        (self.char_count + 3) / 4
    }

    /// Whether this entry is constitutional (Critical tier).
    pub fn is_constitutional(&self) -> bool {
        self.tier == DataTier::Critical
    }
}

/// Raw JSONL record — wire format for corpus pair files.
/// Format: {"q": "...", "a": "...", "tier": "noise", "source": "...", "tags": [...]}
#[derive(Debug, Deserialize, Serialize)]
pub struct JsonlRecord {
    #[serde(alias = "q", alias = "question", alias = "prompt")]
    pub q: String,
    #[serde(alias = "a", alias = "answer", alias = "response", alias = "completion")]
    pub a: String,
    #[serde(default = "default_tier")]
    pub tier: String,
    #[serde(default)]
    pub source: String,
    #[serde(default)]
    pub tags: Vec<String>,
}

fn default_tier() -> String { "noise".to_string() }

/// Corpus statistics — summary of ingested data.
#[derive(Debug, Clone, Default)]
pub struct CorpusStats {
    pub total_entries: usize,
    pub pair_entries: usize,
    pub text_entries: usize,
    pub critical_entries: usize,
    pub personal_entries: usize,
    pub noise_entries: usize,
    pub total_chars: usize,
    pub estimated_tokens: usize,
    pub sources: Vec<String>,
}

impl CorpusStats {
    pub fn update(&mut self, entry: &CorpusEntry) {
        self.total_entries += 1;
        if entry.is_pair { self.pair_entries += 1; } else { self.text_entries += 1; }
        match entry.tier {
            DataTier::Critical => self.critical_entries += 1,
            DataTier::Personal => self.personal_entries += 1,
            DataTier::Noise    => self.noise_entries += 1,
        }
        self.total_chars += entry.char_count;
        self.estimated_tokens += entry.estimated_tokens();
        if !self.sources.contains(&entry.source) {
            self.sources.push(entry.source.clone());
        }
    }
}

/// Corpus ingestion error.
#[derive(Debug, Clone, PartialEq)]
pub enum CorpusError {
    InvalidJsonl { line: usize, reason: String },
    InvalidTier(String),
    EmptyEntry,
    IoError(String),
    InvalidFormat(String),
}

impl std::fmt::Display for CorpusError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::InvalidJsonl { line, reason } =>
                write!(f, "JSONL parse error at line {}: {}", line, reason),
            Self::InvalidTier(s) => write!(f, "invalid tier: {}", s),
            Self::EmptyEntry     => write!(f, "empty corpus entry"),
            Self::IoError(s)     => write!(f, "I/O error: {}", s),
            Self::InvalidFormat(s) => write!(f, "invalid format: {}", s),
        }
    }
}
