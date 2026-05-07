// ============================================================
// AXON Parser — axon_parser/src/lib.rs
// Copyright © 2026 Edison Lepiten — AIEONYX
// github.com/aieonyx/axon
// ============================================================

pub mod ast;
pub mod error;
pub mod parser;

pub use axon_lexer::{FileId, Span};
pub use error::ParseError;

/// Parse an AXON source string into a Program AST.
pub fn parse(source: &str, file_id: FileId) -> ParseResult {
    // Step 1: tokenize
    let raw_tokens = axon_lexer::lex(source, file_id);
    // Step 2: inject INDENT/DEDENT tokens
    let tokens = axon_lexer::inject_indentation(raw_tokens);
    // Step 3: parse
    let mut p       = parser::Parser::new(tokens, source, file_id);
    let program     = p.parse_program();
    ParseResult {
        program,
        errors: p.into_errors(),
    }
}

pub struct ParseResult {
    pub program : ast::Program,
    pub errors  : Vec<ParseError>,
}

impl ParseResult {
    pub fn is_ok(&self) -> bool { self.errors.is_empty() }
    pub fn has_errors(&self) -> bool { !self.errors.is_empty() }
}
