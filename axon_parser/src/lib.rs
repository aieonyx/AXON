// ============================================================
// AXON Parser — axon_parser/src/lib.rs
// Copyright © 2026 Edison Lepiten — AIEONYX
// github.com/aieonyx/axon
//
// Phase 2 — P2-01: Workspace scaffold
// Status  : Module structure defined.
//           AST nodes begin P2-05.
//           Parser implementation begins P2-06.
// ============================================================

pub mod ast;
pub mod error;
pub mod parser;

pub use axon_lexer::{FileId, Span};
pub use error::ParseError;

// ── Public API ───────────────────────────────────────────────

/// Parse an AXON source string into a Program AST.
/// Always returns a ParseResult — errors are collected,
/// not propagated as panics.
pub fn parse(source: &str, file_id: FileId) -> ParseResult {
    let tokens = axon_lexer::lex(source, file_id);
    let mut p  = parser::Parser::new(tokens, source, file_id);
    let program = p.parse_program();
    ParseResult {
        program,
        errors: p.into_errors(),
    }
}

/// The result of parsing an AXON source file.
/// May contain a partial AST alongside collected errors.
pub struct ParseResult {
    pub program : ast::Program,
    pub errors  : Vec<ParseError>,
}

impl ParseResult {
    /// True if parsing completed with no errors.
    pub fn is_ok(&self) -> bool {
        self.errors.is_empty()
    }

    /// True if parsing produced any errors.
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }
}
