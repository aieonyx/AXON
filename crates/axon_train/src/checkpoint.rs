// Copyright (c) 2026 Edison Lepiten / AIEONYX
// axon_train P68-M3 — Checkpoint save/load (.axckpt format)
//
// .axckpt format:
//   Header: "#axckpt v1 <step> <loss> <lr> <corpus_pos> <tokenizer_hash>\n"
//   Body:   JSON-serialized parameter tensors (f32 values)
//   Footer: sovereign hash of (header + body)
//
// Bit-exact resume: RNG state + sampler position serialized.
// Auto-checkpoint every 30 min wall-clock (night-shift friendly).

use serde::{Deserialize, Serialize};
use axon_data::shard::sovereign_hash;

pub const AXCKPT_VERSION: u8 = 1;

/// A named parameter tensor (simplified — no actual tensor dep in P68).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterTensor {
    pub name: String,
    pub shape: Vec<usize>,
    pub data: Vec<f32>,
}

impl ParameterTensor {
    pub fn new(name: &str, shape: Vec<usize>, data: Vec<f32>) -> Self {
        Self { name: name.to_string(), shape, data }
    }

    pub fn zeros(name: &str, shape: Vec<usize>) -> Self {
        let n: usize = shape.iter().product();
        Self::new(name, shape, vec![0.0; n])
    }

    pub fn numel(&self) -> usize { self.data.len() }
}

/// Training state for checkpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckpointState {
    pub version: u8,
    pub step: usize,
    pub epoch: usize,
    pub loss: f32,
    pub best_loss: f32,
    pub learning_rate: f32,
    pub corpus_position: usize,
    pub tokenizer_hash: String,
    pub corpus_name: String,
    pub stage: String,
    /// Deterministic RNG seed for exact resume
    pub rng_seed: u64,
    /// Parameter tensors (model weights)
    pub parameters: Vec<ParameterTensor>,
    /// Optimizer state (Adam moments)
    pub optimizer_m: Vec<ParameterTensor>, // first moment
    pub optimizer_v: Vec<ParameterTensor>, // second moment
}

impl CheckpointState {
    pub fn new(
        step: usize,
        epoch: usize,
        loss: f32,
        best_loss: f32,
        learning_rate: f32,
        corpus_position: usize,
        tokenizer_hash: &str,
        corpus_name: &str,
        stage: &str,
        rng_seed: u64,
    ) -> Self {
        Self {
            version: AXCKPT_VERSION,
            step, epoch, loss, best_loss, learning_rate,
            corpus_position,
            tokenizer_hash: tokenizer_hash.to_string(),
            corpus_name: corpus_name.to_string(),
            stage: stage.to_string(),
            rng_seed,
            parameters: vec![],
            optimizer_m: vec![],
            optimizer_v: vec![],
        }
    }

    pub fn add_param(&mut self, p: ParameterTensor) { self.parameters.push(p); }
    pub fn add_moment_m(&mut self, p: ParameterTensor) { self.optimizer_m.push(p); }
    pub fn add_moment_v(&mut self, p: ParameterTensor) { self.optimizer_v.push(p); }
    pub fn param_count(&self) -> usize { self.parameters.iter().map(|p| p.numel()).sum() }
}

/// Serialize a checkpoint to .axckpt bytes.
pub fn save_checkpoint(state: &CheckpointState) -> Result<Vec<u8>, CheckpointError> {
    // Header line
    let header = format!(
        "#axckpt v{} step={} epoch={} loss={:.6} lr={:.8} corpus_pos={} tok_hash={} corpus={} stage={}\n",
        state.version, state.step, state.epoch, state.loss, state.learning_rate,
        state.corpus_position, state.tokenizer_hash, state.corpus_name, state.stage
    );

    // Body: JSON
    let body = serde_json::to_string(state)
        .map_err(|e| CheckpointError::SerializeError(e.to_string()))?;

    // Footer: sovereign hash
    let mut to_hash = header.as_bytes().to_vec();
    to_hash.extend_from_slice(body.as_bytes());
    let hash = sovereign_hash(&to_hash);
    let hash_hex: String = hash.iter().map(|b| format!("{:02x}", b)).collect();

    let mut out = Vec::new();
    out.extend_from_slice(header.as_bytes());
    out.extend_from_slice(body.as_bytes());
    out.push(b'\n');
    out.extend_from_slice(format!("#hash {}\n", hash_hex).as_bytes());

    Ok(out)
}

/// Deserialize a checkpoint from .axckpt bytes.
pub fn load_checkpoint(data: &[u8]) -> Result<CheckpointState, CheckpointError> {
    let text = std::str::from_utf8(data)
        .map_err(|e| CheckpointError::ParseError(e.to_string()))?;

    let mut lines = text.lines();

    // Parse header
    let header = lines.next().ok_or(CheckpointError::EmptyCheckpoint)?;
    if !header.starts_with("#axckpt v") {
        return Err(CheckpointError::InvalidHeader(header.to_string()));
    }

    // Collect body lines (until #hash line)
    let mut body_lines = Vec::new();
    let mut stored_hash = String::new();
    for line in lines {
        if line.starts_with("#hash ") {
            stored_hash = line[6..].to_string();
        } else {
            body_lines.push(line);
        }
    }
    let body = body_lines.join("\n");

    // Verify hash
    if !stored_hash.is_empty() {
        let mut to_hash = header.as_bytes().to_vec();
        to_hash.push(b'\n');
        to_hash.extend_from_slice(body.as_bytes());
        let computed = sovereign_hash(&to_hash);
        let computed_hex: String = computed.iter().map(|b| format!("{:02x}", b)).collect();
        if computed_hex != stored_hash {
            return Err(CheckpointError::HashMismatch);
        }
    }

    let state: CheckpointState = serde_json::from_str(&body)
        .map_err(|e| CheckpointError::ParseError(e.to_string()))?;

    if state.version != AXCKPT_VERSION {
        return Err(CheckpointError::VersionMismatch(state.version));
    }

    Ok(state)
}

#[derive(Debug, Clone, PartialEq)]
pub enum CheckpointError {
    EmptyCheckpoint,
    InvalidHeader(String),
    VersionMismatch(u8),
    HashMismatch,
    SerializeError(String),
    ParseError(String),
    IoError(String),
}

impl std::fmt::Display for CheckpointError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::EmptyCheckpoint       => write!(f, "empty checkpoint"),
            Self::InvalidHeader(s)      => write!(f, "invalid header: {}", s),
            Self::VersionMismatch(v)    => write!(f, "unsupported version: {}", v),
            Self::HashMismatch          => write!(f, "checkpoint integrity check failed"),
            Self::SerializeError(s)     => write!(f, "serialize error: {}", s),
            Self::ParseError(s)         => write!(f, "parse error: {}", s),
            Self::IoError(s)            => write!(f, "I/O error: {}", s),
        }
    }
}
