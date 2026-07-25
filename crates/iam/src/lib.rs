// Copyright (c) 2026 Edison Lepiten / AIEONYX
// IAM — Intelligent Assistant to Man
// IAMSeed v0.1.0 — 120M parameter sovereign intelligence
//
// Architecture: Hybrid SSM/Attention
//   20 SSM layers  — efficient sequence modeling (long-range context)
//    4 Attn layers — precise reasoning (constitution enforcement points)
//   d_model = 512
//   vocab   = 8,000 (sovereign BPE, IAM-M2)
//
// Constitutional special tokens enforced at attention layers:
//   <intent>/<intent>   — Article 0 gate
//   <conf_g/y/r>        — Article 2 confidence markers
//   <iam>               — IAM identity marker
//
// Epoch declaration: "Wisdom is the Beginning."

pub mod model;
pub mod config;
pub mod brain;

pub use model::IamModel;
pub use config::IamConfig;
