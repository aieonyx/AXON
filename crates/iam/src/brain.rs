// Copyright (c) 2026 Edison Lepiten / AIEONYX
// IAM brain export — .iam format
//
// The .iam file is the trained brain — the artifact that users
// download and place in their IAM /brain folder.
//
// Format:
//   Magic:   "IAM1" (4 bytes)
//   Header:  JSON metadata (length-prefixed)
//   Weights: raw f32 tensors (little-endian)
//   Sig:     Ed25519 signature stub (64 zero bytes until air-gapped ceremony)

use std::fs;
use std::path::Path;
use crate::model::IamModel;

pub const IAM_MAGIC: &[u8; 4] = b"IAM1";
pub const SIG_LEN: usize = 64;

/// Export a trained IAMSeed model to .iam format.
pub fn export_brain(
    model: &IamModel,
    output_path: &Path,
    step: usize,
    loss: f32,
    tokenizer_hash: &str,
) -> Result<(), String> {
    let mut buf = Vec::new();

    // Magic
    buf.extend_from_slice(IAM_MAGIC);

    // Header JSON
    let header = serde_json::json!({
        "version":        "1.0",
        "model":          "IAMSeed",
        "param_count":    model.param_count(),
        "vocab_size":     model.config.vocab_size,
        "d_model":        model.config.d_model,
        "n_ssm_layers":   model.config.n_ssm_layers,
        "n_attn_layers":  model.config.n_attn_layers,
        "n_layers":       model.config.n_layers,
        "step":           step,
        "loss":           loss,
        "tokenizer_hash": tokenizer_hash,
        "author":         "Edison Lepiten / AIEONYX",
        "license":        "AIEONYX Sovereign Model License",
        "epoch":          "Wisdom is the Beginning.",
        "constitution":   "AIEONYX-SPEC-IAM-v1.0",
    });
    let header_bytes = header.to_string().into_bytes();
    let header_len = header_bytes.len() as u32;
    buf.extend_from_slice(&header_len.to_le_bytes());
    buf.extend_from_slice(&header_bytes);

    // Weights — embedding
    for &w in &model.embedding.weight {
        buf.extend_from_slice(&w.to_le_bytes());
    }

    // Weights — layers
    for layer in &model.layers {
        match layer {
            crate::model::Layer::Ssm(l) => {
                for &w in &l.in_proj  { buf.extend_from_slice(&w.to_le_bytes()); }
                for &w in &l.out_proj { buf.extend_from_slice(&w.to_le_bytes()); }
                for &w in &l.a_log    { buf.extend_from_slice(&w.to_le_bytes()); }
                for &w in &l.d_ssm    { buf.extend_from_slice(&w.to_le_bytes()); }
                for &w in &l.norm.weight { buf.extend_from_slice(&w.to_le_bytes()); }
                for &w in &l.norm.bias   { buf.extend_from_slice(&w.to_le_bytes()); }
            }
            crate::model::Layer::Attn(l) => {
                for &w in &l.q_proj  { buf.extend_from_slice(&w.to_le_bytes()); }
                for &w in &l.k_proj  { buf.extend_from_slice(&w.to_le_bytes()); }
                for &w in &l.v_proj  { buf.extend_from_slice(&w.to_le_bytes()); }
                for &w in &l.o_proj  { buf.extend_from_slice(&w.to_le_bytes()); }
                for &w in &l.ff_up   { buf.extend_from_slice(&w.to_le_bytes()); }
                for &w in &l.ff_down { buf.extend_from_slice(&w.to_le_bytes()); }
                for &w in &l.norm1.weight { buf.extend_from_slice(&w.to_le_bytes()); }
                for &w in &l.norm2.weight { buf.extend_from_slice(&w.to_le_bytes()); }
            }
        }
    }

    // Weights — LM head
    for &w in &model.lm_head.weight   { buf.extend_from_slice(&w.to_le_bytes()); }
    for &w in &model.lm_head.norm.weight { buf.extend_from_slice(&w.to_le_bytes()); }

    // Signature stub (zeroed until air-gapped Root Key ceremony)
    buf.extend_from_slice(&[0u8; SIG_LEN]);

    fs::write(output_path, &buf)
        .map_err(|e| format!("cannot write .iam: {}", e))?;

    println!("  Brain exported: {} ({:.1} MB)",
        output_path.display(), buf.len() as f32 / 1_048_576.0);
    println!("  Sig stub: zeroed (sign with Root Key after air-gapped ceremony)");

    Ok(())
}
