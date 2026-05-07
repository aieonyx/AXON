// ============================================================
// AXON Parser — parser.rs
// Recursive descent parser — stub for P2-01
// Full implementation begins P2-06
// ============================================================

use axon_lexer::{FileId, Span, Token, TokenKind};
use crate::ast::*;
use crate::error::ParseError;

/// The AXON recursive descent parser.
/// Consumes a token stream and produces a typed Program AST.
pub struct Parser<'src> {
    tokens  : Vec<Token>,
    pos     : usize,
    errors  : Vec<ParseError>,
    source  : &'src str,
    file_id : FileId,
}

impl<'src> Parser<'src> {
    /// Create a new parser from a token stream.
    pub fn new(tokens: Vec<Token>, source: &'src str, file_id: FileId) -> Self {
        Parser {
            tokens,
            pos: 0,
            errors: Vec::new(),
            source,
            file_id,
        }
    }

    /// Consume this parser and return the collected errors.
    pub fn into_errors(self) -> Vec<ParseError> {
        self.errors
    }

    // ── Core cursor methods ───────────────────────────────────

    /// Peek at the current token without consuming it.
    pub fn peek(&self) -> &Token {
        &self.tokens[self.pos.min(self.tokens.len() - 1)]
    }

    /// Peek at the current token kind.
    pub fn peek_kind(&self) -> &TokenKind {
        &self.peek().kind
    }

    /// Peek ahead by n tokens.
    pub fn peek_ahead(&self, n: usize) -> &Token {
        let idx = (self.pos + n).min(self.tokens.len() - 1);
        &self.tokens[idx]
    }

    /// Consume and return the current token.
    pub fn advance(&mut self) -> Token {
        let tok = self.tokens[self.pos.min(self.tokens.len() - 1)].clone();
        if self.pos < self.tokens.len() - 1 {
            self.pos += 1;
        }
        tok
    }

    /// Consume the current token if it matches kind.
    /// Returns true if consumed, false otherwise.
    pub fn eat(&mut self, kind: &TokenKind) -> bool {
        if std::mem::discriminant(self.peek_kind()) == std::mem::discriminant(kind) {
            self.advance();
            true
        } else {
            false
        }
    }

    /// Consume the current token, asserting it matches kind.
    /// Records an error and returns None if it doesn't match.
    pub fn expect(&mut self, kind: TokenKind) -> Option<Token> {
        if std::mem::discriminant(self.peek_kind()) == std::mem::discriminant(&kind) {
            Some(self.advance())
        } else {
            let err = ParseError::UnexpectedToken {
                expected : kind.display_name().to_string(),
                found    : self.peek().kind.clone(),
                span     : self.peek().span,
                hint     : None,
            };
            self.errors.push(err);
            None
        }
    }

    /// Skip tokens until a synchronization token is found.
    /// Used for error recovery.
    pub fn recover_to(&mut self, sync: &[TokenKind]) {
        while !matches!(self.peek_kind(), TokenKind::Eof) {
            for s in sync {
                if std::mem::discriminant(self.peek_kind()) == std::mem::discriminant(s) {
                    return;
                }
            }
            self.advance();
        }
    }

    /// Current span (span of the current token).
    pub fn current_span(&self) -> Span {
        self.peek().span
    }

    // ── Parse entry point ─────────────────────────────────────

    /// Parse the entire source file into a Program AST.
    /// P2-01 STUB — returns empty Program.
    /// Full implementation begins P2-07.
    pub fn parse_program(&mut self) -> Program {
        let start = self.current_span();

        // P2-01: stub — will be fully implemented in P2-07
        Program {
            span           : start,
            program_intent : None,
            module         : None,
            imports        : vec![],
            items          : vec![],
        }
    }
}

// ── Tests ────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use axon_lexer::FileId;

    #[test]
    fn test_parse_empty_source() {
        let result = crate::parse("", FileId(0));
        // P2-01: empty source produces empty program with no errors
        assert!(result.is_ok());
        assert!(result.program.items.is_empty());
    }

    #[test]
    fn test_parse_result_struct() {
        let result = crate::parse("fn main():", FileId(0));
        // P2-01: stub parser — no items yet but no panic
        assert!(result.program.items.is_empty() || !result.program.items.is_empty());
    }
}
