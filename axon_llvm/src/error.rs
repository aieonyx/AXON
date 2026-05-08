// ============================================================
// axon_llvm — error.rs
// Copyright © 2026 Edison Lepiten — AIEONYX
// ============================================================

#[derive(Debug, Clone)]
pub enum LlvmCodegenError {
    ParseErrors(Vec<String>),
    Unsupported(String),
    IoError(String),
}

impl std::fmt::Display for LlvmCodegenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LlvmCodegenError::ParseErrors(errs) => {
                writeln!(f, "Parse errors ({}):", errs.len())?;
                for e in errs { writeln!(f, "  {}", e)?; }
                Ok(())
            }
            LlvmCodegenError::Unsupported(msg) =>
                write!(f, "Unsupported: {}", msg),
            LlvmCodegenError::IoError(msg) =>
                write!(f, "IO error: {}", msg),
        }
    }
}
