// ============================================================
// AXON Lexer — lib.rs
// Copyright © 2026 Edison Lepiten — AIEONYX
// github.com/aieonyx/axon
// ============================================================

pub mod lexer;
pub mod token;
pub mod span;
pub mod indent;

pub use token::{Token, TokenKind, keyword_from_str, temporal_from_str};
pub use span::{Span, FileId};
pub use lexer::lex;
pub use indent::inject_indentation;
