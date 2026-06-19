// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
// P49 QA -- axon_lex test suite
// Pass bar: 14/14 before P50 begins.

use axon_lex::{lex_all, Token};
use axon_std_string::AxString;

// T1: empty input yields only Eof
#[test]
fn test_lex_empty() {
    let tokens = lex_all("").unwrap();
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0], Token::Eof);
}

// T2: whitespace tokens
#[test]
fn test_lex_whitespace() {
    let tokens = lex_all("  \t  ").unwrap();
    let non_eof: Vec<&Token> = tokens.iter().filter(|t| **t != Token::Eof).collect();
    assert!(!non_eof.is_empty());
    assert!(non_eof.iter().all(|t| **t == Token::Whitespace));
}

// T3: newline token
#[test]
fn test_lex_newline() {
    let tokens = lex_all("\n").unwrap();
    assert_eq!(tokens[0], Token::Newline);
    assert_eq!(tokens[1], Token::Eof);
}

// T4: integer literal
#[test]
fn test_lex_int_literal() {
    let tokens = lex_all("42").unwrap();
    assert_eq!(tokens[0], Token::IntLit(42));
    assert_eq!(tokens[1], Token::Eof);
}

// T5: float literal
#[test]
fn test_lex_float_literal() {
    let tokens = lex_all("3.14").unwrap();
    if let Token::FloatLit(f) = tokens[0] {
        assert!((f - 3.14_f64).abs() < 1e-10);
    } else {
        panic!("expected FloatLit, got {:?}", tokens[0]);
    }
    assert_eq!(tokens[1], Token::Eof);
}

// T6: string literal
#[test]
fn test_lex_string_literal() {
    let tokens = lex_all("\"hello\"").unwrap();
    assert_eq!(tokens[0], Token::StringLit(AxString::ax_from_str("hello")));
    assert_eq!(tokens[1], Token::Eof);
}

// T7: bool literals
#[test]
fn test_lex_bool_literal() {
    let tokens = lex_all("true false").unwrap();
    assert_eq!(tokens[0], Token::BoolLit(true));
    assert_eq!(tokens[1], Token::Whitespace);
    assert_eq!(tokens[2], Token::BoolLit(false));
    assert_eq!(tokens[3], Token::Eof);
}

// T8: core keywords
#[test]
fn test_lex_keywords() {
    let tokens = lex_all("fn let mut if").unwrap();
    let sig: Vec<&Token> = tokens.iter()
        .filter(|t| !t.is_trivia() && **t != Token::Eof).collect();
    assert_eq!(*sig[0], Token::Fn);
    assert_eq!(*sig[1], Token::Let);
    assert_eq!(*sig[2], Token::Mut);
    assert_eq!(*sig[3], Token::If);
}

// T9: sovereign keywords
#[test]
fn test_lex_sovereign_keywords() {
    let tokens = lex_all("sovereign capability seal domain").unwrap();
    let sig: Vec<&Token> = tokens.iter()
        .filter(|t| !t.is_trivia() && **t != Token::Eof).collect();
    assert_eq!(*sig[0], Token::Sovereign);
    assert_eq!(*sig[1], Token::Capability);
    assert_eq!(*sig[2], Token::Seal);
    assert_eq!(*sig[3], Token::Domain);
    assert!(sig.iter().all(|t| t.is_sovereign_keyword()));
}

// T10: operators
#[test]
fn test_lex_operators() {
    let tokens = lex_all("+ - * / == != < > ->").unwrap();
    let sig: Vec<&Token> = tokens.iter()
        .filter(|t| !t.is_trivia() && **t != Token::Eof).collect();
    assert_eq!(*sig[0], Token::Plus);
    assert_eq!(*sig[1], Token::Minus);
    assert_eq!(*sig[2], Token::Star);
    assert_eq!(*sig[3], Token::Slash);
    assert_eq!(*sig[4], Token::EqEq);
    assert_eq!(*sig[5], Token::BangEq);
    assert_eq!(*sig[6], Token::Lt);
    assert_eq!(*sig[7], Token::Gt);
    assert_eq!(*sig[8], Token::Arrow);
}

// T11: delimiters
#[test]
fn test_lex_delimiters() {
    let tokens = lex_all("( ) { } [ ]").unwrap();
    let sig: Vec<&Token> = tokens.iter()
        .filter(|t| !t.is_trivia() && **t != Token::Eof).collect();
    assert_eq!(*sig[0], Token::LParen);
    assert_eq!(*sig[1], Token::RParen);
    assert_eq!(*sig[2], Token::LBrace);
    assert_eq!(*sig[3], Token::RBrace);
    assert_eq!(*sig[4], Token::LBracket);
    assert_eq!(*sig[5], Token::RBracket);
}

// T12: identifier
#[test]
fn test_lex_identifier() {
    let tokens = lex_all("my_var").unwrap();
    assert_eq!(tokens[0], Token::Ident(AxString::ax_from_str("my_var")));
    assert_eq!(tokens[1], Token::Eof);
}

// T13: line comment
#[test]
fn test_lex_line_comment() {
    let tokens = lex_all("// sovereign lexer").unwrap();
    if let Token::Comment(text) = &tokens[0] {
        assert_eq!(text.as_str(), "sovereign lexer");
    } else {
        panic!("expected Comment, got {:?}", tokens[0]);
    }
    assert_eq!(tokens[1], Token::Eof);
}

// T14: full AXONYX program token stream
#[test]
fn test_lex_full_program() {
    let src = "fn add(a: i32, b: i32) -> i32 {\n    return a + b;\n}";
    let tokens = lex_all(src).unwrap();
    let sig: Vec<&Token> = tokens.iter()
        .filter(|t| !t.is_trivia())
        .collect();

    assert_eq!(*sig[0],  Token::Fn);
    assert_eq!(*sig[1],  Token::Ident(AxString::ax_from_str("add")));
    assert_eq!(*sig[2],  Token::LParen);
    assert_eq!(*sig[3],  Token::Ident(AxString::ax_from_str("a")));
    assert_eq!(*sig[4],  Token::Colon);
    assert_eq!(*sig[5],  Token::Ident(AxString::ax_from_str("i32")));
    assert_eq!(*sig[6],  Token::Comma);
    assert_eq!(*sig[7],  Token::Ident(AxString::ax_from_str("b")));
    assert_eq!(*sig[8],  Token::Colon);
    assert_eq!(*sig[9],  Token::Ident(AxString::ax_from_str("i32")));
    assert_eq!(*sig[10], Token::RParen);
    assert_eq!(*sig[11], Token::Arrow);
    assert_eq!(*sig[12], Token::Ident(AxString::ax_from_str("i32")));
    assert_eq!(*sig[13], Token::LBrace);
    assert_eq!(*sig[14], Token::Return);
    assert_eq!(*sig[15], Token::Ident(AxString::ax_from_str("a")));
    assert_eq!(*sig[16], Token::Plus);
    assert_eq!(*sig[17], Token::Ident(AxString::ax_from_str("b")));
    assert_eq!(*sig[18], Token::Semi);
    assert_eq!(*sig[19], Token::RBrace);
    assert_eq!(*sig[20], Token::Eof);
}
