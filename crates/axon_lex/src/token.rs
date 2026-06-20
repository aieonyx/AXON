// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
//
// Rust mirror of token.ax — updated at P55.5.
// DO NOT edit independently — token.ax is the source of truth.

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Literals
    IntLit(i64),
    FloatLit(f64),
    StringLit(String),
    BoolLit(bool),

    // Identifiers
    Ident(String),

    // Core keywords
    Fn, Let, Mut, If, Else, While, For, Return, Match,
    Struct, Enum, Impl, Pub, Use, Mod, Type, Const, Static, Nil,

    // Sovereign keywords
    Sovereign, Capability, Seal, Domain,

    // v0.3 keywords (P55.5)
    Actor, Handle, Intent, Opaque, Foreach, Yield, Uses,
    Tainted, Clean, Secret, Auditable, Expires, Resident, Money, SafeInt,

    // v0.3 operators
    PipeForward,    // |>
    TildeArrow,     // ~>
    CapBang,        // !
    CapQuestion,    // ?
    At,             // @

    // Decorator tokens
    DecoratorDeterministic,
    DecoratorConstantTime,
    DecoratorAiSpecialize,
    DecoratorAiIntent,
    DecoratorAiVerify,
    DecoratorEnsures,
    DecoratorRequiresConsent,
    DecoratorSealedMemory,
    DecoratorBalanced,
    DecoratorAtomicFinancial,
    DecoratorModelSigned,
    DecoratorInferenceBudget,
    DecoratorRequiresHuman,

    // Temporal tokens
    AtNow, AtLifetime, AtEpoch,

    // Arithmetic operators
    Plus, Minus, Star, Slash, Percent,

    // Comparison operators
    Eq, EqEq, Bang, BangEq, Lt, LtEq, Gt, GtEq,

    // Logical operators
    And, AndAnd, Pipe, PipePipe,

    // Punctuation
    Arrow, FatArrow, Dot, DotDot,
    Colon, ColonColon, Semi, Comma,

    // Delimiters
    LParen, RParen, LBrace, RBrace, LBracket, RBracket,

    // Special
    Comment(String),
    Whitespace,
    Newline,
    Eof,
    Unknown(char),
}

pub fn token_is_trivia(tok: &Token) -> bool {
    matches!(tok, Token::Whitespace | Token::Newline | Token::Comment(_))
}

pub fn token_is_sovereign(tok: &Token) -> bool {
    matches!(tok, Token::Sovereign | Token::Capability | Token::Seal | Token::Domain)
}

pub fn token_is_sovereign_type(tok: &Token) -> bool {
    matches!(tok,
        Token::Tainted | Token::Clean | Token::Secret |
        Token::Auditable | Token::Expires | Token::Resident |
        Token::Money | Token::SafeInt
    )
}

pub fn token_is_decorator(tok: &Token) -> bool {
    matches!(tok,
        Token::DecoratorDeterministic  |
        Token::DecoratorConstantTime   |
        Token::DecoratorAiSpecialize   |
        Token::DecoratorAiIntent       |
        Token::DecoratorAiVerify       |
        Token::DecoratorEnsures        |
        Token::DecoratorRequiresConsent|
        Token::DecoratorSealedMemory   |
        Token::DecoratorBalanced       |
        Token::DecoratorAtomicFinancial|
        Token::DecoratorModelSigned    |
        Token::DecoratorInferenceBudget|
        Token::DecoratorRequiresHuman
    )
}
