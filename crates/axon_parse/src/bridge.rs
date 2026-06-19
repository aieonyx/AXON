// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
//
// Rust parser bridge — mirrors parse.ax and token_stream.ax exactly.
// Excised at P55 bootstrap when axonc compiles parse.ax natively.

use axon_lex::Token;
use axon_std_string::AxString;
use crate::ast::*;
use crate::error::{ParseError, ParseResult};

const MAX_DEPTH: usize = 128;

// ── Token stream cursor — mirrors token_stream.ax ─────────────────────────────

pub struct TokenStream {
    tokens: Vec<Token>,
    pos:    usize,
    depth:  usize,
}

impl TokenStream {
    pub fn new(tokens: Vec<Token>) -> Self {
        // Filter trivia — parser never sees whitespace or comments
        let tokens: Vec<Token> = tokens
            .into_iter()
            .filter(|t| !t.is_trivia())
            .collect();
        TokenStream { tokens, pos: 0, depth: 0 }
    }

    pub fn peek(&self) -> &Token {
        self.tokens.get(self.pos).unwrap_or(&Token::Eof)
    }

    pub fn advance(&mut self) -> &Token {
        let tok = self.tokens.get(self.pos).unwrap_or(&Token::Eof);
        if self.pos < self.tokens.len() {
            self.pos += 1;
        }
        tok
    }

    pub fn expect(&mut self, expected: &Token) -> ParseResult<()> {
        if self.peek() == expected {
            self.advance();
            Ok(())
        } else {
            Err(ParseError::UnexpectedToken(
                AxString::ax_from_str(&format!("expected {:?}, got {:?}", expected, self.peek()))
            ))
        }
    }

    pub fn expect_ident(&mut self) -> ParseResult<AxString> {
        match self.peek().clone() {
            Token::Ident(name) => { self.advance(); Ok(name) }
            other => Err(ParseError::UnexpectedToken(
                AxString::ax_from_str(&format!("expected identifier, got {:?}", other))
            ))
        }
    }

    pub fn is_eof(&self) -> bool {
        matches!(self.peek(), Token::Eof)
    }

    fn enter(&mut self) -> ParseResult<()> {
        self.depth += 1;
        if self.depth > MAX_DEPTH {
            Err(ParseError::MaxDepthExceeded)
        } else {
            Ok(())
        }
    }

    fn exit(&mut self) {
        if self.depth > 0 { self.depth -= 1; }
    }
}

// ── Public entry point ────────────────────────────────────────────────────────

pub fn parse(source: &str) -> ParseResult<Program> {
    let tokens = axon_lex::lex_all(source)
        .map_err(|e| ParseError::UnexpectedToken(
            AxString::ax_from_str(&format!("lex error: {}", e))
        ))?;
    let mut stream = TokenStream::new(tokens);
    parse_program(&mut stream)
}

// ── Grammar rules — each mirrors a function in parse.ax ──────────────────────

fn parse_program(s: &mut TokenStream) -> ParseResult<Program> {
    let mut items = Vec::new();
    while !s.is_eof() {
        items.push(parse_item(s)?);
    }
    Ok(Program { items })
}

fn parse_item(s: &mut TokenStream) -> ParseResult<Item> {
    match s.peek() {
        Token::Fn     => parse_fn(s),
        Token::Struct => parse_struct(s),
        other => Err(ParseError::UnexpectedToken(
            AxString::ax_from_str(&format!("expected item, got {:?}", other))
        )),
    }
}

fn parse_fn(s: &mut TokenStream) -> ParseResult<Item> {
    s.advance(); // consume fn
    let name = s.expect_ident()?;
    s.expect(&Token::LParen)?;
    let params = parse_params(s)?;
    s.expect(&Token::RParen)?;
    s.expect(&Token::Arrow)?;
    let ret = parse_type_expr(s)?;
    let body = parse_block_stmts(s)?;
    Ok(Item::Fn { name, params, ret, body })
}

fn parse_params(s: &mut TokenStream) -> ParseResult<Vec<Param>> {
    let mut params = Vec::new();
    if matches!(s.peek(), Token::RParen) {
        return Ok(params);
    }
    params.push(parse_param(s)?);
    while matches!(s.peek(), Token::Comma) {
        s.advance();
        if matches!(s.peek(), Token::RParen) { break; }
        params.push(parse_param(s)?);
    }
    Ok(params)
}

