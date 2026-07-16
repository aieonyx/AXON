// Copyright (c) 2026 Edison Lepiten / AIEONYX
// axon_data P67-M4 — Batch iterator for IAM training
//
// IAM FLAGSHIP: this feeds the training loop.
// Every sequence IAM trains on is assembled here.
//
// Key design decisions from IAM spec:
// - SEQ_LEN = 4096 tokens (SSM context window)
// - Document boundaries tracked (SSM state reset markers)
// - BOS/EOS inserted around each document
// - DataTier-weighted mixture sampling (Critical 3x, Personal 2x, Noise 1x)
// - Constitutional entries always included before general knowledge

use crate::tokenizer::{BOS_ID, EOS_ID, PAD_ID};
use crate::types::DataTier;

pub const SEQ_LEN: usize = 4096;

// ── Training sequence ─────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct TrainingSequence {
    pub tokens: Vec<u32>,
    pub attention_mask: Vec<u8>,
    pub doc_boundaries: Vec<usize>,
    pub real_token_count: usize,
    pub tier: DataTier,
}

impl TrainingSequence {
    pub fn empty() -> Self {
        Self {
            tokens: vec![PAD_ID; SEQ_LEN],
            attention_mask: vec![0; SEQ_LEN],
            doc_boundaries: vec![],
            real_token_count: 0,
            tier: DataTier::Noise,
        }
    }

    pub fn utilization(&self) -> f32 {
        self.real_token_count as f32 / SEQ_LEN as f32
    }

    pub fn is_useful(&self) -> bool {
        self.real_token_count >= SEQ_LEN / 2
    }
}

// ── Training batch ────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct TrainingBatch {
    pub sequences: Vec<TrainingSequence>,
    pub batch_size: usize,
    pub epoch: usize,
    pub step: usize,
}

impl TrainingBatch {
    pub fn new(sequences: Vec<TrainingSequence>, epoch: usize, step: usize) -> Self {
        let batch_size = sequences.len();
        Self { sequences, batch_size, epoch, step }
    }

    pub fn total_tokens(&self) -> usize {
        self.sequences.iter().map(|s| s.real_token_count).sum()
    }

    pub fn avg_utilization(&self) -> f32 {
        if self.sequences.is_empty() { return 0.0; }
        self.sequences.iter().map(|s| s.utilization()).sum::<f32>()
            / self.sequences.len() as f32
    }
}

// ── Tokenized entry ───────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct TokenizedEntry {
    pub tokens: Vec<u32>,
    pub tier: DataTier,
    pub source: String,
}

impl TokenizedEntry {
    pub fn new(tokens: Vec<u32>, tier: DataTier, source: &str) -> Self {
        Self { tokens, tier, source: source.to_string() }
    }
}

// ── Sequence packer ───────────────────────────────────────────────────────────

pub fn pack_sequences(entries: &[TokenizedEntry]) -> Vec<TrainingSequence> {
    let mut sequences = Vec::new();
    let mut current_tokens: Vec<u32> = Vec::with_capacity(SEQ_LEN);
    let mut current_boundaries: Vec<usize> = Vec::new();
    let mut current_tier = DataTier::Noise;
    let mut critical_count = 0usize;

    for entry in entries {
        let doc: Vec<u32> = std::iter::once(BOS_ID)
            .chain(entry.tokens.iter().cloned())
            .chain(std::iter::once(EOS_ID))
            .collect();

        let doc = if doc.len() > SEQ_LEN { doc[..SEQ_LEN].to_vec() } else { doc };

        if current_tokens.len() + doc.len() > SEQ_LEN && !current_tokens.is_empty() {
            sequences.push(finalize_sequence(
                current_tokens, current_boundaries, current_tier, critical_count,
            ));
            current_tokens = Vec::with_capacity(SEQ_LEN);
            current_boundaries = Vec::new();
            critical_count = 0;
            current_tier = DataTier::Noise;
        }

        current_boundaries.push(current_tokens.len());

        match entry.tier {
            DataTier::Critical => {
                critical_count += 1;
                current_tier = DataTier::Critical;
            }
            DataTier::Personal if current_tier != DataTier::Critical => {
                current_tier = DataTier::Personal;
            }
            _ => {}
        }

        current_tokens.extend_from_slice(&doc);
    }

    if !current_tokens.is_empty() {
        sequences.push(finalize_sequence(
            current_tokens, current_boundaries, current_tier, critical_count,
        ));
    }

    sequences
}

