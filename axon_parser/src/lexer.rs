// axon_parser/src/lexer.rs
// AXON Lexer — Stage 8A-1
// Hand-written, no external crates.

#[derive(Debug, Clone, PartialEq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}
impl Span {
    pub fn new(start: usize, end: usize) -> Self { Span { start, end } }
}
#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}
impl Token {
    pub fn new(kind: TokenKind, start: usize, end: usize) -> Self {
        Token { kind, span: Span::new(start, end) }
    }
}
#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    Fn, Trait, Impl, Struct, Enum, Type, Let, Mut, Const,
    If, Else, While, Loop, For, In, Return, Break, Continue, Match, Use,
    Pub, Mod, SelfVal, SelfType, Super, Where, As, Move,
    Extern,
    Sovereign, Capability, Profile, Patchable,
    Requires, Ensures, Invariant, Ghost, Pure, UnsafeAxon,
    Asm,
    AtRequires, AtEnsures, AtInvariant,
    IntLit(u64), FloatLit(f64), StringLit(String), CharLit(char), BoolLit(bool),
    Ident(String), Lifetime(String),
    Plus, Minus, Star, Slash, Percent,
    Amp, Pipe, Caret, Bang, Tilde, LtLt, GtGt,
    AmpAmp, PipePipe, QuestQuest,
    Eq, PlusEq, MinusEq, StarEq, SlashEq, PercentEq,
    AmpEq, PipeEq, CaretEq, LtLtEq, GtGtEq,
    EqEq, BangEq, Lt, Gt, LtEq, GtEq,
    Arrow, FatArrow, ColonColon, Dot, DotDot, DotDotEq,
    Comma, Semi, Colon, Quest, At, Pound,
    LParen, RParen, LBrace, RBrace, LBracket, RBracket,
    Eof,
}
#[derive(Debug, Clone)]
pub struct LexError { pub msg: String, pub span: Span }
impl LexError {
    pub fn new(msg: impl Into<String>, start: usize, end: usize) -> Self {
        LexError { msg: msg.into(), span: Span::new(start, end) }
    }
}
impl std::fmt::Display for LexError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "LexError at {}..{}: {}", self.span.start, self.span.end, self.msg)
    }
}
pub struct Lexer<'src> { src: &'src str, bytes: &'src [u8], pos: usize }
impl<'src> Lexer<'src> {
    pub fn new(src: &'src str) -> Self { Lexer { src, bytes: src.as_bytes(), pos: 0 } }
    /// Maximum source size: 10 MB. Prevents OOM DoS on malicious input.
    pub const MAX_SOURCE_BYTES: usize = 10 * 1024 * 1024;

    pub fn tokenize(mut self) -> Result<Vec<Token>, LexError> {
        // S1: Input size guard
        if self.src.len() > Self::MAX_SOURCE_BYTES {
            return Err(LexError::new(
                format!("source too large: {} bytes (max {})", self.src.len(), Self::MAX_SOURCE_BYTES),
                0, self.src.len(),
            ));
        }
        let mut tokens = Vec::new();
        loop {
            self.skip_ws(&mut tokens)?;
            if self.pos >= self.bytes.len() {
                tokens.push(Token::new(TokenKind::Eof, self.pos, self.pos));
                break;
            }
            tokens.push(self.next_token()?);
        }
        Ok(tokens)
    }
    fn peek(&self) -> Option<u8> { self.bytes.get(self.pos).copied() }
    fn skip_ws(&mut self, _t: &mut Vec<Token>) -> Result<(), LexError> {
        loop {
            while self.pos < self.bytes.len() && self.bytes[self.pos].is_ascii_whitespace() {
                self.pos += 1;
            }
            if self.pos + 1 < self.bytes.len() && self.bytes[self.pos] == b'/' && self.bytes[self.pos+1] == b'/' {
                while self.pos < self.bytes.len() && self.bytes[self.pos] != b'\n' { self.pos += 1; }
                continue;
            }
            if self.pos + 1 < self.bytes.len() && self.bytes[self.pos] == b'/' && self.bytes[self.pos+1] == b'*' {
                let start = self.pos; self.pos += 2; let mut depth = 1usize;
                loop {
                    if self.pos >= self.bytes.len() {
                        return Err(LexError::new("unterminated block comment", start, self.pos));
                    }
                    if self.pos+1 < self.bytes.len() && self.bytes[self.pos]==b'/' && self.bytes[self.pos+1]==b'*' {
                        depth += 1; self.pos += 2;
                    } else if self.pos+1 < self.bytes.len() && self.bytes[self.pos]==b'*' && self.bytes[self.pos+1]==b'/' {
                        depth -= 1; self.pos += 2; if depth == 0 { break; }
                    } else { self.pos += 1; }
                }
                continue;
            }
            break;
        }
        Ok(())
    }
    fn match_kw_ahead(&mut self, kw: &str) -> bool {
        let kb = kw.as_bytes(); let end = self.pos + kb.len();
        if end > self.bytes.len() { return false; }
        if &self.bytes[self.pos..end] != kb { return false; }
        if end < self.bytes.len() && is_ident_cont(self.bytes[end]) { return false; }
        self.pos = end; true
    }
    fn next_token(&mut self) -> Result<Token, LexError> {
        let start = self.pos;
        let b = self.peek().unwrap();
        match b {
            b'@' => {
                self.pos += 1;
                if self.match_kw_ahead("requires") { return Ok(Token::new(TokenKind::AtRequires, start, self.pos)); }
                if self.match_kw_ahead("ensures")  { return Ok(Token::new(TokenKind::AtEnsures,  start, self.pos)); }
                if self.match_kw_ahead("invariant"){ return Ok(Token::new(TokenKind::AtInvariant, start, self.pos)); }
                Ok(Token::new(TokenKind::At, start, self.pos))
            }
            b'\'' => {
                self.pos += 1;
                if self.pos < self.bytes.len() && is_ident_start(self.bytes[self.pos]) {
                    let is = self.pos;
                    while self.pos < self.bytes.len() && is_ident_cont(self.bytes[self.pos]) { self.pos += 1; }
                    let name = &self.src[is..self.pos];
                    if self.pos >= self.bytes.len() || self.bytes[self.pos] != b'\'' {
                        return Ok(Token::new(TokenKind::Lifetime(name.to_string()), start, self.pos));
                    }
                    if name.len() == 1 {
                        self.pos += 1;
                        return Ok(Token::new(TokenKind::CharLit(name.chars().next().unwrap()), start, self.pos));
                    }
                    return Err(LexError::new(format!("invalid char literal '{}'", name), start, self.pos));
                }
                let ch = self.lex_char_body(start)?;
                if self.pos >= self.bytes.len() || self.bytes[self.pos] != b'\'' {
                    return Err(LexError::new("unterminated char literal", start, self.pos));
                }
                self.pos += 1;
                Ok(Token::new(TokenKind::CharLit(ch), start, self.pos))
            }
            b'"' => {
                self.pos += 1; let mut s = String::new();
                loop {
                    if self.pos >= self.bytes.len() { return Err(LexError::new("unterminated string", start, self.pos)); }
                    let c = self.bytes[self.pos];
                    if c == b'"' { self.pos += 1; break; }
                    if c == b'\\' { self.pos += 1; s.push(self.lex_escape(start)?); }
                    else { let ch = self.src[self.pos..].chars().next().unwrap(); self.pos += ch.len_utf8(); s.push(ch); }
                }
                Ok(Token::new(TokenKind::StringLit(s), start, self.pos))
            }
            b'0'..=b'9' => self.lex_number(start),
            b if is_ident_start(b) => {
                while self.pos < self.bytes.len() && is_ident_cont(self.bytes[self.pos]) { self.pos += 1; }
                Ok(Token::new(kw_or_ident(&self.src[start..self.pos]), start, self.pos))
            }
            b'+' => { self.pos+=1; if self.peek()==Some(b'=') { self.pos+=1; Ok(Token::new(TokenKind::PlusEq,start,self.pos)) } else { Ok(Token::new(TokenKind::Plus,start,self.pos)) } }
            b'-' => { self.pos+=1; if self.peek()==Some(b'>') { self.pos+=1; Ok(Token::new(TokenKind::Arrow,start,self.pos)) } else if self.peek()==Some(b'=') { self.pos+=1; Ok(Token::new(TokenKind::MinusEq,start,self.pos)) } else { Ok(Token::new(TokenKind::Minus,start,self.pos)) } }
            b'*' => { self.pos+=1; if self.peek()==Some(b'=') { self.pos+=1; Ok(Token::new(TokenKind::StarEq,start,self.pos)) } else { Ok(Token::new(TokenKind::Star,start,self.pos)) } }
            b'/' => { self.pos+=1; if self.peek()==Some(b'=') { self.pos+=1; Ok(Token::new(TokenKind::SlashEq,start,self.pos)) } else { Ok(Token::new(TokenKind::Slash,start,self.pos)) } }
            b'%' => { self.pos+=1; if self.peek()==Some(b'=') { self.pos+=1; Ok(Token::new(TokenKind::PercentEq,start,self.pos)) } else { Ok(Token::new(TokenKind::Percent,start,self.pos)) } }
            b'&' => { self.pos+=1; if self.peek()==Some(b'&') { self.pos+=1; Ok(Token::new(TokenKind::AmpAmp,start,self.pos)) } else if self.peek()==Some(b'=') { self.pos+=1; Ok(Token::new(TokenKind::AmpEq,start,self.pos)) } else { Ok(Token::new(TokenKind::Amp,start,self.pos)) } }
            b'|' => { self.pos+=1; if self.peek()==Some(b'|') { self.pos+=1; Ok(Token::new(TokenKind::PipePipe,start,self.pos)) } else if self.peek()==Some(b'=') { self.pos+=1; Ok(Token::new(TokenKind::PipeEq,start,self.pos)) } else { Ok(Token::new(TokenKind::Pipe,start,self.pos)) } }
            b'^' => { self.pos+=1; if self.peek()==Some(b'=') { self.pos+=1; Ok(Token::new(TokenKind::CaretEq,start,self.pos)) } else { Ok(Token::new(TokenKind::Caret,start,self.pos)) } }
            b'!' => { self.pos+=1; if self.peek()==Some(b'=') { self.pos+=1; Ok(Token::new(TokenKind::BangEq,start,self.pos)) } else { Ok(Token::new(TokenKind::Bang,start,self.pos)) } }
            b'~' => { self.pos+=1; Ok(Token::new(TokenKind::Tilde,start,self.pos)) }
            b'<' => { self.pos+=1; if self.peek()==Some(b'<') { self.pos+=1; if self.peek()==Some(b'=') { self.pos+=1; Ok(Token::new(TokenKind::LtLtEq,start,self.pos)) } else { Ok(Token::new(TokenKind::LtLt,start,self.pos)) } } else if self.peek()==Some(b'=') { self.pos+=1; Ok(Token::new(TokenKind::LtEq,start,self.pos)) } else { Ok(Token::new(TokenKind::Lt,start,self.pos)) } }
            b'>' => { self.pos+=1; if self.peek()==Some(b'>') { self.pos+=1; if self.peek()==Some(b'=') { self.pos+=1; Ok(Token::new(TokenKind::GtGtEq,start,self.pos)) } else { Ok(Token::new(TokenKind::GtGt,start,self.pos)) } } else if self.peek()==Some(b'=') { self.pos+=1; Ok(Token::new(TokenKind::GtEq,start,self.pos)) } else { Ok(Token::new(TokenKind::Gt,start,self.pos)) } }
            b'=' => { self.pos+=1; if self.peek()==Some(b'=') { self.pos+=1; Ok(Token::new(TokenKind::EqEq,start,self.pos)) } else if self.peek()==Some(b'>') { self.pos+=1; Ok(Token::new(TokenKind::FatArrow,start,self.pos)) } else { Ok(Token::new(TokenKind::Eq,start,self.pos)) } }
            b'.' => { self.pos+=1; if self.peek()==Some(b'.') { self.pos+=1; if self.peek()==Some(b'=') { self.pos+=1; Ok(Token::new(TokenKind::DotDotEq,start,self.pos)) } else { Ok(Token::new(TokenKind::DotDot,start,self.pos)) } } else { Ok(Token::new(TokenKind::Dot,start,self.pos)) } }
            b':' => { self.pos+=1; if self.peek()==Some(b':') { self.pos+=1; Ok(Token::new(TokenKind::ColonColon,start,self.pos)) } else { Ok(Token::new(TokenKind::Colon,start,self.pos)) } }
            b'?' => { self.pos+=1; if self.peek()==Some(b'?') { self.pos+=1; Ok(Token::new(TokenKind::QuestQuest,start,self.pos)) } else { Ok(Token::new(TokenKind::Quest,start,self.pos)) } }
            b'#' => { self.pos+=1; Ok(Token::new(TokenKind::Pound,start,self.pos)) }
            b',' => { self.pos+=1; Ok(Token::new(TokenKind::Comma,start,self.pos)) }
            b';' => { self.pos+=1; Ok(Token::new(TokenKind::Semi,start,self.pos)) }
            b'(' => { self.pos+=1; Ok(Token::new(TokenKind::LParen,start,self.pos)) }
            b')' => { self.pos+=1; Ok(Token::new(TokenKind::RParen,start,self.pos)) }
            b'{' => { self.pos+=1; Ok(Token::new(TokenKind::LBrace,start,self.pos)) }
            b'}' => { self.pos+=1; Ok(Token::new(TokenKind::RBrace,start,self.pos)) }
            b'[' => { self.pos+=1; Ok(Token::new(TokenKind::LBracket,start,self.pos)) }
            b']' => { self.pos+=1; Ok(Token::new(TokenKind::RBracket,start,self.pos)) }
            _ => { self.pos+=1; Err(LexError::new(format!("unexpected char: {:?}", b as char), start, self.pos)) }
        }
    }
    fn lex_number(&mut self, start: usize) -> Result<Token, LexError> {
        if self.bytes[self.pos] == b'0' && self.pos+1 < self.bytes.len() {
            match self.bytes[self.pos+1] {
                b'x'|b'X' => { self.pos+=2; return self.lex_radix(start,16,"hex"); }
                b'b'|b'B' => { self.pos+=2; return self.lex_radix(start,2,"binary"); }
                b'o'|b'O' => { self.pos+=2; return self.lex_radix(start,8,"octal"); }
                _ => {}
            }
        }
        while self.pos < self.bytes.len() && (self.bytes[self.pos].is_ascii_digit() || self.bytes[self.pos]==b'_') { self.pos+=1; }
        let has_dot = self.pos+1 < self.bytes.len() && self.bytes[self.pos]==b'.' && self.bytes[self.pos+1].is_ascii_digit();
        let has_exp = self.pos < self.bytes.len() && (self.bytes[self.pos]==b'e'||self.bytes[self.pos]==b'E');
        if has_dot || has_exp {
            if has_dot { self.pos+=1; while self.pos<self.bytes.len()&&(self.bytes[self.pos].is_ascii_digit()||self.bytes[self.pos]==b'_'){self.pos+=1;} }
            if self.pos<self.bytes.len()&&(self.bytes[self.pos]==b'e'||self.bytes[self.pos]==b'E') {
                self.pos+=1;
                if self.pos<self.bytes.len()&&(self.bytes[self.pos]==b'+'||self.bytes[self.pos]==b'-'){self.pos+=1;}
                while self.pos<self.bytes.len()&&(self.bytes[self.pos].is_ascii_digit()||self.bytes[self.pos]==b'_'){self.pos+=1;}
            }
            let raw = self.src[start..self.pos].replace('_',"");
            let val: f64 = raw.parse().map_err(|_| LexError::new(format!("invalid float: {}",raw),start,self.pos))?;
            return Ok(Token::new(TokenKind::FloatLit(val),start,self.pos));
        }
        let raw = self.src[start..self.pos].replace('_',"");
        let val: u64 = raw.parse().map_err(|_| LexError::new(format!("int overflow: {}",raw),start,self.pos))?;
        Ok(Token::new(TokenKind::IntLit(val),start,self.pos))
    }
    fn lex_radix(&mut self, start: usize, radix: u32, name: &str) -> Result<Token, LexError> {
        let ds = self.pos;
        while self.pos<self.bytes.len()&&(self.bytes[self.pos]==b'_'||self.bytes[self.pos].is_ascii_alphanumeric()){self.pos+=1;}
        if self.pos==ds { return Err(LexError::new(format!("empty {} literal",name),start,self.pos)); }
        let raw = self.src[ds..self.pos].replace('_',"");
        let val = u64::from_str_radix(&raw,radix).map_err(|_| LexError::new(format!("invalid {} literal: {}",name,raw),start,self.pos))?;
        Ok(Token::new(TokenKind::IntLit(val),start,self.pos))
    }
    fn lex_char_body(&mut self, start: usize) -> Result<char, LexError> {
        if self.pos >= self.bytes.len() { return Err(LexError::new("unterminated char",start,self.pos)); }
        if self.bytes[self.pos]==b'\\' { self.pos+=1; self.lex_escape(start) }
        else { let ch=self.src[self.pos..].chars().next().unwrap(); self.pos+=ch.len_utf8(); Ok(ch) }
    }
    fn lex_escape(&mut self, start: usize) -> Result<char, LexError> {
        if self.pos>=self.bytes.len() { return Err(LexError::new("unexpected end of escape",start,self.pos)); }
        let b=self.bytes[self.pos]; self.pos+=1;
        match b {
            b'n'=>Ok('\n'), b't'=>Ok('\t'), b'r'=>Ok('\r'), b'\\'=>Ok('\\'),
            b'"'=>Ok('"'), b'\''=>Ok('\''), b'0'=>Ok('\0'),
            b'x' => {
                if self.pos+2>self.bytes.len() { return Err(LexError::new("incomplete \\x escape",start,self.pos)); }
                let hex=&self.src[self.pos..self.pos+2];
                let val=u8::from_str_radix(hex,16).map_err(|_| LexError::new(format!("invalid \\x: {}",hex),start,self.pos))?;
                self.pos+=2; Ok(val as char)
            }
            b'u' => {
                if self.pos>=self.bytes.len()||self.bytes[self.pos]!=b'{' { return Err(LexError::new("expected { after \\u",start,self.pos)); }
                self.pos+=1; let hs=self.pos;
                while self.pos<self.bytes.len()&&self.bytes[self.pos]!=b'}' { self.pos+=1; }
                if self.pos>=self.bytes.len() { return Err(LexError::new("unterminated \\u{}",start,self.pos)); }
                let hex=&self.src[hs..self.pos]; self.pos+=1;
                let code=u32::from_str_radix(hex,16).map_err(|_| LexError::new(format!("invalid \\u{{}}: {}",hex),start,self.pos))?;
                char::from_u32(code).ok_or_else(|| LexError::new(format!("invalid codepoint U+{:X}",code),start,self.pos))
            }
            _ => Err(LexError::new(format!("invalid escape \\{}",b as char),start,self.pos))
        }
    }
}
fn is_ident_start(b: u8) -> bool { b.is_ascii_alphabetic() || b==b'_' }
fn is_ident_cont(b: u8) -> bool { b.is_ascii_alphanumeric() || b==b'_' }
fn kw_or_ident(word: &str) -> TokenKind {
    match word {
        "fn"=>TokenKind::Fn,"trait"=>TokenKind::Trait,"impl"=>TokenKind::Impl,
        "struct"=>TokenKind::Struct,"enum"=>TokenKind::Enum,"type"=>TokenKind::Type,
        "let"=>TokenKind::Let,"mut"=>TokenKind::Mut,"const"=>TokenKind::Const,
        "if"=>TokenKind::If,"else"=>TokenKind::Else,"while"=>TokenKind::While,
        "loop"=>TokenKind::Loop,"for"=>TokenKind::For,"in"=>TokenKind::In,
        "return"=>TokenKind::Return,"break"=>TokenKind::Break,"continue"=>TokenKind::Continue,"match"=>TokenKind::Match,"use"=>TokenKind::Use,
        "pub"=>TokenKind::Pub,"mod"=>TokenKind::Mod,"self"=>TokenKind::SelfVal,
        "Self"=>TokenKind::SelfType,"super"=>TokenKind::Super,"where"=>TokenKind::Where,
        "as"=>TokenKind::As,"move"=>TokenKind::Move,
        "true"=>TokenKind::BoolLit(true),"false"=>TokenKind::BoolLit(false),
        "extern"=>TokenKind::Extern,
        "sovereign"=>TokenKind::Sovereign,"capability"=>TokenKind::Capability,
        "profile"=>TokenKind::Profile,"patchable"=>TokenKind::Patchable,
        "requires"=>TokenKind::Requires,"ensures"=>TokenKind::Ensures,
        "invariant"=>TokenKind::Invariant,"ghost"=>TokenKind::Ghost,
        "pure"=>TokenKind::Pure,"unsafe_axon"=>TokenKind::UnsafeAxon,
        "asm"=>TokenKind::Asm,
        _=>TokenKind::Ident(word.to_string()),
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn kinds(src: &str) -> Vec<TokenKind> {
        Lexer::new(src).tokenize().unwrap().into_iter()
            .filter(|t| t.kind != TokenKind::Eof).map(|t| t.kind).collect()
    }
    #[test] fn t1_keywords() {
        let src = "fn trait impl struct enum type let mut const if else while loop for in return match use pub mod self Self super where as move";
        assert_eq!(kinds(src), vec![TokenKind::Fn,TokenKind::Trait,TokenKind::Impl,TokenKind::Struct,TokenKind::Enum,TokenKind::Type,TokenKind::Let,TokenKind::Mut,TokenKind::Const,TokenKind::If,TokenKind::Else,TokenKind::While,TokenKind::Loop,TokenKind::For,TokenKind::In,TokenKind::Return,TokenKind::Match,TokenKind::Use,TokenKind::Pub,TokenKind::Mod,TokenKind::SelfVal,TokenKind::SelfType,TokenKind::Super,TokenKind::Where,TokenKind::As,TokenKind::Move]);
    }
    #[test] fn t2_axon_keywords() {
        assert_eq!(kinds("sovereign capability profile patchable requires ensures invariant ghost pure unsafe_axon"),
            vec![TokenKind::Sovereign,TokenKind::Capability,TokenKind::Profile,TokenKind::Patchable,TokenKind::Requires,TokenKind::Ensures,TokenKind::Invariant,TokenKind::Ghost,TokenKind::Pure,TokenKind::UnsafeAxon]);
    }
    #[test] fn t3_contract_annotations() {
        assert_eq!(kinds("@requires @ensures @invariant"), vec![TokenKind::AtRequires,TokenKind::AtEnsures,TokenKind::AtInvariant]);
    }
    #[test] fn t3b_at_plain() {
        let t = kinds("@foo"); assert_eq!(t[0], TokenKind::At); assert_eq!(t[1], TokenKind::Ident("foo".into()));
    }
    #[test] fn t4_patchable_attr() {
        assert_eq!(kinds("#[patchable]"), vec![TokenKind::Pound,TokenKind::LBracket,TokenKind::Patchable,TokenKind::RBracket]);
    }
    #[test] fn t5_int_lit() {
        assert_eq!(kinds("42"),        vec![TokenKind::IntLit(42)]);
        assert_eq!(kinds("0xFF"),      vec![TokenKind::IntLit(255)]);
        assert_eq!(kinds("0b1010"),    vec![TokenKind::IntLit(10)]);
        assert_eq!(kinds("0o17"),      vec![TokenKind::IntLit(15)]);
        assert_eq!(kinds("1_000_000"),vec![TokenKind::IntLit(1_000_000)]);
    }
    #[test] fn t5_int_overflow() { assert!(Lexer::new("18446744073709551616").tokenize().is_err()); }
    #[test] fn t6_float_lit() {
        assert_eq!(kinds("3.14"),   vec![TokenKind::FloatLit(3.14)]);
        assert_eq!(kinds("1e10"),   vec![TokenKind::FloatLit(1e10)]);
        assert_eq!(kinds("1.5e-3"),vec![TokenKind::FloatLit(1.5e-3)]);
    }
    #[test] fn t7_string_lit() { assert_eq!(kinds("\"hello\""), vec![TokenKind::StringLit("hello".into())]); }
    #[test] fn t7_string_escapes() { assert_eq!(kinds("\"\\n\\t\""), vec![TokenKind::StringLit("\n\t".into())]); }
    #[test] fn t7_string_invalid_escape() { assert!(Lexer::new("\"\\q\"").tokenize().is_err()); }
    #[test] fn t7_string_unterminated() { assert!(Lexer::new("\"hello").tokenize().is_err()); }
    #[test] fn t8_char_lit() {
        assert_eq!(kinds("'a'"),    vec![TokenKind::CharLit('a')]);
        assert_eq!(kinds("'\\n'"), vec![TokenKind::CharLit('\n')]);
        assert_eq!(kinds("'\\t'"), vec![TokenKind::CharLit('\t')]);
    }
    #[test] fn t9_lifetime() {
        assert_eq!(kinds("'a"),      vec![TokenKind::Lifetime("a".into())]);
        assert_eq!(kinds("'static"),vec![TokenKind::Lifetime("static".into())]);
        assert_eq!(kinds("'src"),   vec![TokenKind::Lifetime("src".into())]);
    }
    #[test] fn t10_symbols() {
        assert_eq!(kinds("+"),   vec![TokenKind::Plus]);
        assert_eq!(kinds("->"),  vec![TokenKind::Arrow]);
        assert_eq!(kinds("=>"),  vec![TokenKind::FatArrow]);
        assert_eq!(kinds("::"),  vec![TokenKind::ColonColon]);
        assert_eq!(kinds("..="),vec![TokenKind::DotDotEq]);
        assert_eq!(kinds("<<="),vec![TokenKind::LtLtEq]);
        assert_eq!(kinds(">>="),vec![TokenKind::GtGtEq]);
        assert_eq!(kinds("&&"), vec![TokenKind::AmpAmp]);
        assert_eq!(kinds("||"), vec![TokenKind::PipePipe]);
        assert_eq!(kinds("??"), vec![TokenKind::QuestQuest]);
        assert_eq!(kinds("!="), vec![TokenKind::BangEq]);
        assert_eq!(kinds("=="), vec![TokenKind::EqEq]);
    }
    #[test] fn t11_line_comment() { assert_eq!(kinds("// comment\nfn"), vec![TokenKind::Fn]); }
    #[test] fn t12_nested_block_comment() { assert_eq!(kinds("/* /* inner */ outer */ fn"), vec![TokenKind::Fn]); }
    #[test] fn t12_unterminated_block() { assert!(Lexer::new("/* not closed").tokenize().is_err()); }
    #[test] fn t13_span_accuracy() {
        let tokens = Lexer::new("fn foo").tokenize().unwrap();
        assert_eq!(tokens[0].span.start, 0); assert_eq!(tokens[0].span.end, 2);
        assert_eq!(tokens[1].span.start, 3); assert_eq!(tokens[1].span.end, 6);
    }
    #[test] fn t14_no_panics() {
        assert!(Lexer::new("\"unterminated").tokenize().is_err());
        assert!(Lexer::new("\"\\z\"").tokenize().is_err());
        assert!(Lexer::new("99999999999999999999").tokenize().is_err());
    }
    #[test] fn t15_bool_lits() {
        assert_eq!(kinds("true false"), vec![TokenKind::BoolLit(true),TokenKind::BoolLit(false)]);
    }
}
