// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
//
// AXONYX token definitions — source of truth for the P49 lexer.
// The TextMate grammar for GitHub Linguist AXONYX submission
// is derived directly from these token definitions.

use axon_std_string::{AxChar, AxString};

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Literals
    IntLit(i64),
    FloatLit(f64),
    StringLit(AxString),
    BoolLit(bool),

    // Identifiers
    Ident(AxString),

    // Core keywords
    Fn, Let, Mut, If, Else, While, For, Return, Match,
    Struct, Enum, Impl, Pub, Use, Mod, Type, Const, Static, Nil,

    // Sovereign keywords — AIEONYX extension to the language
    Sovereign, Capability, Seal, Domain,

    // Arithmetic
    Plus, Minus, Star, Slash, Percent,

    // Comparison
    Eq, EqEq, Bang, BangEq, Lt, LtEq, Gt, GtEq,

    // Logical
    And, AndAnd, Pipe, PipePipe,

    // Punctuation
    Arrow, FatArrow, Dot, DotDot,
    Colon, ColonColon, Semi, Comma,

    // Delimiters
    LParen, RParen, LBrace, RBrace, LBracket, RBracket,

    // Special
    Comment(AxString),
    Whitespace,
    Newline,
    Eof,
    Unknown(AxChar),
}

impl Token {
    /// Trivia tokens are present in the stream but skipped by the parser.
    pub fn is_trivia(&self) -> bool {
        matches!(self, Token::Whitespace | Token::Newline | Token::Comment(_))
    }

    /// Returns true if this is a sovereign AIEONYX keyword.
    pub fn is_sovereign_keyword(&self) -> bool {
        matches!(self, Token::Sovereign | Token::Capability | Token::Seal | Token::Domain)
    }
}
