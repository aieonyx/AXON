// ============================================================
// AXON Codegen — error.rs
// Copyright © 2026 Edison Lepiten — AIEONYX
// ============================================================

#[derive(Debug, Clone)]
pub enum CodegenError {
    /// AXON source had parse errors — fix these first
    ParseErrors(Vec<String>),
    /// Transpiler encountered an unsupported AST node
    Unsupported(String),
    /// Internal transpiler error — should not happen
    Internal(String),
}

impl std::fmt::Display for CodegenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CodegenError::ParseErrors(errs) => {
                writeln!(f, "Parse errors ({}):", errs.len())?;
                for e in errs { writeln!(f, "  {}", e)?; }
                Ok(())
            }
            CodegenError::Unsupported(msg) =>
                write!(f, "Unsupported: {}", msg),
            CodegenError::Internal(msg) =>
                write!(f, "Internal codegen error: {}", msg),
        }
    }
}
