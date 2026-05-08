// ============================================================
// axon_ai — error.rs
// Copyright © 2026 Edison Lepiten — AIEONYX
// ============================================================

#[derive(Debug, Clone)]
pub enum AiError {
    /// Ollama not reachable — AI assistance unavailable
    /// This is NOT a compile error — the formal verifier still runs
    OllamaUnavailable(String),
    /// LLM returned malformed output — use fallback
    MalformedResponse(String),
    /// Formal verifier found a constraint violation
    ConstraintViolation(ConstraintViolation),
    /// Parse error in the AXON source
    ParseError(String),
}

#[derive(Debug, Clone)]
pub struct ConstraintViolation {
    pub constraint     : String,
    pub function_name  : String,
    pub violating_path : String,
    pub suggestion     : String,
}

impl std::fmt::Display for AiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AiError::OllamaUnavailable(msg) =>
                write!(f, "AI assistance unavailable (Ollama): {}\n  → Formal verification still runs", msg),
            AiError::MalformedResponse(msg) =>
                write!(f, "AI returned malformed output: {}\n  → Using fallback mode", msg),
            AiError::ConstraintViolation(v) => {
                writeln!(f, "error[E411]: @ai.intent constraint violated")?;
                writeln!(f, "  → fn {} claims: {}", v.function_name, v.constraint)?;
                writeln!(f, "  → violating path: {}", v.violating_path)?;
                write!(f, "  → hint: {}", v.suggestion)
            }
            AiError::ParseError(msg) =>
                write!(f, "Parse error: {}", msg),
        }
    }
}
