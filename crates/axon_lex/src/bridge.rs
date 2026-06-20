// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
//
// Rust lexer bridge — mirrors lex.ax and cursor.ax logic exactly.
// Excised at P55 bootstrap when axonc compiles lex.ax natively.
// Any divergence between this file and the .ax sources is a bug.

use axon_std_string::{AxChar, AxString};
use crate::error::{LexError, LexResult};
use crate::token::Token;

// ── Cursor — mirrors cursor.ax ─────────────────────────────────────────────────

pub struct Cursor {
    source: Vec<char>,
    pos: usize,
}

impl Cursor {
    pub fn new(source: &str) -> Self {
        Cursor { source: source.chars().collect(), pos: 0 }
    }

    /// Return current char without consuming. Returns '\0' at EOF.
    pub fn peek(&self) -> char {
        self.source.get(self.pos).copied().unwrap_or('\0')
    }

    /// Return char after current without consuming.
    pub fn peek2(&self) -> char {
        self.source.get(self.pos + 1).copied().unwrap_or('\0')
    }

    /// Consume and return current char.
    pub fn advance(&mut self) -> char {
        let ch = self.peek();
        if ch != '\0' { self.pos += 1; }
        ch
    }
}

// ── Public entry point ─────────────────────────────────────────────────────────

/// Tokenize an entire source string into a Vec<Token>.
/// The final token is always Token::Eof.
pub fn lex_all(source: &str) -> LexResult<Vec<Token>> {
    let mut cursor = Cursor::new(source);
    let mut tokens = Vec::new();
    loop {
        let tok = lex_next(&mut cursor)?;
        let done = matches!(tok, Token::Eof);
        tokens.push(tok);
        if done { break; }
    }
    Ok(tokens)
}

/// Tokenize the next single token from the cursor.
pub fn lex_next(cursor: &mut Cursor) -> LexResult<Token> {
    let ch = cursor.advance();
    match ch {
        '\0'                       => Ok(Token::Eof),
        '\n'                       => Ok(Token::Newline),
        ' ' | '\t' | '\r'    => Ok(Token::Whitespace),
        '/'                         => lex_slash(cursor),
        '"' => lex_string(cursor),
        '0'..='9'                 => lex_number(cursor, ch),
        'a'..='z' | 'A'..='Z' | '_' => lex_ident(cursor, ch),
        '+'  => Ok(Token::Plus),
        '*'  => Ok(Token::Star),
        '%'  => Ok(Token::Percent),
        ';'  => Ok(Token::Semi),
        ','  => Ok(Token::Comma),
        '('  => Ok(Token::LParen),
        ')'  => Ok(Token::RParen),
        '{'  => Ok(Token::LBrace),
        '}'  => Ok(Token::RBrace),
        '['  => Ok(Token::LBracket),
        ']'  => Ok(Token::RBracket),
        '-'  => {
            if cursor.peek() == '>' { cursor.advance(); Ok(Token::Arrow) }
            else { Ok(Token::Minus) }
        }
        '='  => {
            if cursor.peek() == '=' { cursor.advance(); Ok(Token::EqEq) }
            else if cursor.peek() == '>' { cursor.advance(); Ok(Token::FatArrow) }
            else { Ok(Token::Eq) }
        }
        '!'  => {
            if cursor.peek() == '=' { cursor.advance(); Ok(Token::BangEq) }
            else { Ok(Token::Bang) }
        }
        '<'  => {
            if cursor.peek() == '=' { cursor.advance(); Ok(Token::LtEq) }
            else { Ok(Token::Lt) }
        }
        '>'  => {
            if cursor.peek() == '=' { cursor.advance(); Ok(Token::GtEq) }
            else { Ok(Token::Gt) }
        }
        '&'  => {
            if cursor.peek() == '&' { cursor.advance(); Ok(Token::AndAnd) }
            else { Ok(Token::And) }
        }
        '|'  => {
            if cursor.peek() == '|' { cursor.advance(); Ok(Token::PipePipe) }
            else { Ok(Token::Pipe) }
        }
        '.'  => {
            if cursor.peek() == '.' { cursor.advance(); Ok(Token::DotDot) }
            else { Ok(Token::Dot) }
        }
        ':'  => {
            if cursor.peek() == ':' { cursor.advance(); Ok(Token::ColonColon) }
            else { Ok(Token::Colon) }
        }
        other => Ok(Token::Unknown(other)),
    }
}