fn parse_param(s: &mut TokenStream) -> ParseResult<Param> {
    let name = s.expect_ident()?;
    s.expect(&Token::Colon)?;
    let ty = parse_type_expr(s)?;
    Ok(Param { name, ty })
}

fn parse_struct(s: &mut TokenStream) -> ParseResult<Item> {
    s.advance(); // consume struct
    let name = s.expect_ident()?;
    s.expect(&Token::LBrace)?;
    let fields = parse_fields(s)?;
    s.expect(&Token::RBrace)?;
    Ok(Item::Struct { name, fields })
}

fn parse_fields(s: &mut TokenStream) -> ParseResult<Vec<Field>> {
    let mut fields = Vec::new();
    while !matches!(s.peek(), Token::RBrace | Token::Eof) {
        let name = s.expect_ident()?;
        s.expect(&Token::Colon)?;
        let ty = parse_type_expr(s)?;
        fields.push(Field { name, ty });
        if matches!(s.peek(), Token::Comma) { s.advance(); }
    }
    Ok(fields)
}

fn parse_type_expr(s: &mut TokenStream) -> ParseResult<TypeExpr> {
    let name = s.expect_ident()?;
    Ok(TypeExpr { name })
}

fn parse_block_stmts(s: &mut TokenStream) -> ParseResult<Vec<Stmt>> {
    s.expect(&Token::LBrace)?;
    let mut stmts = Vec::new();
    while !matches!(s.peek(), Token::RBrace | Token::Eof) {
        stmts.push(parse_stmt(s)?);
    }
    s.expect(&Token::RBrace)?;
    Ok(stmts)
}

fn parse_stmt(s: &mut TokenStream) -> ParseResult<Stmt> {
    match s.peek() {
        Token::Let    => parse_let(s),
        Token::Return => parse_return(s),
        _             => parse_expr_stmt(s),
    }
}

fn parse_let(s: &mut TokenStream) -> ParseResult<Stmt> {
    s.advance(); // consume let
    let mutable = if matches!(s.peek(), Token::Mut) {
        s.advance(); true
    } else { false };
    let name = s.expect_ident()?;
    // Skip optional type annotation: ": type"
    if matches!(s.peek(), Token::Colon) {
        s.advance();
        parse_type_expr(s)?;
    }
    s.expect(&Token::Eq)?;
    let value = parse_expr(s)?;
    s.expect(&Token::Semi)?;
    Ok(Stmt::Let { name, mutable, value })
}

fn parse_return(s: &mut TokenStream) -> ParseResult<Stmt> {
    s.advance(); // consume return
    let expr = parse_expr(s)?;
    s.expect(&Token::Semi)?;
    Ok(Stmt::Return(expr))
}

fn parse_expr_stmt(s: &mut TokenStream) -> ParseResult<Stmt> {
    let expr = parse_expr(s)?;
    // Block-like expressions (if, block) do not require a trailing semicolon
    let needs_semi = !matches!(expr, Expr::If { .. } | Expr::Block(_));
    if needs_semi {
        s.expect(&Token::Semi)?;
    }
    Ok(Stmt::ExprStmt(expr))
}

// ── Expression parsing — precedence climbing ──────────────────────────────────

fn parse_expr(s: &mut TokenStream) -> ParseResult<Expr> {
    s.enter()?;
    let result = parse_equality(s);
    s.exit();
    result
}

fn parse_equality(s: &mut TokenStream) -> ParseResult<Expr> {
    let mut lhs = parse_comparison(s)?;
    loop {
        let op = match s.peek() {
            Token::EqEq  => BinOpKind::Eq,
            Token::BangEq => BinOpKind::Ne,
            _ => break,
        };
        s.advance();
        let rhs = parse_comparison(s)?;
        lhs = Expr::BinOp { op, lhs: Box::new(lhs), rhs: Box::new(rhs) };
    }
    Ok(lhs)
}

fn parse_comparison(s: &mut TokenStream) -> ParseResult<Expr> {
    let mut lhs = parse_term(s)?;
    loop {
        let op = match s.peek() {
            Token::Lt   => BinOpKind::Lt,
            Token::LtEq => BinOpKind::Le,
            Token::Gt   => BinOpKind::Gt,
            Token::GtEq => BinOpKind::Ge,
            _ => break,
        };
        s.advance();
        let rhs = parse_term(s)?;
        lhs = Expr::BinOp { op, lhs: Box::new(lhs), rhs: Box::new(rhs) };
    }
    Ok(lhs)
}

