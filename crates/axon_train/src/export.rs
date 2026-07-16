// Copyright (c) 2026 Edison Lepiten / AIEONYX
// axon_train P68-M5 — .iam export (sovereign brain file)
//
// .iam format (IAM Brain File):
//   Magic:   b"IAM1" (4 bytes)
//   Header:  JSON metadata (arch, tokenizer hash, constitutional hash, version)
//   Tensors: named f32 arrays (safetensors-style layout)
//   Sig:     Ed25519 detached signature stub (64 bytes, zeroed until ceremony)
//
// Production signing: AIEONYX Root Key (air-gapped ceremony, USB-A primary).
// Stub here: signature field zeroed, marked UNSIGNED in header.
// Never deploy unsigned .iam in production.

use serde::{Deserialize, Serialize};
use axon_data::shard::sovereign_hash;
use crate::checkpoint::CheckpointState;

pub const IAM_MAGIC: [u8; 4] = [b'I', b'A', b'M', b'1'];
pub const IAM_VERSION: u8 = 1;
pub const SIG_LEN: usize = 64;

/// .iam file header metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IamHeader {
    pub version: u8,
    pub stage: String,
    pub param_count: usize,
    pub vocab_size: usize,
    pub d_model: usize,
    pub n_ssm_layers: usize,
    pub n_attn_layers: usize,
    pub tokenizer_hash: String,
    pub constitutional_hash: String,
    pub corpus_name: String,
    pub training_steps: usize,
    pub final_loss: f32,
    pub signed: bool,
    pub signer_pubkey_hex: String,
}

/// Export a trained checkpoint to .iam format.
pub fn export_iam(
    state: &CheckpointState,
    vocab_size: usize,
    d_model: usize,
    n_ssm_layers: usize,
    n_attn_layers: usize,
) -> Result<Vec<u8>, ExportError> {
    // Compute constitutional hash (hash of all parameter names — integrity anchor)
    let param_names: Vec<u8> = state.parameters.iter()
        .flat_map(|p| p.name.as_bytes().iter().chain(&[0u8]).copied())
        .collect();
    let constitutional_hash = sovereign_hash(&param_names);
    let constitutional_hash_hex: String = constitutional_hash.iter()
        .map(|b| format!("{:02x}", b)).collect();

    let header = IamHeader {
        version: IAM_VERSION,
        stage: state.stage.clone(),
        param_count: state.param_count(),
        vocab_size,
        d_model,
        n_ssm_layers,
        n_attn_layers,
        tokenizer_hash: state.tokenizer_hash.clone(),
        constitutional_hash: constitutional_hash_hex,
        corpus_name: state.corpus_name.clone(),
        training_steps: state.step,
        final_loss: state.loss,
        signed: false, // UNSIGNED stub
        signer_pubkey_hex: "0".repeat(64), // placeholder
    };

    let header_json = serde_json::to_string(&header)
        .map_err(|e| ExportError::SerializeError(e.to_string()))?;

    // Serialize parameter tensors
    let tensors_json = serde_json::to_string(&state.parameters)
        .map_err(|e| ExportError::SerializeError(e.to_string()))?;

    // Signature stub (zeroed — production signing requires air-gapped ceremony)
    let sig_stub = [0u8; SIG_LEN];

    // Assemble .iam file
    let mut out = Vec::new();
    out.extend_from_slice(&IAM_MAGIC);
    out.push(IAM_VERSION);

    // Header length prefix (4 bytes LE)
    let header_bytes = header_json.as_bytes();
    out.extend_from_slice(&(header_bytes.len() as u32).to_le_bytes());
    out.extend_from_slice(header_bytes);

    // Tensor data length prefix (4 bytes LE)
    let tensor_bytes = tensors_json.as_bytes();
    out.extend_from_slice(&(tensor_bytes.len() as u32).to_le_bytes());
    out.extend_from_slice(tensor_bytes);

    // Signature stub
    out.extend_from_slice(&sig_stub);

    Ok(out)
}

/// Parse .iam header from file bytes.
pub fn parse_iam_header(data: &[u8]) -> Result<IamHeader, ExportError> {
    if data.len() < 9 {
        return Err(ExportError::TooShort(data.len()));
    }
    if &data[0..4] != &IAM_MAGIC {
        return Err(ExportError::InvalidMagic([data[0], data[1], data[2], data[3]]));
    }
    if data[4] != IAM_VERSION {
        return Err(ExportError::VersionMismatch(data[4]));
    }
    let header_len = u32::from_le_bytes([data[5], data[6], data[7], data[8]]) as usize;
    if data.len() < 9 + header_len {
        return Err(ExportError::TooShort(data.len()));
    }
    let header_json = std::str::from_utf8(&data[9..9+header_len])
        .map_err(|e| ExportError::ParseError(e.to_string()))?;
    serde_json::from_str(header_json)
        .map_err(|e| ExportError::ParseError(e.to_string()))
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExportError {
    TooShort(usize),
    InvalidMagic([u8; 4]),
    VersionMismatch(u8),
    SerializeError(String),
    ParseError(String),
    UnsignedExport,
}

impl std::fmt::Display for ExportError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::TooShort(n)        => write!(f, ".iam too short: {} bytes", n),
            Self::InvalidMagic(m)    => write!(f, "invalid magic: {:?}", m),
            Self::VersionMismatch(v) => write!(f, "unsupported version: {}", v),
            Self::SerializeError(s)  => write!(f, "serialize error: {}", s),
            Self::ParseError(s)      => write!(f, "parse error: {}", s),
            Self::UnsignedExport     => write!(f, ".iam unsigned — run key ceremony before deployment"),
        }
    }
}