// ── Helpers ────────────────────────────────────────────────────────────────────

fn lex_slash(cursor: &mut Cursor) -> LexResult<Token> {
    if cursor.peek() == '/' {
        cursor.advance();
        let mut comment = String::new();
        while cursor.peek() != '\n' && cursor.peek() != '\0' {
            comment.push(cursor.advance());
        }
        Ok(Token::Comment(comment.trim().to_string()))
    } else if cursor.peek() == '*' {
        cursor.advance();
        let mut comment = String::new();
        loop {
            let c = cursor.advance();
            if c == '\0' { return Err(LexError::UnterminatedComment); }
            if c == '*' && cursor.peek() == '/' {
                cursor.advance();
                return Ok(Token::Comment(comment.trim().to_string()));
            }
            comment.push(c);
        }
    } else {
        Ok(Token::Slash)
    }
}

fn lex_string(cursor: &mut Cursor) -> LexResult<Token> {
    let mut s = String::new();
    loop {
        let c = cursor.advance();
        if c == '\0' { return Err(LexError::UnterminatedString); }
        if c == '"' { return Ok(Token::StringLit(s.clone())); }
        if c == '\\' {
            let e = cursor.advance();
            match e {
                'n'  => s.push('\n'),
                't'  => s.push('\t'),
                'r'  => s.push('\r'),
                '"' => s.push('"'),
                '\\' => s.push('\\'),
                '0'  => s.push('\0'),
                _    => s.push(e),
            }
        } else {
            s.push(c);
        }
    }
}

fn lex_number(cursor: &mut Cursor, first: char) -> LexResult<Token> {
    let mut num = String::new();
    num.push(first);
    while cursor.peek().is_ascii_digit() {
        num.push(cursor.advance());
    }
    if cursor.peek() == '.' && cursor.peek2().is_ascii_digit() {
        num.push(cursor.advance());
        while cursor.peek().is_ascii_digit() {
            num.push(cursor.advance());
        }
        let f: f64 = num.parse().map_err(|_| {
            LexError::InvalidLiteral(AxString::ax_from_str(&num))
        })?;
        Ok(Token::FloatLit(f))
    } else {
        let i: i64 = num.parse().map_err(|_| {
            LexError::InvalidLiteral(AxString::ax_from_str(&num))
        })?;
        Ok(Token::IntLit(i))
    }
}

fn lex_ident(cursor: &mut Cursor, first: char) -> LexResult<Token> {
    let mut ident = String::new();
    ident.push(first);
    while cursor.peek().is_alphanumeric() || cursor.peek() == '_' {
        ident.push(cursor.advance());
    }
    Ok(match ident.as_str() {
        "fn"         => Token::Fn,
        "let"        => Token::Let,
        "mut"        => Token::Mut,
        "if"         => Token::If,
        "else"       => Token::Else,
        "while"      => Token::While,
        "for"        => Token::For,
        "return"     => Token::Return,
        "match"      => Token::Match,
        "struct"     => Token::Struct,
        "enum"       => Token::Enum,
        "impl"       => Token::Impl,
        "pub"        => Token::Pub,
        "use"        => Token::Use,
        "mod"        => Token::Mod,
        "type"       => Token::Type,
        "const"      => Token::Const,
        "static"     => Token::Static,
        "nil"        => Token::Nil,
        "true"       => Token::BoolLit(true),
        "false"      => Token::BoolLit(false),
        "sovereign"  => Token::Sovereign,
        "capability" => Token::Capability,
        "seal"       => Token::Seal,
        "domain"     => Token::Domain,
        _            => Token::Ident(ident.clone()),
    })
}