fn parse_term(s: &mut TokenStream) -> ParseResult<Expr> {
    let mut lhs = parse_factor(s)?;
    loop {
        let op = match s.peek() {
            Token::Plus  => BinOpKind::Add,
            Token::Minus => BinOpKind::Sub,
            _ => break,
        };
        s.advance();
        let rhs = parse_factor(s)?;
        lhs = Expr::BinOp { op, lhs: Box::new(lhs), rhs: Box::new(rhs) };
    }
    Ok(lhs)
}

fn parse_factor(s: &mut TokenStream) -> ParseResult<Expr> {
    let mut lhs = parse_unary(s)?;
    loop {
        let op = match s.peek() {
            Token::Star    => BinOpKind::Mul,
            Token::Slash   => BinOpKind::Div,
            Token::Percent => BinOpKind::Mod,
            _ => break,
        };
        s.advance();
        let rhs = parse_unary(s)?;
        lhs = Expr::BinOp { op, lhs: Box::new(lhs), rhs: Box::new(rhs) };
    }
    Ok(lhs)
}

fn parse_unary(s: &mut TokenStream) -> ParseResult<Expr> {
    match s.peek() {
        Token::Bang  => { s.advance(); Ok(Expr::UnaryOp { op: UnaryOpKind::Not, expr: Box::new(parse_unary(s)?) }) }
        Token::Minus => { s.advance(); Ok(Expr::UnaryOp { op: UnaryOpKind::Neg, expr: Box::new(parse_unary(s)?) }) }
        _ => parse_call(s),
    }
}

fn parse_call(s: &mut TokenStream) -> ParseResult<Expr> {
    let primary = parse_primary(s)?;
    // Check if this is a function call: Ident followed by "("
    if let Expr::Ident(ref name) = primary {
        if matches!(s.peek(), Token::LParen) {
            let name = name.clone();
            s.advance(); // consume "("
            let args = parse_args(s)?;
            s.expect(&Token::RParen)?;
            return Ok(Expr::Call { name, args });
        }
    }
    Ok(primary)
}

fn parse_args(s: &mut TokenStream) -> ParseResult<Vec<Expr>> {
    let mut args = Vec::new();
    if matches!(s.peek(), Token::RParen) {
        return Ok(args);
    }
    args.push(parse_expr(s)?);
    while matches!(s.peek(), Token::Comma) {
        s.advance();
        if matches!(s.peek(), Token::RParen) { break; }
        args.push(parse_expr(s)?);
    }
    Ok(args)
}

fn parse_if(s: &mut TokenStream) -> ParseResult<Expr> {
    s.advance(); // consume if
    let cond = parse_expr(s)?;
    let then_stmts = parse_block_stmts(s)?;
    let then = Expr::Block(then_stmts);
    let else_ = if matches!(s.peek(), Token::Else) {
        s.advance();
        let else_stmts = parse_block_stmts(s)?;
        Some(Box::new(Expr::Block(else_stmts)))
    } else {
        None
    };
    Ok(Expr::If {
        cond: Box::new(cond),
        then: Box::new(then),
        else_,
    })
}

fn parse_primary(s: &mut TokenStream) -> ParseResult<Expr> {
    match s.peek().clone() {
        Token::IntLit(n)       => { s.advance(); Ok(Expr::IntLit(n)) }
        Token::FloatLit(f)     => { s.advance(); Ok(Expr::FloatLit(f)) }
        Token::StringLit(st)   => { s.advance(); Ok(Expr::StringLit(st)) }
        Token::BoolLit(b)      => { s.advance(); Ok(Expr::BoolLit(b)) }
        Token::Nil             => { s.advance(); Ok(Expr::Nil) }
        Token::Ident(name)     => { s.advance(); Ok(Expr::Ident(name)) }
        Token::If              => parse_if(s),
        Token::LParen          => {
            s.advance();
            let expr = parse_expr(s)?;
            s.expect(&Token::RParen)?;
            Ok(expr)
        }
        other => Err(ParseError::UnexpectedToken(
            AxString::ax_from_str(&format!("expected expression, got {:?}", other))
        )),
    }
}