fn finalize_sequence(
    mut tokens: Vec<u32>,
    boundaries: Vec<usize>,
    tier: DataTier,
    _critical_count: usize,
) -> TrainingSequence {
    let real_count = tokens.len();
    tokens.resize(SEQ_LEN, PAD_ID);
    let mut attention_mask = vec![1u8; real_count];
    attention_mask.resize(SEQ_LEN, 0);
    TrainingSequence {
        tokens,
        attention_mask,
        doc_boundaries: boundaries,
        real_token_count: real_count,
        tier,
    }
}

// ── Mixture sampler ───────────────────────────────────────────────────────────

pub struct MixtureSampler {
    critical: Vec<TokenizedEntry>,
    personal: Vec<TokenizedEntry>,
    noise: Vec<TokenizedEntry>,
    epoch: usize,
}

impl MixtureSampler {
    pub fn new(entries: Vec<TokenizedEntry>) -> Self {
        let mut critical = Vec::new();
        let mut personal = Vec::new();
        let mut noise = Vec::new();
        for e in entries {
            match e.tier {
                DataTier::Critical => critical.push(e),
                DataTier::Personal => personal.push(e),
                DataTier::Noise    => noise.push(e),
            }
        }
        Self { critical, personal, noise, epoch: 0 }
    }

    pub fn build_epoch_sequence(&mut self) -> Vec<TokenizedEntry> {
        let mut sequence = Vec::new();
        for _ in 0..3 { sequence.extend(self.critical.iter().cloned()); }
        for _ in 0..2 { sequence.extend(self.personal.iter().cloned()); }
        sequence.extend(self.noise.iter().cloned());
        lcg_shuffle(&mut sequence, self.epoch as u64 * 0x9e3779b97f4a7c15 + 1);
        self.epoch += 1;
        sequence
    }

    pub fn epoch(&self) -> usize { self.epoch }
    pub fn critical_count(&self) -> usize { self.critical.len() }
    pub fn personal_count(&self) -> usize { self.personal.len() }
    pub fn noise_count(&self) -> usize { self.noise.len() }

    pub fn epoch_size(&self) -> usize {
        self.critical.len() * 3 + self.personal.len() * 2 + self.noise.len()
    }
}

// ── Batch iterator ────────────────────────────────────────────────────────────

pub struct BatchIterator {
    sequences: Vec<TrainingSequence>,
    batch_size: usize,
    current_idx: usize,
    epoch: usize,
    step: usize,
}

impl BatchIterator {
    pub fn new(sequences: Vec<TrainingSequence>, batch_size: usize) -> Self {
        Self { sequences, batch_size, current_idx: 0, epoch: 0, step: 0 }
    }

    pub fn batches_per_epoch(&self) -> usize {
        (self.sequences.len() + self.batch_size - 1) / self.batch_size
    }

    pub fn sequence_count(&self) -> usize { self.sequences.len() }

    pub fn has_next(&self) -> bool { self.current_idx < self.sequences.len() }

    pub fn next_batch(&mut self) -> Option<TrainingBatch> {
        if self.current_idx >= self.sequences.len() { return None; }
        let end = (self.current_idx + self.batch_size).min(self.sequences.len());
        let batch_seqs = self.sequences[self.current_idx..end].to_vec();
        let batch = TrainingBatch::new(batch_seqs, self.epoch, self.step);
        self.current_idx = end;
        self.step += 1;
        Some(batch)
    }

    pub fn reset_epoch(&mut self) {
        self.current_idx = 0;
        self.epoch += 1;
    }

    pub fn epoch(&self) -> usize { self.epoch }
    pub fn step(&self) -> usize { self.step }
}

// ── LCG shuffle ──────────────────────────────────────────────────────────────
fn lcg_shuffle<T>(v: &mut Vec<T>, seed: u64) {
    let n = v.len();
    if n <= 1 { return; }
    let mut state = seed ^ 0xcafe_babe_dead_beef;
    for i in (1..n).rev() {
        state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        let j = (state >> 33) as usize % (i + 1);
        v.swap(i, j);
    }
}
