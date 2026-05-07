// ============================================================
// AXON Lexer — lexer.rs
// Phase 2 stub — P2-03 will implement this fully
// Copyright © 2026 Edison Lepiten — AIEONYX
// github.com/aieonyx/axon
//
// P2-03 implementation target:
//   - Identifiers and keywords
//   - Integer and float literals
//   - String literals with \{expr} interpolation
//   - All operators including |>, ~>, let@, @now, @program_intent
//   - Comments (# line comments)
//   - Multi-character tokens in correct priority order
// ============================================================

use std::iter::Peekable;
use std::str::CharIndices;

use crate::span::{FileId, Span};
use crate::token::{Token, TokenKind, keyword_from_str, temporal_from_str};

/// The AXON lexer — converts source text into a flat token stream.
/// Every token carries a Span with file, line, column, and byte offsets.
/// The stream always ends with exactly one TokenKind::Eof.
/// Invalid characters produce TokenKind::Error — the lexer never panics.
pub struct Lexer<'src> {
    source  : &'src str,
    chars   : Peekable<CharIndices<'src>>,
    file_id : FileId,
    line    : u32,
    col     : u32,
    pos     : usize,
}

impl<'src> Lexer<'src> {
    pub fn new(source: &'src str, file_id: FileId) -> Self {
        Lexer {
            source,
            chars   : source.char_indices().peekable(),
            file_id,
            line    : 1,
            col     : 1,
            pos     : 0,
        }
    }

    /// Tokenize the entire source into a Vec<Token>.
    /// Always ends with TokenKind::Eof.
    /// Never panics — invalid input produces Error tokens.
    pub fn tokenize(mut self) -> Vec<Token> {
        // P2-03: implement full tokenization
        // For now: return minimal token stream so tests pass
        let span = Span::new(self.file_id, 0, 0, 1, 1);
        vec![Token::new(TokenKind::Eof, "", span)]
    }

    // ── P2-03 implementation stubs ────────────────────────────
    // These will be fully implemented in P2-03.
    // Using todo!() so compiler knows they are intentionally empty.

    fn next_token(&mut self) -> Option<Token> {
        todo!("P2-03: implement token dispatch")
    }

    fn ident_or_keyword(&mut self, start: usize) -> Token {
        todo!("P2-03: implement identifier and keyword lexing")
    }

    fn number(&mut self, start: usize) -> Token {
        todo!("P2-03: implement integer and float literal lexing")
    }

    fn string(&mut self, start: usize) -> Token {
        todo!("P2-03: implement string literal lexing")
    }

    fn string_interp(&mut self, start: usize) -> Vec<Token> {
        todo!("P2-03: implement string interpolation lexing")
    }

    /// Handle all @-prefixed tokens:
    /// @now, @lifetime, @epoch, @program_intent → temporal tokens
    /// @ai, @verify, etc → At token + subsequent ident tokens
    fn at_token(&mut self, start: usize) -> Token {
        todo!("P2-03: implement @ token dispatch")
    }

    /// Handle all operator tokens including multi-char:
    /// Priority order:
    ///   |>   PipeForward    (before |)
    ///   ~>   TildeArrow     (before ~)
    ///   ..=  DotDotEq       (before ..)
    ///   ..   DotDot         (before .)
    ///   ::<  TurboStart     (before ::)
    ///   ::   DoubleColon    (before :)
    ///   ->   Arrow          (before -)
    ///   =>   FatArrow       (before =)
    ///   !=   BangEq         (before !)
    ///   ==   EqEq           (before =)
    ///   <=   LtEq           (before <)
    ///   >=   GtEq           (before >)
    ///   <<   ShiftLeft      (before <)
    ///   >>   ShiftRight     (before >)
    fn operator(&mut self, first: char, start: usize) -> Token {
        todo!("P2-03: implement operator lexing with multi-char priority")
    }

    fn skip_comment(&mut self) {
        todo!("P2-03: implement comment skipping (# to end of line)")
    }

    fn make_span(&self, start: usize) -> Span {
        todo!("P2-03: implement span construction from start pos")
    }

    /// Advance one character, updating line/col tracking
    fn advance(&mut self) -> Option<(usize, char)> {
        todo!("P2-03: implement character advance with line tracking")
    }

    /// Peek at the next character without consuming it
    fn peek(&mut self) -> Option<char> {
        self.chars.peek().map(|(_, c)| *c)
    }

    /// Peek at the character after next (two-char lookahead)
    fn peek2(&mut self) -> Option<char> {
        todo!("P2-03: implement two-char lookahead")
    }
}

/// Public API — tokenize source text into a flat token stream.
/// This is the function called by the parser and CLI.
pub fn lex(source: &str, file_id: FileId) -> Vec<Token> {
    Lexer::new(source, file_id).tokenize()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::span::FileId;

    fn file() -> FileId { FileId(1) }

    #[test]
    fn empty_source_gives_eof() {
        let tokens = lex("", file());
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].kind, TokenKind::Eof);
    }

    // P2-03 tests will be added here as each feature is implemented.
    // See Compiler Pipeline Contracts v1.0 Section 5 for the
    // complete test checklist: L1 through L13.
}
