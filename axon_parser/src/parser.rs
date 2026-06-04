// axon_parser/src/parser.rs -- AXON Recursive Descent Parser Stage 8A-2
use crate::lexer::{Lexer, LexError, Token, TokenKind, Span};

#[derive(Debug, Clone, PartialEq)]
pub struct Ident { pub name: String, pub span: Span }

#[derive(Debug, Clone, PartialEq)]
pub enum Ty {
    Named(Ident, Vec<Ty>),
    Ref(bool, Option<String>, Box<Ty>),
    Ptr(bool, Box<Ty>),
    Slice(Box<Ty>),
    Array(Box<Ty>, Box<Expr>),
    Tuple(Vec<Ty>),
    Fn(Vec<Ty>, Option<Box<Ty>>),
    Never,
    Infer,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Pat {
    Wildcard(Span),
    Ident(Ident, bool),
    Tuple(Vec<Pat>, Span),
    Struct(Ident, Vec<(Ident, Pat)>, Span),
    Enum(Ident, Vec<Pat>, Span),
    Lit(Lit, Span),
    Ref(bool, Box<Pat>, Span),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Lit { Int(u64), Float(f64), Str(String), Char(char), Bool(bool) }

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Lit(Lit, Span),
    Ident(Ident),
    Block(Vec<Stmt>, Option<Box<Expr>>, Span),
    Call(Box<Expr>, Vec<Expr>, Span),
    MethodCall(Box<Expr>, Ident, Vec<Expr>, Span),
    Field(Box<Expr>, Ident, Span),
    Index(Box<Expr>, Box<Expr>, Span),
    Unary(UnaryOp, Box<Expr>, Span),
    Binary(BinaryOp, Box<Expr>, Box<Expr>, Span),
    Assign(Box<Expr>, Box<Expr>, Span),
    AssignOp(BinaryOp, Box<Expr>, Box<Expr>, Span),
    If(Box<Expr>, Box<Expr>, Option<Box<Expr>>, Span),
    While(Box<Expr>, Box<Expr>, Span),
    Loop(Box<Expr>, Span),
    For(Pat, Box<Expr>, Box<Expr>, Span),
    Match(Box<Expr>, Vec<MatchArm>, Span),
    Return(Option<Box<Expr>>, Span),
    Break(Option<Box<Expr>>, Span),
    Continue(Span),
    Struct(Ident, Vec<(Ident, Expr)>, Span),
    Tuple(Vec<Expr>, Span),
    Array(Vec<Expr>, Span),
    Cast(Box<Expr>, Box<Ty>, Span),
    Ref(bool, Box<Expr>, Span),
    Deref(Box<Expr>, Span),
    Range(Option<Box<Expr>>, Option<Box<Expr>>, bool, Span),
    Path(Vec<Ident>, Span),
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOp { Neg, Not, Deref, Ref, RefMut }

#[derive(Debug, Clone, PartialEq)]
pub enum BinaryOp {
    Add, Sub, Mul, Div, Rem,
    And, Or, BitAnd, BitOr, BitXor,
    Shl, Shr, Eq, Ne, Lt, Le, Gt, Ge,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MatchArm { pub pat: Pat, pub guard: Option<Expr>, pub body: Expr, pub span: Span }

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Let(Pat, Option<Ty>, Option<Expr>, Span),
    Expr(Expr, bool),
    Item(Item),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Contract { pub kind: ContractKind, pub expr: Expr, pub span: Span }

#[derive(Debug, Clone, PartialEq)]
pub enum ContractKind { Requires, Ensures, Invariant }

#[derive(Debug, Clone, PartialEq)]
pub struct Attr { pub name: String, pub args: Vec<String>, pub span: Span }

#[derive(Debug, Clone, PartialEq)]
pub struct GenericParam { pub name: String, pub bounds: Vec<Ty>, pub span: Span }

#[derive(Debug, Clone, PartialEq)]
pub struct FnParam { pub pat: Pat, pub ty: Ty, pub span: Span }

#[derive(Debug, Clone, PartialEq)]
pub struct FnSig {
    pub name: Ident, pub generics: Vec<GenericParam>,
    pub params: Vec<FnParam>, pub ret: Option<Ty>,
    pub contracts: Vec<Contract>, pub attrs: Vec<Attr>,
    pub is_pub: bool, pub is_pure: bool, pub is_ghost: bool, pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Item {
    Fn(FnSig, Expr), Struct(StructDef), Enum(EnumDef),
    Trait(TraitDef), Impl(ImplBlock),
    TypeAlias(Ident, Vec<GenericParam>, Ty, Span),
    Use(Vec<String>, Span), Mod(Ident, Vec<Item>, Span),
    Profile(ProfileDef), Const(Ident, Ty, Expr, Span),
}

#[derive(Debug, Clone, PartialEq)]
pub struct StructDef {
    pub name: Ident, pub generics: Vec<GenericParam>,
    pub fields: Vec<FieldDef>, pub attrs: Vec<Attr>,
    pub is_pub: bool, pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FieldDef { pub name: Ident, pub ty: Ty, pub is_pub: bool, pub span: Span }

#[derive(Debug, Clone, PartialEq)]
pub struct EnumDef {
    pub name: Ident, pub generics: Vec<GenericParam>,
    pub variants: Vec<EnumVariant>, pub attrs: Vec<Attr>,
    pub is_pub: bool, pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EnumVariant { pub name: Ident, pub fields: EnumVariantFields, pub span: Span }

#[derive(Debug, Clone, PartialEq)]
pub enum EnumVariantFields { Unit, Tuple(Vec<Ty>), Struct(Vec<FieldDef>) }

#[derive(Debug, Clone, PartialEq)]
pub struct TraitDef {
    pub name: Ident, pub generics: Vec<GenericParam>,
    pub supertraits: Vec<Ty>, pub items: Vec<TraitItem>,
    pub attrs: Vec<Attr>, pub is_pub: bool, pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TraitItem {
    Fn(FnSig, Option<Expr>),
    Type(Ident, Vec<Ty>, Span),
    Const(Ident, Ty, Option<Expr>, Span),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ImplBlock {
    pub generics: Vec<GenericParam>, pub trait_: Option<Ty>,
    pub self_ty: Ty, pub items: Vec<ImplItem>,
    pub attrs: Vec<Attr>, pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ImplItem {
    Fn(FnSig, Expr), Type(Ident, Ty, Span), Const(Ident, Ty, Expr, Span),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProfileDef { pub name: Ident, pub capabilities: Vec<String>, pub span: Span }

#[derive(Debug, Clone)]
pub struct ParseError { pub msg: String, pub span: Span }
impl ParseError {
    pub fn new(msg: impl Into<String>, span: Span) -> Self { ParseError { msg: msg.into(), span } }
}
impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ParseError at {}..{}: {}", self.span.start, self.span.end, self.msg)
    }
}

pub struct Parser { tokens: Vec<Token>, pos: usize, allow_struct_lit: bool, expr_depth: usize }

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self { Parser { tokens, pos: 0, allow_struct_lit: true, expr_depth: 0 } }
    pub fn from_source(src: &str) -> Result<Self, LexError> {
        Ok(Parser::new(Lexer::new(src).tokenize()?))
    }
    pub fn parse_program(&mut self) -> Result<Vec<Item>, ParseError> {
        let mut items = Vec::new();
        while !self.is_eof() { items.push(self.parse_item()?); }
        Ok(items)
    }
    fn peek(&self) -> &TokenKind { &self.tokens[self.pos].kind }
    fn current_span(&self) -> Span { self.tokens[self.pos].span.clone() }
    fn advance(&mut self) -> &Token {
        let t = &self.tokens[self.pos];
        if self.pos + 1 < self.tokens.len() { self.pos += 1; }
        t
    }
    fn is_eof(&self) -> bool { matches!(self.peek(), TokenKind::Eof) }
    fn expect(&mut self, kind: &TokenKind) -> Result<Token, ParseError> {
        if self.peek() == kind { Ok(self.advance().clone()) }
        else { Err(ParseError::new(format!("expected {:?}, got {:?}", kind, self.peek()), self.current_span())) }
    }
    fn expect_ident(&mut self) -> Result<Ident, ParseError> {
        let span = self.current_span();
        match self.peek().clone() {
            TokenKind::Ident(name) => { self.advance(); Ok(Ident { name, span }) }
            other => Err(ParseError::new(format!("expected identifier, got {:?}", other), span))
        }
    }
    fn eat(&mut self, kind: &TokenKind) -> bool {
        if self.peek() == kind { self.advance(); true } else { false }
    }
    fn parse_attrs(&mut self) -> Result<Vec<Attr>, ParseError> {
        let mut attrs = Vec::new();
        while matches!(self.peek(), TokenKind::Pound) {
            let start = self.current_span().start;
            self.advance();
            self.expect(&TokenKind::LBracket)?;
            let name_span = self.current_span();
            let name = match self.peek().clone() {
                TokenKind::Ident(n) => { self.advance(); n }
                TokenKind::Patchable => { self.advance(); "patchable".to_string() }
                TokenKind::Sovereign => { self.advance(); "sovereign".to_string() }
                other => return Err(ParseError::new(format!("expected attr name, got {:?}", other), name_span)),
            };
            let mut args = Vec::new();
            if self.eat(&TokenKind::LParen) {
                while !matches!(self.peek(), TokenKind::RParen | TokenKind::Eof) {
                    match self.peek().clone() {
                        TokenKind::Ident(s) => { self.advance(); args.push(s); }
                        TokenKind::Comma => { self.advance(); }
                        _ => { self.advance(); }
                    }
                }
                self.expect(&TokenKind::RParen)?;
            }
            let end = self.current_span().end;
            self.expect(&TokenKind::RBracket)?;
            attrs.push(Attr { name, args, span: Span::new(start, end) });
        }
        Ok(attrs)
    }
    fn parse_contracts(&mut self) -> Result<Vec<Contract>, ParseError> {
        let mut contracts = Vec::new();
        loop {
            let kind = match self.peek() {
                TokenKind::AtRequires  => ContractKind::Requires,
                TokenKind::AtEnsures   => ContractKind::Ensures,
                TokenKind::AtInvariant => ContractKind::Invariant,
                _ => break,
            };
            let start = self.current_span().start;
            self.advance();
            self.expect(&TokenKind::LParen)?;
            let expr = self.parse_expr()?;
            self.expect(&TokenKind::RParen)?;
            let end = self.current_span().end;
            contracts.push(Contract { kind, expr, span: Span::new(start, end) });
        }
        Ok(contracts)
    }
    fn parse_generic_params(&mut self) -> Result<Vec<GenericParam>, ParseError> {
        if !matches!(self.peek(), TokenKind::Lt) { return Ok(vec![]); }
        self.advance();
        let mut params = Vec::new();
        while !matches!(self.peek(), TokenKind::Gt | TokenKind::Eof) {
            let span = self.current_span();
            let name = match self.peek().clone() {
                TokenKind::Ident(n) => { self.advance(); n }
                TokenKind::Lifetime(l) => { self.advance(); format!("'{}",l) }
                _ => return Err(ParseError::new("expected generic param", span)),
            };
            let mut bounds = Vec::new();
            if self.eat(&TokenKind::Colon) {
                bounds.push(self.parse_ty()?);
                while self.eat(&TokenKind::Plus) { bounds.push(self.parse_ty()?); }
            }
            params.push(GenericParam { name, bounds, span });
            if !self.eat(&TokenKind::Comma) { break; }
        }
        self.expect(&TokenKind::Gt)?;
        Ok(params)
    }
    fn parse_ty(&mut self) -> Result<Ty, ParseError> {
        match self.peek().clone() {
            TokenKind::Amp => {
                self.advance();
                let lifetime = if let TokenKind::Lifetime(l) = self.peek().clone() { self.advance(); Some(l) } else { None };
                let is_mut = self.eat(&TokenKind::Mut);
                Ok(Ty::Ref(is_mut, lifetime, Box::new(self.parse_ty()?)))
            }
            TokenKind::Star => {
                self.advance();
                let is_mut = if self.eat(&TokenKind::Mut) { true } else { self.eat(&TokenKind::Const); false };
                Ok(Ty::Ptr(is_mut, Box::new(self.parse_ty()?)))
            }
            TokenKind::Bang => { self.advance(); Ok(Ty::Never) }
            TokenKind::LParen => {
                self.advance();
                if self.eat(&TokenKind::RParen) { return Ok(Ty::Tuple(vec![])); }
                let first = self.parse_ty()?;
                if self.eat(&TokenKind::RParen) { return Ok(first); }
                let mut tys = vec![first];
                while self.eat(&TokenKind::Comma) {
                    if matches!(self.peek(), TokenKind::RParen) { break; }
                    tys.push(self.parse_ty()?);
                }
                self.expect(&TokenKind::RParen)?;
                Ok(Ty::Tuple(tys))
            }
            TokenKind::LBracket => {
                self.advance();
                let elem = self.parse_ty()?;
                if self.eat(&TokenKind::Semi) {
                    let len = self.parse_expr()?;
                    self.expect(&TokenKind::RBracket)?;
                    Ok(Ty::Array(Box::new(elem), Box::new(len)))
                } else {
                    self.expect(&TokenKind::RBracket)?;
                    Ok(Ty::Slice(Box::new(elem)))
                }
            }
            TokenKind::Fn => {
                self.advance();
                self.expect(&TokenKind::LParen)?;
                let mut params = Vec::new();
                while !matches!(self.peek(), TokenKind::RParen | TokenKind::Eof) {
                    params.push(self.parse_ty()?);
                    if !self.eat(&TokenKind::Comma) { break; }
                }
                self.expect(&TokenKind::RParen)?;
                let ret = if self.eat(&TokenKind::Arrow) { Some(Box::new(self.parse_ty()?)) } else { None };
                Ok(Ty::Fn(params, ret))
            }
            _ => {
                let span = self.current_span();
                let name = match self.peek().clone() {
                    TokenKind::Ident(n) => { self.advance(); n }
                    TokenKind::SelfType => { self.advance(); "Self".to_string() }
                    other => return Err(ParseError::new(format!("expected type, got {:?}", other), span)),
                };
                let ident = Ident { name, span };
                let args = if matches!(self.peek(), TokenKind::Lt) {
                    self.advance();
                    let mut args = Vec::new();
                    while !matches!(self.peek(), TokenKind::Gt | TokenKind::Eof) {
                        if let TokenKind::Lifetime(_) = self.peek().clone() { self.advance(); }
                        else { args.push(self.parse_ty()?); }
                        if !self.eat(&TokenKind::Comma) { break; }
                    }
                    self.expect(&TokenKind::Gt)?;
                    args
                } else { vec![] };
                Ok(Ty::Named(ident, args))
            }
        }
    }
    fn parse_item(&mut self) -> Result<Item, ParseError> {
        let attrs = self.parse_attrs()?;
        let contracts = self.parse_contracts()?;
        let is_pub = self.eat(&TokenKind::Pub);
        match self.peek().clone() {
            TokenKind::Fn      => self.parse_fn(attrs, contracts, is_pub),
            TokenKind::Struct  => self.parse_struct(attrs, is_pub),
            TokenKind::Enum    => self.parse_enum(attrs, is_pub),
            TokenKind::Trait   => self.parse_trait(attrs, is_pub),
            TokenKind::Impl    => self.parse_impl(attrs),
            TokenKind::Type    => self.parse_type_alias(is_pub),
            TokenKind::Use     => self.parse_use(),
            TokenKind::Mod     => self.parse_mod(is_pub),
            TokenKind::Profile => self.parse_profile(),
            TokenKind::Const   => self.parse_const(is_pub),
            other => Err(ParseError::new(format!("expected item, got {:?}", other), self.current_span())),
        }
    }
    fn parse_fn(&mut self, attrs: Vec<Attr>, contracts: Vec<Contract>, is_pub: bool) -> Result<Item, ParseError> {
        let start = self.current_span().start;
        self.expect(&TokenKind::Fn)?;
        let name = self.expect_ident()?;
        let generics = self.parse_generic_params()?;
        self.expect(&TokenKind::LParen)?;
        let mut params = Vec::new();
        while !matches!(self.peek(), TokenKind::RParen | TokenKind::Eof) {
            params.push(self.parse_fn_param()?);
            if !self.eat(&TokenKind::Comma) { break; }
        }
        self.expect(&TokenKind::RParen)?;
        let ret = if self.eat(&TokenKind::Arrow) { Some(self.parse_ty()?) } else { None };
        let mut all_contracts = contracts;
        all_contracts.extend(self.parse_contracts()?);
        let is_pure  = attrs.iter().any(|a| a.name == "pure");
        let is_ghost = attrs.iter().any(|a| a.name == "ghost");
        let end = self.current_span().end;
        let sig = FnSig { name, generics, params, ret, contracts: all_contracts, attrs, is_pub, is_pure, is_ghost, span: Span::new(start, end) };
        let body = self.parse_block_expr()?;
        Ok(Item::Fn(sig, body))
    }
    fn parse_fn_param(&mut self) -> Result<FnParam, ParseError> {
        let span = self.current_span();
        if matches!(self.peek(), TokenKind::SelfVal) {
            self.advance();
            return Ok(FnParam { pat: Pat::Ident(Ident { name: "self".into(), span: span.clone() }, false), ty: Ty::Named(Ident { name: "Self".into(), span: span.clone() }, vec![]), span });
        }
        if matches!(self.peek(), TokenKind::Amp) {
            let ref_span = self.current_span(); self.advance();
            let is_mut = self.eat(&TokenKind::Mut);
            if matches!(self.peek(), TokenKind::SelfVal) {
                self.advance();
                return Ok(FnParam { pat: Pat::Ident(Ident { name: "self".into(), span: span.clone() }, is_mut), ty: Ty::Ref(is_mut, None, Box::new(Ty::Named(Ident { name: "Self".into(), span: ref_span }, vec![]))), span });
            }
        }
        let pat = self.parse_pat()?;
        self.expect(&TokenKind::Colon)?;
        let ty = self.parse_ty()?;
        Ok(FnParam { pat, ty, span })
    }
    fn parse_struct(&mut self, attrs: Vec<Attr>, is_pub: bool) -> Result<Item, ParseError> {
        let start = self.current_span().start;
        self.expect(&TokenKind::Struct)?;
        let name = self.expect_ident()?;
        let generics = self.parse_generic_params()?;
        self.expect(&TokenKind::LBrace)?;
        let mut fields = Vec::new();
        while !matches!(self.peek(), TokenKind::RBrace | TokenKind::Eof) {
            let fspan = self.current_span();
            let fis_pub = self.eat(&TokenKind::Pub);
            let fname = self.expect_ident()?;
            self.expect(&TokenKind::Colon)?;
            let fty = self.parse_ty()?;
            fields.push(FieldDef { name: fname, ty: fty, is_pub: fis_pub, span: fspan });
            if !self.eat(&TokenKind::Comma) { break; }
        }
        let end = self.current_span().end;
        self.expect(&TokenKind::RBrace)?;
        Ok(Item::Struct(StructDef { name, generics, fields, attrs, is_pub, span: Span::new(start, end) }))
    }
    fn parse_enum(&mut self, attrs: Vec<Attr>, is_pub: bool) -> Result<Item, ParseError> {
        let start = self.current_span().start;
        self.expect(&TokenKind::Enum)?;
        let name = self.expect_ident()?;
        let generics = self.parse_generic_params()?;
        self.expect(&TokenKind::LBrace)?;
        let mut variants = Vec::new();
        while !matches!(self.peek(), TokenKind::RBrace | TokenKind::Eof) {
            let vspan = self.current_span();
            let vname = self.expect_ident()?;
            let fields = match self.peek() {
                TokenKind::LParen => {
                    self.advance();
                    let mut tys = Vec::new();
                    while !matches!(self.peek(), TokenKind::RParen | TokenKind::Eof) {
                        tys.push(self.parse_ty()?);
                        if !self.eat(&TokenKind::Comma) { break; }
                    }
                    self.expect(&TokenKind::RParen)?;
                    EnumVariantFields::Tuple(tys)
                }
                TokenKind::LBrace => {
                    self.advance();
                    let mut fdefs = Vec::new();
                    while !matches!(self.peek(), TokenKind::RBrace | TokenKind::Eof) {
                        let fspan = self.current_span();
                        let fname = self.expect_ident()?;
                        self.expect(&TokenKind::Colon)?;
                        let fty = self.parse_ty()?;
                        fdefs.push(FieldDef { name: fname, ty: fty, is_pub: false, span: fspan });
                        if !self.eat(&TokenKind::Comma) { break; }
                    }
                    self.expect(&TokenKind::RBrace)?;
                    EnumVariantFields::Struct(fdefs)
                }
                _ => EnumVariantFields::Unit,
            };
            variants.push(EnumVariant { name: vname, fields, span: vspan });
            if !self.eat(&TokenKind::Comma) { break; }
        }
        let end = self.current_span().end;
        self.expect(&TokenKind::RBrace)?;
        Ok(Item::Enum(EnumDef { name, generics, variants, attrs, is_pub, span: Span::new(start, end) }))
    }
    fn parse_trait(&mut self, attrs: Vec<Attr>, is_pub: bool) -> Result<Item, ParseError> {
        let start = self.current_span().start;
        self.expect(&TokenKind::Trait)?;
        let name = self.expect_ident()?;
        let generics = self.parse_generic_params()?;
        let mut supertraits = Vec::new();
        if self.eat(&TokenKind::Colon) {
            supertraits.push(self.parse_ty()?);
            while self.eat(&TokenKind::Plus) { supertraits.push(self.parse_ty()?); }
        }
        self.expect(&TokenKind::LBrace)?;
        let mut items = Vec::new();
        while !matches!(self.peek(), TokenKind::RBrace | TokenKind::Eof) {
            let ia = self.parse_attrs()?;
            let ic = self.parse_contracts()?;
            let ip = self.eat(&TokenKind::Pub);
            match self.peek() {
                TokenKind::Fn => {
                    if let Item::Fn(sig, body) = self.parse_fn(ia, ic, ip)? {
                        items.push(TraitItem::Fn(sig, Some(body)));
                    }
                }
                TokenKind::Type => {
                    let tspan = self.current_span(); self.advance();
                    let tname = self.expect_ident()?;
                    let mut bounds = Vec::new();
                    if self.eat(&TokenKind::Colon) {
                        bounds.push(self.parse_ty()?);
                        while self.eat(&TokenKind::Plus) { bounds.push(self.parse_ty()?); }
                    }
                    self.expect(&TokenKind::Semi)?;
                    items.push(TraitItem::Type(tname, bounds, tspan));
                }
                TokenKind::Const => {
                    let cspan = self.current_span(); self.advance();
                    let cname = self.expect_ident()?;
                    self.expect(&TokenKind::Colon)?;
                    let cty = self.parse_ty()?;
                    let cval = if self.eat(&TokenKind::Eq) { Some(self.parse_expr()?) } else { None };
                    self.expect(&TokenKind::Semi)?;
                    items.push(TraitItem::Const(cname, cty, cval, cspan));
                }
                _ => return Err(ParseError::new(format!("expected trait item, got {:?}", self.peek()), self.current_span())),
            }
        }
        let end = self.current_span().end;
        self.expect(&TokenKind::RBrace)?;
        Ok(Item::Trait(TraitDef { name, generics, supertraits, items, attrs, is_pub, span: Span::new(start, end) }))
    }
    fn parse_impl(&mut self, attrs: Vec<Attr>) -> Result<Item, ParseError> {
        let start = self.current_span().start;
        self.expect(&TokenKind::Impl)?;
        let generics = self.parse_generic_params()?;
        let first_ty = self.parse_ty()?;
        let (trait_, self_ty) = if self.eat(&TokenKind::For) { (Some(first_ty), self.parse_ty()?) } else { (None, first_ty) };
        self.expect(&TokenKind::LBrace)?;
        let mut items = Vec::new();
        while !matches!(self.peek(), TokenKind::RBrace | TokenKind::Eof) {
            let ia = self.parse_attrs()?;
            let ic = self.parse_contracts()?;
            let ip = self.eat(&TokenKind::Pub);
            match self.peek() {
                TokenKind::Fn => {
                    if let Item::Fn(sig, body) = self.parse_fn(ia, ic, ip)? {
                        items.push(ImplItem::Fn(sig, body));
                    }
                }
                TokenKind::Type => {
                    let tspan = self.current_span(); self.advance();
                    let tname = self.expect_ident()?;
                    self.expect(&TokenKind::Eq)?;
                    let tty = self.parse_ty()?;
                    self.expect(&TokenKind::Semi)?;
                    items.push(ImplItem::Type(tname, tty, tspan));
                }
                TokenKind::Const => {
                    let cspan = self.current_span(); self.advance();
                    let cname = self.expect_ident()?;
                    self.expect(&TokenKind::Colon)?;
                    let cty = self.parse_ty()?;
                    self.expect(&TokenKind::Eq)?;
                    let cv = self.parse_expr()?;
                    self.expect(&TokenKind::Semi)?;
                    items.push(ImplItem::Const(cname, cty, cv, cspan));
                }
                _ => return Err(ParseError::new(format!("expected impl item, got {:?}", self.peek()), self.current_span())),
            }
        }
        let end = self.current_span().end;
        self.expect(&TokenKind::RBrace)?;
        Ok(Item::Impl(ImplBlock { generics, trait_, self_ty, items, attrs, span: Span::new(start, end) }))
    }
    fn parse_type_alias(&mut self, _is_pub: bool) -> Result<Item, ParseError> {
        let start = self.current_span().start;
        self.expect(&TokenKind::Type)?;
        let name = self.expect_ident()?;
        let generics = self.parse_generic_params()?;
        self.expect(&TokenKind::Eq)?;
        let ty = self.parse_ty()?;
        let end = self.current_span().end;
        self.expect(&TokenKind::Semi)?;
        Ok(Item::TypeAlias(name, generics, ty, Span::new(start, end)))
    }
    fn parse_use(&mut self) -> Result<Item, ParseError> {
        let start = self.current_span().start;
        self.expect(&TokenKind::Use)?;
        let mut path = vec![self.expect_ident()?.name];
        while self.eat(&TokenKind::ColonColon) { path.push(self.expect_ident()?.name); }
        let end = self.current_span().end;
        self.expect(&TokenKind::Semi)?;
        Ok(Item::Use(path, Span::new(start, end)))
    }
    fn parse_mod(&mut self, _is_pub: bool) -> Result<Item, ParseError> {
        let start = self.current_span().start;
        self.expect(&TokenKind::Mod)?;
        let name = self.expect_ident()?;
        self.expect(&TokenKind::LBrace)?;
        let mut items = Vec::new();
        while !matches!(self.peek(), TokenKind::RBrace | TokenKind::Eof) { items.push(self.parse_item()?); }
        let end = self.current_span().end;
        self.expect(&TokenKind::RBrace)?;
        Ok(Item::Mod(name, items, Span::new(start, end)))
    }
    fn parse_profile(&mut self) -> Result<Item, ParseError> {
        let start = self.current_span().start;
        self.expect(&TokenKind::Profile)?;
        let name = self.expect_ident()?;
        self.expect(&TokenKind::LBrace)?;
        let mut capabilities = Vec::new();
        while !matches!(self.peek(), TokenKind::RBrace | TokenKind::Eof) {
            match self.peek().clone() {
                TokenKind::Ident(cap) => { self.advance(); capabilities.push(cap); }
                _ => { self.advance(); }
            }
            self.eat(&TokenKind::Comma);
            self.eat(&TokenKind::Semi);
        }
        let end = self.current_span().end;
        self.expect(&TokenKind::RBrace)?;
        Ok(Item::Profile(ProfileDef { name, capabilities, span: Span::new(start, end) }))
    }
    fn parse_const(&mut self, _is_pub: bool) -> Result<Item, ParseError> {
        let start = self.current_span().start;
        self.expect(&TokenKind::Const)?;
        let name = self.expect_ident()?;
        self.expect(&TokenKind::Colon)?;
        let ty = self.parse_ty()?;
        self.expect(&TokenKind::Eq)?;
        let val = self.parse_expr()?;
        let end = self.current_span().end;
        self.expect(&TokenKind::Semi)?;
        Ok(Item::Const(name, ty, val, Span::new(start, end)))
    }
    fn parse_block_expr(&mut self) -> Result<Expr, ParseError> {
        let start = self.current_span().start;
        self.expect(&TokenKind::LBrace)?;
        let mut stmts = Vec::new();
        let mut tail: Option<Box<Expr>> = None;
        while !matches!(self.peek(), TokenKind::RBrace | TokenKind::Eof) {
            let stmt = self.parse_stmt()?;
            match stmt {
                Stmt::Expr(e, false) if matches!(self.peek(), TokenKind::RBrace) => { tail = Some(Box::new(e)); break; }
                s => stmts.push(s),
            }
        }
        let end = self.current_span().end;
        self.expect(&TokenKind::RBrace)?;
        Ok(Expr::Block(stmts, tail, Span::new(start, end)))
    }
    fn parse_stmt(&mut self) -> Result<Stmt, ParseError> {
        let span = self.current_span();
        match self.peek() {
            TokenKind::Let => {
                self.advance();
                let pat = self.parse_pat()?;
                let ty = if self.eat(&TokenKind::Colon) { Some(self.parse_ty()?) } else { None };
                let val = if self.eat(&TokenKind::Eq) { Some(self.parse_expr()?) } else { None };
                self.expect(&TokenKind::Semi)?;
                Ok(Stmt::Let(pat, ty, val, span))
            }
            TokenKind::Fn | TokenKind::Struct | TokenKind::Enum | TokenKind::Trait |
            TokenKind::Impl | TokenKind::Use | TokenKind::Mod | TokenKind::Const => {
                Ok(Stmt::Item(self.parse_item()?))
            }
            _ => {
                let expr = self.parse_expr()?;
                let semi = self.eat(&TokenKind::Semi);
                Ok(Stmt::Expr(expr, semi))
            }
        }
    }
    fn parse_pat(&mut self) -> Result<Pat, ParseError> {
        let span = self.current_span();
        match self.peek().clone() {
            TokenKind::Ident(ref s) if s == "_" => {
                self.advance();
                Ok(Pat::Wildcard(span))
            }
            TokenKind::Mut => {
                self.advance();
                let i = self.expect_ident()?;
                Ok(Pat::Ident(i, true))
            }
            TokenKind::Amp => {
                self.advance();
                let m = self.eat(&TokenKind::Mut);
                Ok(Pat::Ref(m, Box::new(self.parse_pat()?), span))
            }
            TokenKind::LParen => {
                self.advance();
                let mut pats = Vec::new();
                while !matches!(self.peek(), TokenKind::RParen | TokenKind::Eof) {
                    pats.push(self.parse_pat()?);
                    if !self.eat(&TokenKind::Comma) { break; }
                }
                self.expect(&TokenKind::RParen)?;
                Ok(Pat::Tuple(pats, span))
            }
            TokenKind::IntLit(n) => {
                let val = n;
                self.advance();
                Ok(Pat::Lit(Lit::Int(val), span))
            }
            TokenKind::FloatLit(f) => {
                let val = f;
                self.advance();
                Ok(Pat::Lit(Lit::Float(val), span))
            }
            TokenKind::StringLit(s) => {
                let val = s;
                self.advance();
                Ok(Pat::Lit(Lit::Str(val), span))
            }
            TokenKind::CharLit(c) => {
                let val = c;
                self.advance();
                Ok(Pat::Lit(Lit::Char(val), span))
            }
            TokenKind::BoolLit(b) => {
                let val = b;
                self.advance();
                Ok(Pat::Lit(Lit::Bool(val), span))
            }
            TokenKind::Ident(_) => {
                let i = self.expect_ident()?;
                Ok(Pat::Ident(i, false))
            }
            other => Err(ParseError::new(
                format!("expected pattern, got {:?}", other),
                span,
            ))
        }
    }
    /// Maximum expression nesting depth — prevents stack overflow on pathological input (S2).
    const MAX_EXPR_DEPTH: usize = 256;

    fn parse_expr(&mut self) -> Result<Expr, ParseError> {
        self.expr_depth += 1;
        if self.expr_depth > Self::MAX_EXPR_DEPTH {
            self.expr_depth -= 1;
            return Err(ParseError::new(
                format!("expression nested too deeply (max depth {})", Self::MAX_EXPR_DEPTH),
                self.current_span(),
            ));
        }
        let result = self.parse_assign_expr();
        self.expr_depth -= 1;
        result
    }
    fn parse_assign_expr(&mut self) -> Result<Expr, ParseError> {
        let lhs = self.parse_range_expr()?;
        let span = self.current_span();
        match self.peek().clone() {
            TokenKind::Eq      => { self.advance(); Ok(Expr::Assign(Box::new(lhs), Box::new(self.parse_assign_expr()?), span)) }
            TokenKind::PlusEq  => { self.advance(); Ok(Expr::AssignOp(BinaryOp::Add, Box::new(lhs), Box::new(self.parse_assign_expr()?), span)) }
            TokenKind::MinusEq => { self.advance(); Ok(Expr::AssignOp(BinaryOp::Sub, Box::new(lhs), Box::new(self.parse_assign_expr()?), span)) }
            TokenKind::StarEq  => { self.advance(); Ok(Expr::AssignOp(BinaryOp::Mul, Box::new(lhs), Box::new(self.parse_assign_expr()?), span)) }
            TokenKind::SlashEq => { self.advance(); Ok(Expr::AssignOp(BinaryOp::Div, Box::new(lhs), Box::new(self.parse_assign_expr()?), span)) }
            _ => Ok(lhs)
        }
    }
    fn parse_range_expr(&mut self) -> Result<Expr, ParseError> {
        let span = self.current_span();
        if matches!(self.peek(), TokenKind::DotDot | TokenKind::DotDotEq) {
            let inc = matches!(self.peek(), TokenKind::DotDotEq); self.advance();
            let rhs = if !matches!(self.peek(), TokenKind::Semi | TokenKind::RBracket | TokenKind::RParen | TokenKind::Eof) { Some(Box::new(self.parse_or_expr()?)) } else { None };
            return Ok(Expr::Range(None, rhs, inc, span));
        }
        let lhs = self.parse_or_expr()?;
        if matches!(self.peek(), TokenKind::DotDot | TokenKind::DotDotEq) {
            let inc = matches!(self.peek(), TokenKind::DotDotEq); self.advance();
            let rhs = if !matches!(self.peek(), TokenKind::Semi | TokenKind::RBracket | TokenKind::RParen | TokenKind::Eof) { Some(Box::new(self.parse_or_expr()?)) } else { None };
            Ok(Expr::Range(Some(Box::new(lhs)), rhs, inc, span))
        } else { Ok(lhs) }
    }
    fn parse_or_expr(&mut self) -> Result<Expr, ParseError> {
        let mut lhs = self.parse_and_expr()?;
        while matches!(self.peek(), TokenKind::PipePipe) { let s = self.current_span(); self.advance(); lhs = Expr::Binary(BinaryOp::Or, Box::new(lhs), Box::new(self.parse_and_expr()?), s); }
        Ok(lhs)
    }
    fn parse_and_expr(&mut self) -> Result<Expr, ParseError> {
        let mut lhs = self.parse_cmp_expr()?;
        while matches!(self.peek(), TokenKind::AmpAmp) { let s = self.current_span(); self.advance(); lhs = Expr::Binary(BinaryOp::And, Box::new(lhs), Box::new(self.parse_cmp_expr()?), s); }
        Ok(lhs)
    }
    fn parse_cmp_expr(&mut self) -> Result<Expr, ParseError> {
        let mut lhs = self.parse_bitor_expr()?;
        loop {
            let s = self.current_span();
            let op = match self.peek() { TokenKind::EqEq=>BinaryOp::Eq, TokenKind::BangEq=>BinaryOp::Ne, TokenKind::Lt=>BinaryOp::Lt, TokenKind::LtEq=>BinaryOp::Le, TokenKind::Gt=>BinaryOp::Gt, TokenKind::GtEq=>BinaryOp::Ge, _=>break };
            self.advance(); lhs = Expr::Binary(op, Box::new(lhs), Box::new(self.parse_bitor_expr()?), s);
        }
        Ok(lhs)
    }
    fn parse_bitor_expr(&mut self) -> Result<Expr, ParseError> {
        let mut lhs = self.parse_bitxor_expr()?;
        while matches!(self.peek(), TokenKind::Pipe) { let s = self.current_span(); self.advance(); lhs = Expr::Binary(BinaryOp::BitOr, Box::new(lhs), Box::new(self.parse_bitxor_expr()?), s); }
        Ok(lhs)
    }
    fn parse_bitxor_expr(&mut self) -> Result<Expr, ParseError> {
        let mut lhs = self.parse_bitand_expr()?;
        while matches!(self.peek(), TokenKind::Caret) { let s = self.current_span(); self.advance(); lhs = Expr::Binary(BinaryOp::BitXor, Box::new(lhs), Box::new(self.parse_bitand_expr()?), s); }
        Ok(lhs)
    }
    fn parse_bitand_expr(&mut self) -> Result<Expr, ParseError> {
        let mut lhs = self.parse_shift_expr()?;
        while matches!(self.peek(), TokenKind::Amp) { let s = self.current_span(); self.advance(); lhs = Expr::Binary(BinaryOp::BitAnd, Box::new(lhs), Box::new(self.parse_shift_expr()?), s); }
        Ok(lhs)
    }
    fn parse_shift_expr(&mut self) -> Result<Expr, ParseError> {
        let mut lhs = self.parse_add_expr()?;
        loop {
            let s = self.current_span();
            let op = match self.peek() { TokenKind::LtLt=>BinaryOp::Shl, TokenKind::GtGt=>BinaryOp::Shr, _=>break };
            self.advance(); lhs = Expr::Binary(op, Box::new(lhs), Box::new(self.parse_add_expr()?), s);
        }
        Ok(lhs)
    }
    fn parse_add_expr(&mut self) -> Result<Expr, ParseError> {
        let mut lhs = self.parse_mul_expr()?;
        loop {
            let s = self.current_span();
            let op = match self.peek() { TokenKind::Plus=>BinaryOp::Add, TokenKind::Minus=>BinaryOp::Sub, _=>break };
            self.advance(); lhs = Expr::Binary(op, Box::new(lhs), Box::new(self.parse_mul_expr()?), s);
        }
        Ok(lhs)
    }
    fn parse_mul_expr(&mut self) -> Result<Expr, ParseError> {
        let mut lhs = self.parse_cast_expr()?;
        loop {
            let s = self.current_span();
            let op = match self.peek() { TokenKind::Star=>BinaryOp::Mul, TokenKind::Slash=>BinaryOp::Div, TokenKind::Percent=>BinaryOp::Rem, _=>break };
            self.advance(); lhs = Expr::Binary(op, Box::new(lhs), Box::new(self.parse_cast_expr()?), s);
        }
        Ok(lhs)
    }
    fn parse_cast_expr(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.parse_unary_expr()?;
        while matches!(self.peek(), TokenKind::As) { let s = self.current_span(); self.advance(); expr = Expr::Cast(Box::new(expr), Box::new(self.parse_ty()?), s); }
        Ok(expr)
    }
    fn parse_unary_expr(&mut self) -> Result<Expr, ParseError> {
        let s = self.current_span();
        match self.peek() {
            TokenKind::Minus => { self.advance(); Ok(Expr::Unary(UnaryOp::Neg, Box::new(self.parse_unary_expr()?), s)) }
            TokenKind::Bang  => { self.advance(); Ok(Expr::Unary(UnaryOp::Not, Box::new(self.parse_unary_expr()?), s)) }
            TokenKind::Star  => { self.advance(); Ok(Expr::Deref(Box::new(self.parse_unary_expr()?), s)) }
            TokenKind::Amp   => { self.advance(); let m = self.eat(&TokenKind::Mut); Ok(Expr::Ref(m, Box::new(self.parse_unary_expr()?), s)) }
            _ => self.parse_postfix_expr(),
        }
    }
    fn parse_postfix_expr(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.parse_primary_expr()?;
        loop {
            let s = self.current_span();
            match self.peek() {
                TokenKind::Dot => {
                    self.advance(); let field = self.expect_ident()?;
                    if matches!(self.peek(), TokenKind::LParen) { self.advance(); let args = self.parse_call_args()?; expr = Expr::MethodCall(Box::new(expr), field, args, s); }
                    else { expr = Expr::Field(Box::new(expr), field, s); }
                }
                TokenKind::LParen   => { self.advance(); let args = self.parse_call_args()?; expr = Expr::Call(Box::new(expr), args, s); }
                TokenKind::LBracket => { self.advance(); let idx = self.parse_expr()?; self.expect(&TokenKind::RBracket)?; expr = Expr::Index(Box::new(expr), Box::new(idx), s); }
                _ => break,
            }
        }
        Ok(expr)
    }
    fn parse_call_args(&mut self) -> Result<Vec<Expr>, ParseError> {
        let mut args = Vec::new();
        while !matches!(self.peek(), TokenKind::RParen | TokenKind::Eof) { args.push(self.parse_expr()?); if !self.eat(&TokenKind::Comma) { break; } }
        self.expect(&TokenKind::RParen)?;
        Ok(args)
    }
    fn parse_primary_expr(&mut self) -> Result<Expr, ParseError> {
        let s = self.current_span();
        match self.peek().clone() {
            TokenKind::IntLit(n)    => { self.advance(); Ok(Expr::Lit(Lit::Int(n), s)) }
            TokenKind::FloatLit(f)  => { self.advance(); Ok(Expr::Lit(Lit::Float(f), s)) }
            TokenKind::StringLit(v) => { self.advance(); Ok(Expr::Lit(Lit::Str(v), s)) }
            TokenKind::CharLit(c)   => { self.advance(); Ok(Expr::Lit(Lit::Char(c), s)) }
            TokenKind::BoolLit(b)   => { self.advance(); Ok(Expr::Lit(Lit::Bool(b), s)) }
            TokenKind::SelfVal      => { self.advance(); Ok(Expr::Ident(Ident { name: "self".into(), span: s })) }
            TokenKind::Return => {
                self.advance();
                let val = if !matches!(self.peek(), TokenKind::Semi | TokenKind::RBrace | TokenKind::Eof) { Some(Box::new(self.parse_expr()?)) } else { None };
                Ok(Expr::Return(val, s))
            }
            TokenKind::Break => {
                self.advance();
                let val = if !matches!(self.peek(), TokenKind::Semi | TokenKind::RBrace | TokenKind::Eof) { Some(Box::new(self.parse_expr()?)) } else { None };
                Ok(Expr::Break(val, s))
            }
            TokenKind::Continue => { self.advance(); Ok(Expr::Continue(s)) }
            TokenKind::If    => self.parse_if_expr(),
            TokenKind::While => self.parse_while_expr(),
            TokenKind::Loop  => { self.advance(); Ok(Expr::Loop(Box::new(self.parse_block_expr()?), s)) }
            TokenKind::For   => self.parse_for_expr(),
            TokenKind::Match => self.parse_match_expr(),
            TokenKind::LBrace => self.parse_block_expr(),
            TokenKind::LParen => {
                self.advance();
                if self.eat(&TokenKind::RParen) { return Ok(Expr::Tuple(vec![], s)); }
                let first = self.parse_expr()?;
                if self.eat(&TokenKind::RParen) { return Ok(first); }
                let mut exprs = vec![first];
                while self.eat(&TokenKind::Comma) { if matches!(self.peek(), TokenKind::RParen) { break; } exprs.push(self.parse_expr()?); }
                self.expect(&TokenKind::RParen)?;
                Ok(Expr::Tuple(exprs, s))
            }
            TokenKind::LBracket => {
                self.advance();
                let mut elems = Vec::new();
                while !matches!(self.peek(), TokenKind::RBracket | TokenKind::Eof) { elems.push(self.parse_expr()?); if !self.eat(&TokenKind::Comma) { break; } }
                self.expect(&TokenKind::RBracket)?;
                Ok(Expr::Array(elems, s))
            }
            TokenKind::Ident(_) => {
                let ident = self.expect_ident()?;
                if self.allow_struct_lit && matches!(self.peek(), TokenKind::LBrace) {
                    self.advance();
                    let mut fields = Vec::new();
                    while !matches!(self.peek(), TokenKind::RBrace | TokenKind::Eof) {
                        let fname = self.expect_ident()?; self.expect(&TokenKind::Colon)?; let fval = self.parse_expr()?;
                        fields.push((fname, fval)); if !self.eat(&TokenKind::Comma) { break; }
                    }
                    self.expect(&TokenKind::RBrace)?;
                    return Ok(Expr::Struct(ident, fields, s));
                }
                if matches!(self.peek(), TokenKind::ColonColon) {
                    let mut path = vec![ident];
                    while self.eat(&TokenKind::ColonColon) { path.push(self.expect_ident()?); }
                    return Ok(Expr::Path(path, s));
                }
                Ok(Expr::Ident(ident))
            }
            other => Err(ParseError::new(format!("expected expression, got {:?}", other), s))
        }
    }
    fn parse_if_expr(&mut self) -> Result<Expr, ParseError> {
        let s = self.current_span(); self.expect(&TokenKind::If)?;
        let cond = self.parse_expr()?; let then = self.parse_block_expr()?;
        let else_ = if self.eat(&TokenKind::Else) {
            if matches!(self.peek(), TokenKind::If) { Some(Box::new(self.parse_if_expr()?)) }
            else { Some(Box::new(self.parse_block_expr()?)) }
        } else { None };
        Ok(Expr::If(Box::new(cond), Box::new(then), else_, s))
    }
    fn parse_while_expr(&mut self) -> Result<Expr, ParseError> {
        let s = self.current_span(); self.expect(&TokenKind::While)?;
        Ok(Expr::While(Box::new(self.parse_expr()?), Box::new(self.parse_block_expr()?), s))
    }
    fn parse_for_expr(&mut self) -> Result<Expr, ParseError> {
        let s = self.current_span(); self.expect(&TokenKind::For)?;
        let pat = self.parse_pat()?; self.expect(&TokenKind::In)?;
        Ok(Expr::For(pat, Box::new(self.parse_expr()?), Box::new(self.parse_block_expr()?), s))
    }
    // parse_expr_no_struct: like parse_expr but disallows struct literals
    // Used for match scrutinees and if conditions to avoid { ambiguity
    fn parse_expr_no_struct(&mut self) -> Result<Expr, ParseError> {
        let prev = self.allow_struct_lit;
        self.allow_struct_lit = false;
        let result = self.parse_assign_expr();
        self.allow_struct_lit = prev;
        result
    }
    fn parse_assign_expr_no_struct(&mut self) -> Result<Expr, ParseError> {
        let lhs = self.parse_range_expr_no_struct()?;
        let span = self.current_span();
        match self.peek().clone() {
            TokenKind::Eq      => { self.advance(); Ok(Expr::Assign(Box::new(lhs), Box::new(self.parse_assign_expr_no_struct()?), span)) }
            TokenKind::PlusEq  => { self.advance(); Ok(Expr::AssignOp(BinaryOp::Add, Box::new(lhs), Box::new(self.parse_assign_expr_no_struct()?), span)) }
            TokenKind::MinusEq => { self.advance(); Ok(Expr::AssignOp(BinaryOp::Sub, Box::new(lhs), Box::new(self.parse_assign_expr_no_struct()?), span)) }
            TokenKind::StarEq  => { self.advance(); Ok(Expr::AssignOp(BinaryOp::Mul, Box::new(lhs), Box::new(self.parse_assign_expr_no_struct()?), span)) }
            TokenKind::SlashEq => { self.advance(); Ok(Expr::AssignOp(BinaryOp::Div, Box::new(lhs), Box::new(self.parse_assign_expr_no_struct()?), span)) }
            _ => Ok(lhs)
        }
    }
    fn parse_range_expr_no_struct(&mut self) -> Result<Expr, ParseError> {
        let span = self.current_span();
        let lhs = self.parse_or_expr_no_struct()?;
        if matches!(self.peek(), TokenKind::DotDot | TokenKind::DotDotEq) {
            let inc = matches!(self.peek(), TokenKind::DotDotEq); self.advance();
            let rhs = if !matches!(self.peek(), TokenKind::Semi | TokenKind::RBracket | TokenKind::RParen | TokenKind::Eof) { Some(Box::new(self.parse_or_expr_no_struct()?)) } else { None };
            Ok(Expr::Range(Some(Box::new(lhs)), rhs, inc, span))
        } else { Ok(lhs) }
    }
    fn parse_or_expr_no_struct(&mut self) -> Result<Expr, ParseError> {
        let mut lhs = self.parse_and_expr_no_struct()?;
        while matches!(self.peek(), TokenKind::PipePipe) { let s = self.current_span(); self.advance(); lhs = Expr::Binary(BinaryOp::Or, Box::new(lhs), Box::new(self.parse_and_expr_no_struct()?), s); }
        Ok(lhs)
    }
    fn parse_and_expr_no_struct(&mut self) -> Result<Expr, ParseError> {
        let mut lhs = self.parse_cmp_expr_no_struct()?;
        while matches!(self.peek(), TokenKind::AmpAmp) { let s = self.current_span(); self.advance(); lhs = Expr::Binary(BinaryOp::And, Box::new(lhs), Box::new(self.parse_cmp_expr_no_struct()?), s); }
        Ok(lhs)
    }
    fn parse_cmp_expr_no_struct(&mut self) -> Result<Expr, ParseError> {
        let mut lhs = self.parse_add_expr()?;
        loop {
            let s = self.current_span();
            let op = match self.peek() { TokenKind::EqEq=>BinaryOp::Eq, TokenKind::BangEq=>BinaryOp::Ne, TokenKind::Lt=>BinaryOp::Lt, TokenKind::LtEq=>BinaryOp::Le, TokenKind::Gt=>BinaryOp::Gt, TokenKind::GtEq=>BinaryOp::Ge, _=>break };
            self.advance(); lhs = Expr::Binary(op, Box::new(lhs), Box::new(self.parse_add_expr()?), s);
        }
        Ok(lhs)
    }
    fn parse_primary_expr_no_struct(&mut self) -> Result<Expr, ParseError> {
        let s = self.current_span();
        // Same as parse_primary_expr but Ident never enters struct literal path
        match self.peek().clone() {
            TokenKind::Ident(_) => {
                let ident = self.expect_ident()?;
                if matches!(self.peek(), TokenKind::ColonColon) {
                    let mut path = vec![ident];
                    while self.eat(&TokenKind::ColonColon) { path.push(self.expect_ident()?); }
                    return Ok(Expr::Path(path, s));
                }
                Ok(Expr::Ident(ident))
            }
            _ => self.parse_primary_expr()
        }
    }
        fn parse_match_expr(&mut self) -> Result<Expr, ParseError> {
        let s = self.current_span(); self.expect(&TokenKind::Match)?;
        let scrutinee = self.parse_expr_no_struct()?; self.expect(&TokenKind::LBrace)?;
        let mut arms = Vec::new();
        while !matches!(self.peek(), TokenKind::RBrace | TokenKind::Eof) {
            let aspan = self.current_span(); let pat = self.parse_pat()?;
            let guard = if self.eat(&TokenKind::If) { Some(self.parse_expr()?) } else { None };
            self.expect(&TokenKind::FatArrow)?; let body = self.parse_expr()?; self.eat(&TokenKind::Comma);
            arms.push(MatchArm { pat, guard, body, span: aspan });
        }
        self.expect(&TokenKind::RBrace)?;
        Ok(Expr::Match(Box::new(scrutinee), arms, s))
    }
}

pub fn parse(src: &str) -> Result<Vec<Item>, ParseError> {
    Parser::from_source(src).map_err(|e| ParseError::new(e.msg, e.span))?.parse_program()
}

#[cfg(test)]
mod axon_parser_tests {
    use super::*;
    #[test] fn tp1_simple_fn() { assert!(matches!(parse("fn add(x: i32) -> i32 { return x; }").unwrap()[0], Item::Fn(_, _))); }
    #[test] fn tp2_contracts() {
        let items = parse("@requires(x > 0) @ensures(result > 0) fn pos(x: i32) -> i32 { return x; }").unwrap();
        if let Item::Fn(sig, _) = &items[0] { assert_eq!(sig.contracts.len(), 2); assert_eq!(sig.contracts[0].kind, ContractKind::Requires); }
        else { panic!(); }
    }
    #[test] fn tp3_patchable() {
        let items = parse("#[patchable] fn update() -> () { }").unwrap();
        if let Item::Fn(sig, _) = &items[0] { assert!(sig.attrs.iter().any(|a| a.name == "patchable")); }
    }
    #[test] fn tp4_struct() {
        let items = parse("struct Point { x: i32, y: i32, }").unwrap();
        if let Item::Struct(s) = &items[0] { assert_eq!(s.fields.len(), 2); } else { panic!(); }
    }
    #[test] fn tp5_enum() {
        let items = parse("enum Color { Red, Green, Blue, }").unwrap();
        if let Item::Enum(e) = &items[0] { assert_eq!(e.variants.len(), 3); } else { panic!(); }
    }
    #[test] fn tp6_impl() {
        let items = parse("impl Point { fn new(x: i32) -> Point { return x; } }").unwrap();
        if let Item::Impl(i) = &items[0] { assert_eq!(i.items.len(), 1); } else { panic!(); }
    }
    #[test] fn tp7_trait() { assert!(parse("trait Shape { fn area(&self) -> f64 { } }").is_ok()); }
    #[test] fn tp8_if_else() { assert!(parse("fn f(x: i32) -> i32 { if x > 0 { return x; } else { return 0; } }").is_ok()); }
    #[test] fn tp9_while() { assert!(parse("fn f() -> () { while true { } }").is_ok()); }
    #[test] fn tp10_let() {
        let items = parse("fn f() -> () { let x: i32 = 42; }").unwrap();
        if let Item::Fn(_, body) = &items[0] { if let Expr::Block(stmts, _, _) = body { assert!(matches!(stmts[0], Stmt::Let(_, _, _, _))); } }
    }
    #[test] fn tp11_binary() { assert!(parse("fn f() -> () { let z = 1 + 2 * 3; }").is_ok()); }
    #[test] fn tp12_match() { assert!(parse("fn f(x: i32) -> () { match x { 0 => return 0, _ => return 1, } }").is_ok()); }
    #[test] fn tp13_profile() { assert!(matches!(parse("profile seL4Strict { }").unwrap()[0], Item::Profile(_))); }
    #[test] fn tp14_type_alias() { assert!(matches!(parse("type MyInt = i32;").unwrap()[0], Item::TypeAlias(_, _, _, _))); }
    #[test] fn tp15_use() { assert!(matches!(parse("use std::vec;").unwrap()[0], Item::Use(_, _))); }
    #[test] fn tp16_const() { assert!(matches!(parse("const MAX: i32 = 100;").unwrap()[0], Item::Const(_, _, _, _))); }
    #[test] fn tp17_method_call() { assert!(parse("fn f() -> () { let r = v.push(1); }").is_ok()); }
    #[test] fn tp18_field_access() { assert!(parse("fn f() -> () { let x = p.x; }").is_ok()); }
    #[test] fn tp19_generic_fn() {
        let items = parse("fn identity<T>(x: T) -> T { return x; }").unwrap();
        if let Item::Fn(sig, _) = &items[0] { assert_eq!(sig.generics.len(), 1); }
    }
    #[test] fn tp20_error_on_malformed() { assert!(parse("fn { }").is_err()); }

    #[test]
    fn tp12_debug() {
        let result = parse("fn f(x: i32) -> () { match x { 0 => return 0, _ => return 1, } }");
        match result {
            Ok(_) => println!("OK"),
            Err(e) => println!("ERROR: {}", e),
        }
    }

}