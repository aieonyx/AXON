// Copyright (c) 2026 Edison Lepiten / AIEONYX
// axon_data P67-M1 — Corpus ingestion: plain text + JSONL pair format
//
// IAM FLAGSHIP: this is the mouth of the pipeline.
// Everything IAM learns enters through here.

use crate::types::{CorpusEntry, CorpusError, CorpusStats, DataTier, JsonlRecord};

/// Ingestion options — controls how raw files are parsed.
#[derive(Debug, Clone)]
pub struct IngestOptions {
    /// Default tier for entries that don't specify one
    pub default_tier: DataTier,
    /// Default source label
    pub default_source: String,
    /// Minimum character count to accept an entry (filters noise)
    pub min_chars: usize,
    /// Maximum character count per entry (splits longer text)
    pub max_chars: usize,
    /// Skip entries with empty responses in QA pairs
    pub skip_empty_response: bool,
    /// Chunk size for plain text splitting (in chars)
    pub chunk_size: usize,
}

impl Default for IngestOptions {
    fn default() -> Self {
        Self {
            default_tier: DataTier::Noise,
            default_source: "unknown".to_string(),
            min_chars: 10,
            max_chars: 8192,
            skip_empty_response: true,
            chunk_size: 2048,
        }
    }
}

impl IngestOptions {
    pub fn constitutional() -> Self {
        Self {
            default_tier: DataTier::Critical,
            default_source: "constitutional".to_string(),
            min_chars: 5,
            max_chars: 4096,
            skip_empty_response: false,
            chunk_size: 1024,
        }
    }
}

/// Ingest a JSONL string — one JSON object per line.
/// Format: {"q": "...", "a": "...", "tier": "noise", "source": "...", "tags": [...]}
pub fn ingest_jsonl(
    content: &str,
    source: &str,
    opts: &IngestOptions,
) -> (Vec<CorpusEntry>, Vec<CorpusError>) {
    let mut entries = Vec::new();
    let mut errors = Vec::new();

    for (line_idx, line) in content.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with("//") || line.starts_with('#') {
            continue;
        }

        let record: JsonlRecord = match serde_json::from_str(line) {
            Ok(r) => r,
            Err(e) => {
                errors.push(CorpusError::InvalidJsonl {
                    line: line_idx + 1,
                    reason: e.to_string(),
                });
                continue;
            }
        };

        // Validate tier
        let tier = if record.tier.is_empty() {
            opts.default_tier.clone()
        } else {
            match DataTier::from_str(&record.tier) {
                Some(t) => t,
                None => {
                    errors.push(CorpusError::InvalidTier(record.tier.clone()));
                    continue;
                }
            }
        };

        // Validate content
        if record.q.trim().is_empty() {
            errors.push(CorpusError::EmptyEntry);
            continue;
        }

        if opts.skip_empty_response && record.a.trim().is_empty() {
            continue;
        }

        let total_chars = record.q.len() + record.a.len();
        if total_chars < opts.min_chars {
            continue;
        }

        let src = if record.source.is_empty() {
            source.to_string()
        } else {
            record.source
        };

        entries.push(CorpusEntry::from_pair(
            record.q,
            record.a,
            &src,
            tier,
            record.tags,
        ));
    }

    (entries, errors)
}

/// Ingest a plain text document — split into chunks of `opts.chunk_size` chars.
/// Tries to split on paragraph boundaries (double newline) first.
pub fn ingest_text(
    content: &str,
    source: &str,
    opts: &IngestOptions,
) -> Vec<CorpusEntry> {
    let mut entries = Vec::new();

    // Try paragraph splitting first
    let paragraphs: Vec<&str> = content.split("\n\n").collect();
    let mut current_chunk = String::new();

    for para in paragraphs {
        let para = para.trim();
        if para.is_empty() { continue; }

        if current_chunk.len() + para.len() > opts.chunk_size && !current_chunk.is_empty() {
            // Flush current chunk
            if current_chunk.len() >= opts.min_chars {
                entries.push(CorpusEntry::from_text(
                    current_chunk.clone(),
                    source,
                    opts.default_tier.clone(),
                ));
            }
            current_chunk.clear();
        }

        if !current_chunk.is_empty() { current_chunk.push('\n'); }
        current_chunk.push_str(para);

        // If single paragraph exceeds max, split by sentence
        if current_chunk.len() > opts.max_chars {
            for chunk in split_by_sentence(&current_chunk, opts.chunk_size) {
                if chunk.len() >= opts.min_chars {
                    entries.push(CorpusEntry::from_text(
                        chunk,
                        source,
                        opts.default_tier.clone(),
                    ));
                }
            }
            current_chunk.clear();
        }
    }

    // Flush remaining
    if current_chunk.len() >= opts.min_chars {
        entries.push(CorpusEntry::from_text(
            current_chunk,
            source,
            opts.default_tier.clone(),
        ));
    }

    entries
}

/// Split text by sentence boundaries, targeting `chunk_size` chars per chunk.
fn split_by_sentence(text: &str, chunk_size: usize) -> Vec<String> {
    let mut chunks = Vec::new();
    let mut current = String::new();

    for sentence in text.split(". ") {
        if current.len() + sentence.len() > chunk_size && !current.is_empty() {
            chunks.push(current.trim().to_string());
            current.clear();
        }
        if !current.is_empty() { current.push(' '); }
        current.push_str(sentence);
        if !sentence.ends_with('.') { current.push('.'); }
    }

    if !current.trim().is_empty() {
        chunks.push(current.trim().to_string());
    }

    chunks
}

/// Compute corpus statistics from a slice of entries.
pub fn compute_stats(entries: &[CorpusEntry]) -> CorpusStats {
    let mut stats = CorpusStats::default();
    for entry in entries {
        stats.update(entry);
    }
    stats
}

/// Validate a corpus entry — returns Ok or a CorpusError.
pub fn validate_entry(entry: &CorpusEntry, opts: &IngestOptions) -> Result<(), CorpusError> {
    if entry.text.trim().is_empty() {
        return Err(CorpusError::EmptyEntry);
    }
    if entry.char_count < opts.min_chars {
        return Err(CorpusError::InvalidFormat(
            format!("entry too short: {} < {} chars", entry.char_count, opts.min_chars)
        ));
    }
    if entry.source.is_empty() {
        return Err(CorpusError::InvalidFormat("entry has no source".into()));
    }
    Ok(())
}
