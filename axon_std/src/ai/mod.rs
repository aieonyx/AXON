//! # axon::ai
//!
//! AXON AI runtime module — sovereign, local-first inference.
//!
//! All inference runs through Ollama (localhost:11434).
//! Zero cloud dependency. Model selection is caller-controlled.
//! The LLM is never in the Trusted Computing Base.
//!
//! ## Architecture principle (carried from axon_ai Phase 5)
//!
//! "AI is ASSISTANT. Formal methods are GATE."
//! axon::ai provides runtime inference capability.
//! axon::verify provides the deterministic gate.
//!
//! ## Security-weighted inference (Bio DNA: Transcription Factor Specificity)
//!
//! High-security contexts get full inference depth.
//! Low-risk contexts get fast-pass (minimal tokens, faster response).
//! The caller declares weight via InferenceWeight.
//!
//! ## Quick start
//!
//! ```rust,ignore
//! use axon_std::ai::{infer, embed, infer_weighted, InferenceWeight};
//!
//! let reply = infer("Explain seL4 capabilities", "llama3.2").unwrap();
//! let vec   = embed("sovereign computing", "nomic-embed-text").unwrap();
//! let reply = infer_weighted("Audit this function", "llama3.2",
//!                            InferenceWeight::SecurityCritical).unwrap();
//! ```

pub mod client;
pub mod embeddings;
pub mod inference;
pub mod model;
pub mod weight;
pub mod aipl;

pub use inference::{infer, infer_weighted};
pub use embeddings::embed;
pub use model::{ModelHandle, model_list, model_load};
pub use weight::InferenceWeight;
pub use aipl::{AiplSuggestion, aipl_suggest};

use std::fmt;

/// Result type for all axon::ai operations.
pub type AiResult<T> = Result<T, AiError>;

/// Errors from the axon::ai runtime module.
#[derive(Debug, Clone)]
pub enum AiError {
    /// Ollama is not reachable at localhost:11434.
    /// NOT a fatal error — axon::verify still runs without Ollama.
    OllamaUnavailable(String),
    /// Ollama returned a response that could not be parsed.
    MalformedResponse(String),
    /// The requested model is not loaded in Ollama.
    ModelNotFound(String),
    /// An I/O error occurred during the HTTP request.
    IoError(String),
    /// The inference request was rejected by the security weight policy.
    WeightPolicyViolation(String),
}

impl fmt::Display for AiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AiError::OllamaUnavailable(m) =>
                write!(f, "axon::ai: Ollama unavailable at localhost:11434 — {m}"),
            AiError::MalformedResponse(m) =>
                write!(f, "axon::ai: malformed Ollama response — {m}"),
            AiError::ModelNotFound(m) =>
                write!(f, "axon::ai: model not found — {m}"),
            AiError::IoError(m) =>
                write!(f, "axon::ai: I/O error — {m}"),
            AiError::WeightPolicyViolation(m) =>
                write!(f, "axon::ai: weight policy violation — {m}"),
        }
    }
}

impl std::error::Error for AiError {}
