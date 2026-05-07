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
    let mut tokens = Vec::new();

    while let Some((pos, ch)) = self.chars.next() {
        self.pos = pos;
        match ch {
            // Skip spaces and tabs
            ' ' | '\t' | '\r' => {
                self.col += 1;
            }

            // Newline
            '\n' => {
                let span = Span::new(self.file_id, pos, pos + 1, self.line, self.col);
                tokens.push(Token::new(TokenKind::Newline, "\n", span));
                self.line += 1;
                self.col = 1;
            }

            // Identifiers and keywords
            'a'..='z' | 'A'..='Z' | '_' => {
                let start     = pos;
                let start_col = self.col;
                let mut word  = String::from(ch);
                self.col += 1;

                while let Some(&(_, nc)) = self.chars.peek() {
                    if nc.is_alphanumeric() || nc == '_' {
                        let (_, nc) = self.chars.next().unwrap();
                        word.push(nc);
                        self.col += 1;
                    } else if nc == '@' && word == "let" {
                            // let@ — consume the @ and emit LetAt
                            self.chars.next();
                            self.col += 1;
                            word.push('@');
                            break;
                    } else {
                        break;
            }
    }

    let span = Span::new(self.file_id, start, start + word.len(), self.line, start_col);

    // check if it is a keyword — let@ maps to LetAt
    let kind = if word == "let@" {
        TokenKind::LetAt
    } else {
        match keyword_from_str(&word) {
            Some(kw) => kw,
            None     => TokenKind::Ident(word.clone()),
        }
    };
    tokens.push(Token::new(kind, word, span));
}

            // Everything else — Error token for now
            // More cases will be added in P2-03 step by step
            // Integer literals
'0'..='9' => {
    let start     = pos;
    let start_col = self.col;
    let mut num   = String::from(ch);
    self.col += 1;

    while let Some(&(_, nc)) = self.chars.peek() {
        if nc.is_ascii_digit() {
            let (_, nc) = self.chars.next().unwrap();
            num.push(nc);
            self.col += 1;
        } else {
            break;
        }
    }

    let span = Span::new(
        self.file_id, start,
        start + num.len(),
        self.line, start_col
    );
    let value: i64 = num.parse().unwrap_or(0);
    tokens.push(Token::new(TokenKind::IntLit(value), num, span));
}
    // String literals
'"' => {
    let start     = pos;
    let start_col = self.col;
    self.col += 1;
    let mut content = String::new();
    let mut closed  = false;

    while let Some((_, sc)) = self.chars.next() {
        self.col += 1;
        match sc {
            '"' => { closed = true; break; }

            '\\' => {
                // peek at next char
                if let Some(&(_, nc)) = self.chars.peek() {
                    self.chars.next();
                    self.col += 1;
                    match nc {
                        'n'  => content.push('\n'),
                        't'  => content.push('\t'),
                        'r'  => content.push('\r'),
                        '\\' => content.push('\\'),
                        '"'  => content.push('"'),
                        '{'  => {
                            // String interpolation \{expr}
                            // Emit what we have so far as StrInterpStart
                            let span = Span::new(
                                self.file_id, start,
                                start + content.len() + 1,
                                self.line, start_col
                            );
                            tokens.push(Token::new(
                                TokenKind::StrInterpStart,
                                "\"",
                                span
                            ));
                            // Emit the literal part before \{
                            if !content.is_empty() {
                                let ps = Span::new(
                                    self.file_id, start + 1,
                                    start + 1 + content.len(),
                                    self.line, start_col + 1
                                );
                                tokens.push(Token::new(
                                    TokenKind::StrInterpPart(content.clone()),
                                    &content.clone(),
                                    ps
                                ));
                            }
                            // Emit StrInterpExprStart
                            let es = Span::new(
                                self.file_id, self.pos,
                                self.pos + 2,
                                self.line, self.col
                            );
                            tokens.push(Token::new(
                                TokenKind::StrInterpExprStart,
                                "\\{",
                                es
                            ));
                            // Lex tokens until we hit }
                            content.clear();
                            let mut depth = 1;
                            while let Some((ipos, ic)) = self.chars.next() {
                                self.pos = ipos;
                                self.col += 1;
                                if ic == '{' {
                                    depth += 1;
                                    let s = Span::new(self.file_id, ipos, ipos+1, self.line, self.col);
                                    tokens.push(Token::new(TokenKind::LBrace, "{", s));
                                } else if ic == '}' {
                                    depth -= 1;
                                    if depth == 0 {
                                        // Emit StrInterpExprEnd
                                        let s = Span::new(self.file_id, ipos, ipos+1, self.line, self.col);
                                        tokens.push(Token::new(TokenKind::StrInterpExprEnd, "}", s));
                                        break;
                                    } else {
                                        let s = Span::new(self.file_id, ipos, ipos+1, self.line, self.col);
                                        tokens.push(Token::new(TokenKind::RBrace, "}", s));
                                    }
                                } else if ic.is_alphabetic() || ic == '_' {
                                    // simple ident inside interpolation
                                    let istart = ipos;
                                    let icol   = self.col;
                                    let mut word = String::from(ic);
                                    while let Some(&(_, wc)) = self.chars.peek() {
                                        if wc.is_alphanumeric() || wc == '_' {
                                            let (_, wc) = self.chars.next().unwrap();
                                            word.push(wc);
                                            self.col += 1;
                                        } else { break; }
                                    }
                                    let s = Span::new(self.file_id, istart, istart+word.len(), self.line, icol);
                                    let kind = match keyword_from_str(&word) {
                                        Some(k) => k,
                                        None    => TokenKind::Ident(word.clone()),
                                    };
                                    tokens.push(Token::new(kind, word, s));
                                } else {
                                    let s = Span::new(self.file_id, ipos, ipos+1, self.line, self.col);
                                    tokens.push(Token::new(
                                        TokenKind::Error(format!("in interp: {:?}", ic)),
                                        &ic.to_string(), s
                                    ));
                                }
                            }
                            // continue scanning rest of string after }
                            continue;
                        }
                        other => {
                            content.push('\\');
                            content.push(other);
                        }
                    }
                }
            }

            '\n' => {
                content.push('\n');
                self.line += 1;
                self.col = 1;
            }
       
            other => { content.push(other); }
        }
    }

    // Check if this was an interpolated string
    // If tokens already has StrInterpStart — emit final part + End
    let has_interp = tokens.iter().rev().any(|t|
        t.kind == TokenKind::StrInterpStart ||
        t.kind == TokenKind::StrInterpExprEnd
    );

    if has_interp {
        // emit remaining content as final part
        if !content.is_empty() {
            let ps = Span::new(
                self.file_id, pos, pos + content.len(),
                self.line, start_col
            );
            tokens.push(Token::new(
                TokenKind::StrInterpPart(content.clone()),
                &content.clone(),
                ps
            ));
        }
        // emit StrInterpEnd
        let span = Span::new(self.file_id, pos, pos+1, self.line, self.col);
        tokens.push(Token::new(TokenKind::StrInterpEnd, "\"", span));
    } else {
        // plain string — emit as StrLit
        let span = Span::new(
            self.file_id, start,
            start + content.len() + 2,
            self.line, start_col
        );
        tokens.push(Token::new(
            TokenKind::StrLit(content),
            "",
            span
        ));
    }

    if !closed {
        let span = Span::new(self.file_id, pos, pos+1, self.line, start_col);
        tokens.push(Token::new(
            TokenKind::Error("unterminated string".into()),
            "",
            span
        ));
    }
}
    // Operators and punctuation
'|' => {
    let start_col = self.col;
    self.col += 1;
    if let Some(&(_, '>')) = self.chars.peek() {
        self.chars.next();
        self.col += 1;
        let span = Span::new(self.file_id, pos, pos+2, self.line, start_col);
        tokens.push(Token::new(TokenKind::PipeForward, "|>", span));
    } else {
        let span = Span::new(self.file_id, pos, pos+1, self.line, start_col);
        tokens.push(Token::new(TokenKind::Pipe, "|", span));
    }
}
'~' => {
    let start_col = self.col;
    self.col += 1;
    if let Some(&(_, '>')) = self.chars.peek() {
        self.chars.next();
        self.col += 1;
        let span = Span::new(self.file_id, pos, pos+2, self.line, start_col);
        tokens.push(Token::new(TokenKind::TildeArrow, "~>", span));
    } else {
        let span = Span::new(self.file_id, pos, pos+1, self.line, start_col);
        tokens.push(Token::new(TokenKind::Tilde, "~", span));
    }
}
'-' => {
    let start_col = self.col;
    self.col += 1;
    if let Some(&(_, '>')) = self.chars.peek() {
        self.chars.next();
        self.col += 1;
        let span = Span::new(self.file_id, pos, pos+2, self.line, start_col);
        tokens.push(Token::new(TokenKind::Arrow, "->", span));
    } else if let Some(&(_, '=')) = self.chars.peek() {
        self.chars.next();
        self.col += 1;
        let span = Span::new(self.file_id, pos, pos+2, self.line, start_col);
        tokens.push(Token::new(TokenKind::MinusAssign, "-=", span));
    } else {
        let span = Span::new(self.file_id, pos, pos+1, self.line, start_col);
        tokens.push(Token::new(TokenKind::Minus, "-", span));
    }
}
'=' => {
    let start_col = self.col;
    self.col += 1;
    if let Some(&(_, '>')) = self.chars.peek() {
        self.chars.next();
        self.col += 1;
        let span = Span::new(self.file_id, pos, pos+2, self.line, start_col);
        tokens.push(Token::new(TokenKind::FatArrow, "=>", span));
    } else if let Some(&(_, '=')) = self.chars.peek() {
        self.chars.next();
        self.col += 1;
        let span = Span::new(self.file_id, pos, pos+2, self.line, start_col);
        tokens.push(Token::new(TokenKind::EqEq, "==", span));
    } else {
        let span = Span::new(self.file_id, pos, pos+1, self.line, start_col);
        tokens.push(Token::new(TokenKind::Assign, "=", span));
    }
}
'!' => {
    let start_col = self.col;
    self.col += 1;
    if let Some(&(_, '=')) = self.chars.peek() {
        self.chars.next();
        self.col += 1;
        let span = Span::new(self.file_id, pos, pos+2, self.line, start_col);
        tokens.push(Token::new(TokenKind::BangEq, "!=", span));
    } else {
        let span = Span::new(self.file_id, pos, pos+1, self.line, start_col);
        tokens.push(Token::new(TokenKind::Bang, "!", span));
    }
}
'<' => {
    let start_col = self.col;
    self.col += 1;
    if let Some(&(_, '=')) = self.chars.peek() {
        self.chars.next();
        self.col += 1;
        let span = Span::new(self.file_id, pos, pos+2, self.line, start_col);
        tokens.push(Token::new(TokenKind::LtEq, "<=", span));
    } else if let Some(&(_, '<')) = self.chars.peek() {
        self.chars.next();
        self.col += 1;
        let span = Span::new(self.file_id, pos, pos+2, self.line, start_col);
        tokens.push(Token::new(TokenKind::ShiftLeft, "<<", span));
    } else {
        let span = Span::new(self.file_id, pos, pos+1, self.line, start_col);
        tokens.push(Token::new(TokenKind::Lt, "<", span));
    }
}
'>' => {
    let start_col = self.col;
    self.col += 1;
    if let Some(&(_, '=')) = self.chars.peek() {
        self.chars.next();
        self.col += 1;
        let span = Span::new(self.file_id, pos, pos+2, self.line, start_col);
        tokens.push(Token::new(TokenKind::GtEq, ">=", span));
    } else if let Some(&(_, '>')) = self.chars.peek() {
        self.chars.next();
        self.col += 1;
        let span = Span::new(self.file_id, pos, pos+2, self.line, start_col);
        tokens.push(Token::new(TokenKind::ShiftRight, ">>", span));
    } else {
        let span = Span::new(self.file_id, pos, pos+1, self.line, start_col);
        tokens.push(Token::new(TokenKind::Gt, ">", span));
    }
}
':' => {
    let start_col = self.col;
    self.col += 1;
    if let Some(&(_, ':')) = self.chars.peek() {
        self.chars.next();
        self.col += 1;
        if let Some(&(_, '<')) = self.chars.peek() {
            self.chars.next();
            self.col += 1;
            let span = Span::new(self.file_id, pos, pos+3, self.line, start_col);
            tokens.push(Token::new(TokenKind::TurboStart, "::<", span));
        } else {
            let span = Span::new(self.file_id, pos, pos+2, self.line, start_col);
            tokens.push(Token::new(TokenKind::DoubleColon, "::", span));
        }
    } else {
        let span = Span::new(self.file_id, pos, pos+1, self.line, start_col);
        tokens.push(Token::new(TokenKind::Colon, ":", span));
    }
}
'.' => {
    let start_col = self.col;
    self.col += 1;
    if let Some(&(_, '.')) = self.chars.peek() {
        self.chars.next();
        self.col += 1;
        if let Some(&(_, '=')) = self.chars.peek() {
            self.chars.next();
            self.col += 1;
            let span = Span::new(self.file_id, pos, pos+3, self.line, start_col);
            tokens.push(Token::new(TokenKind::DotDotEq, "..=", span));
        } else {
            let span = Span::new(self.file_id, pos, pos+2, self.line, start_col);
            tokens.push(Token::new(TokenKind::DotDot, "..", span));
        }
    } else {
        let span = Span::new(self.file_id, pos, pos+1, self.line, start_col);
        tokens.push(Token::new(TokenKind::Dot, ".", span));
    }
}
'+' => {
    let start_col = self.col; self.col += 1;
    if let Some(&(_, '=')) = self.chars.peek() {
        self.chars.next(); self.col += 1;
        let span = Span::new(self.file_id, pos, pos+2, self.line, start_col);
        tokens.push(Token::new(TokenKind::PlusAssign, "+=", span));
    } else {
        let span = Span::new(self.file_id, pos, pos+1, self.line, start_col);
        tokens.push(Token::new(TokenKind::Plus, "+", span));
    }
}
'*' => {
    let start_col = self.col; self.col += 1;
    if let Some(&(_, '=')) = self.chars.peek() {
        self.chars.next(); self.col += 1;
        let span = Span::new(self.file_id, pos, pos+2, self.line, start_col);
        tokens.push(Token::new(TokenKind::StarAssign, "*=", span));
    } else {
        let span = Span::new(self.file_id, pos, pos+1, self.line, start_col);
        tokens.push(Token::new(TokenKind::Star, "*", span));
    }
}
'/' => {
    let start_col = self.col; self.col += 1;
    if let Some(&(_, '=')) = self.chars.peek() {
        self.chars.next(); self.col += 1;
        let span = Span::new(self.file_id, pos, pos+2, self.line, start_col);
        tokens.push(Token::new(TokenKind::SlashAssign, "/=", span));
    } else {
        let span = Span::new(self.file_id, pos, pos+1, self.line, start_col);
        tokens.push(Token::new(TokenKind::Slash, "/", span));
    }
}
'%' => {
    let start_col = self.col; self.col += 1;
    if let Some(&(_, '=')) = self.chars.peek() {
        self.chars.next(); self.col += 1;
        let span = Span::new(self.file_id, pos, pos+2, self.line, start_col);
        tokens.push(Token::new(TokenKind::PercentAssign, "%=", span));
    } else {
        let span = Span::new(self.file_id, pos, pos+1, self.line, start_col);
        tokens.push(Token::new(TokenKind::Percent, "%", span));
    }
}
'&' => {
    let start_col = self.col; self.col += 1;
    if let Some(&(_, '=')) = self.chars.peek() {
        self.chars.next(); self.col += 1;
        let span = Span::new(self.file_id, pos, pos+2, self.line, start_col);
        tokens.push(Token::new(TokenKind::AmpAssign, "&=", span));
    } else {
        let span = Span::new(self.file_id, pos, pos+1, self.line, start_col);
        tokens.push(Token::new(TokenKind::Ampersand, "&", span));
    }
}
'^' => {
    let start_col = self.col; self.col += 1;
    if let Some(&(_, '=')) = self.chars.peek() {
        self.chars.next(); self.col += 1;
        let span = Span::new(self.file_id, pos, pos+2, self.line, start_col);
        tokens.push(Token::new(TokenKind::CaretAssign, "^=", span));
    } else {
        let span = Span::new(self.file_id, pos, pos+1, self.line, start_col);
        tokens.push(Token::new(TokenKind::Caret, "^", span));
    }
}
'(' => { let s = Span::new(self.file_id,pos,pos+1,self.line,self.col); self.col+=1; tokens.push(Token::new(TokenKind::LParen,"(",s)); }
')' => { let s = Span::new(self.file_id,pos,pos+1,self.line,self.col); self.col+=1; tokens.push(Token::new(TokenKind::RParen,")",s)); }
'[' => { let s = Span::new(self.file_id,pos,pos+1,self.line,self.col); self.col+=1; tokens.push(Token::new(TokenKind::LBracket,"[",s)); }
']' => { let s = Span::new(self.file_id,pos,pos+1,self.line,self.col); self.col+=1; tokens.push(Token::new(TokenKind::RBracket,"]",s)); }
'{' => { let s = Span::new(self.file_id,pos,pos+1,self.line,self.col); self.col+=1; tokens.push(Token::new(TokenKind::LBrace,"{",s)); }
'}' => { let s = Span::new(self.file_id,pos,pos+1,self.line,self.col); self.col+=1; tokens.push(Token::new(TokenKind::RBrace,"}",s)); }
',' => { let s = Span::new(self.file_id,pos,pos+1,self.line,self.col); self.col+=1; tokens.push(Token::new(TokenKind::Comma,",",s)); }
';' => { let s = Span::new(self.file_id,pos,pos+1,self.line,self.col); self.col+=1; tokens.push(Token::new(TokenKind::Semicolon,";",s)); }
'?' => { let s = Span::new(self.file_id,pos,pos+1,self.line,self.col); self.col+=1; tokens.push(Token::new(TokenKind::Question,"?",s)); }

    '@' => {
    let start_col = self.col;
    self.col += 1;
    // collect the identifier after @
    let mut name = String::new();
    while let Some(&(_, nc)) = self.chars.peek() {
        if nc.is_alphanumeric() || nc == '_' {
            let (_, nc) = self.chars.next().unwrap();
            name.push(nc);
            self.col += 1;
        } else {
            break;
        }
    }
    let full  = format!("@{}", name);
    let span  = Span::new(
        self.file_id, pos,
        pos + full.len(),
        self.line, start_col
    );
    // check for temporal / program_intent tokens
    match temporal_from_str(&name) {
        Some(kind) => tokens.push(Token::new(kind, full, span)),
        None => {
            // regular decorator — emit At + Ident separately
            let at_span = Span::new(
                self.file_id, pos, pos+1,
                self.line, start_col
            );
            tokens.push(Token::new(TokenKind::At, "@", at_span));
            if !name.is_empty() {
                let id_span = Span::new(
                    self.file_id, pos+1,
                    pos+1+name.len(),
                    self.line, start_col+1
                );
                tokens.push(Token::new(
                    TokenKind::Ident(name.clone()),
                    name,
                    id_span
                ));
            }
        }
    }
}
            '#' => {
    // Line comment — skip everything to end of line
    self.col += 1;
    while let Some(&(_, nc)) = self.chars.peek() {
        if nc == '\n' { break; }
        self.chars.next();
        self.col += 1;
    }
    // do not emit a token — comments are invisible to parser
}
            
            
            other => {
                let span = Span::new(self.file_id, pos, pos + 1, self.line, self.col);
                tokens.push(Token::new(
                    TokenKind::Error(format!("unexpected character: {:?}", other)),
                    &other.to_string(),
                    span,
                ));
                self.col += 1;
            }
        }
    }

    // Always end with Eof
    let eof_span = Span::new(self.file_id, self.pos, self.pos, self.line, self.col);
    tokens.push(Token::new(TokenKind::Eof, "", eof_span));
    tokens
}
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
