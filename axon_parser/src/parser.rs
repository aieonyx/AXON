// ============================================================
// AXON Parser — parser.rs
// P2-10 complete — full statement parsing
// Copyright © 2026 Edison Lepiten — AIEONYX
// github.com/aieonyx/axon
// ============================================================

use axon_lexer::{FileId, Span, Token, TokenKind};
use crate::ast::{
    Program, TopLevelItem,
    ModuleDecl, ImportDecl,
    StructDecl, FieldDecl,
    EnumDecl, VariantDecl,
    FnDecl, TaskDecl, Param,
    ProgramIntent,
    Type, PrimitiveType,
    Ident, Block, Stmt,
    LetStmt, MutStmt, EphemeralStmt,
    ReturnStmt, ExprStmt,
    IfStmt, ForStmt, WhileStmt,
    DeferStmt, WithStmt,
    Expr, Literal,
    Decorator, DecoratorArg,
    UsesList, EffectName,
    MemMode, AssignStmt, AssignTarget, AssignOp,
    FieldAccessExpr,
};
use crate::error::ParseError;

// ── Parser struct ─────────────────────────────────────────────

pub struct Parser<'src> {
    tokens  : Vec<Token>,
    pos     : usize,
    source  : &'src str,
    file_id : FileId,
    pub errors : Vec<ParseError>,
}

impl<'src> Parser<'src> {
    pub fn new(tokens: Vec<Token>, source: &'src str, file_id: FileId) -> Self {
        Parser { tokens, pos: 0, source, file_id, errors: Vec::new() }
    }

    pub fn into_errors(self) -> Vec<ParseError> { self.errors }

    fn merge(a: Span, b: Span) -> Span {
        Span::new(a.file, a.start.min(b.start), a.end.max(b.end),
                  a.line.min(b.line), a.col.min(b.col))
    }

    // ── Cursor methods ────────────────────────────────────────

    pub fn peek(&self) -> &Token {
        let mut i = self.pos;
        while i < self.tokens.len() {
            match &self.tokens[i].kind {
                TokenKind::Comment(_) | TokenKind::DocComment(_) => i += 1,
                _ => return &self.tokens[i],
            }
        }
        self.tokens.last().expect("token stream must end with Eof")
    }

    pub fn peek_kind(&self) -> &TokenKind { &self.peek().kind }
    pub fn current_span(&self) -> Span    { self.peek().span  }

    pub fn advance(&mut self) -> Token {
        while self.pos < self.tokens.len() {
            match &self.tokens[self.pos].kind {
                TokenKind::Comment(_) | TokenKind::DocComment(_) => { self.pos += 1; }
                _ => break,
            }
        }
        if self.pos < self.tokens.len() {
            let tok = self.tokens[self.pos].clone();
            self.pos += 1;
            tok
        } else {
            self.tokens.last().cloned().expect("token stream must end with Eof")
        }
    }

    pub fn eat(&mut self, kind: &TokenKind) -> Option<Token> {
        if self.at(kind) { Some(self.advance()) } else { None }
    }

    pub fn expect(&mut self, kind: &TokenKind) -> Option<Token> {
        if self.at(kind) {
            Some(self.advance())
        } else {
            let span     = self.current_span();
            let found    = self.peek_kind().display_name().to_string();
            let expected = kind.display_name().to_string();
            self.error(span, format!("expected {} but found {}", expected, found));
            None
        }
    }

    pub fn at(&self, kind: &TokenKind) -> bool {
        use std::mem::discriminant;
        match (self.peek_kind(), kind) {
            (TokenKind::Ident(_),    TokenKind::Ident(_))    => true,
            (TokenKind::IntLit(_),   TokenKind::IntLit(_))   => true,
            (TokenKind::FloatLit(_), TokenKind::FloatLit(_)) => true,
            (TokenKind::StrLit(_),   TokenKind::StrLit(_))   => true,
            (TokenKind::BoolLit(_),  TokenKind::BoolLit(_))  => true,
            (TokenKind::Error(_),    TokenKind::Error(_))    => true,
            (a, b) => discriminant(a) == discriminant(b),
        }
    }

    pub fn at_eof(&self) -> bool { matches!(self.peek_kind(), TokenKind::Eof) }

    pub fn skip_newlines(&mut self) {
        while matches!(self.peek_kind(), TokenKind::Newline) { self.advance(); }
    }

    pub fn skip_whitespace_tokens(&mut self) {
        loop {
            match self.peek_kind() {
                TokenKind::Newline | TokenKind::Indent | TokenKind::Dedent => { self.advance(); }
                _ => break,
            }
        }
    }

    pub fn error(&mut self, span: Span, message: impl Into<String>) {
        if self.errors.len() < 20 {
            self.errors.push(ParseError::Custom {
                message: message.into(), span, hint: None,
            });
        }
    }

    pub fn error_and_recover(&mut self, span: Span, message: impl Into<String>) {
        self.error(span, message);
        self.synchronize();
    }

    fn synchronize(&mut self) {
        while !self.at_eof() {
            match self.peek_kind() {
                TokenKind::Fn | TokenKind::Task | TokenKind::Struct |
                TokenKind::Enum | TokenKind::Impl | TokenKind::Trait |
                TokenKind::Actor | TokenKind::Opaque |
                TokenKind::Module | TokenKind::Import |
                TokenKind::Dedent => return,
                _ => { self.advance(); }
            }
        }
    }

    // ── Module / Import ───────────────────────────────────────

    fn parse_program_intent(&mut self) -> Option<ProgramIntent> {
        if !matches!(self.peek_kind(), TokenKind::ProgramIntentDecl) { return None; }
        let span = self.current_span();
        self.advance();
        self.skip_newlines();
        let description = match self.peek_kind().clone() {
            TokenKind::StrLit(s) => { self.advance(); s }
            _ => {
                self.error(self.current_span(),
                    "@program_intent must be followed by a string");
                return None;
            }
        };
        self.skip_newlines();
        Some(ProgramIntent { span, description, constraints: vec![] })
    }

    fn parse_module_decl(&mut self) -> Option<ModuleDecl> {
        if !self.at(&TokenKind::Module) { return None; }
        let span = self.current_span();
        self.advance();
        let path = self.parse_dotted_path();
        if path.is_empty() {
            self.error(self.current_span(), "expected module path after 'module'");
            return None;
        }
        self.eat(&TokenKind::Newline);
        Some(ModuleDecl { span, path })
    }

    fn parse_import_decl(&mut self) -> Option<ImportDecl> {
        let span = self.current_span();
        self.expect(&TokenKind::Import)?;
        let path = self.parse_dotted_path();
        if path.is_empty() {
            self.error(self.current_span(), "expected path after 'import'");
            return None;
        }
        let alias = if self.at(&TokenKind::As) {
            self.advance();
            match self.peek_kind().clone() {
                TokenKind::Ident(name) => {
                    let s = self.current_span(); self.advance();
                    Some(Ident::new(name, s))
                }
                _ => {
                    self.error(self.current_span(), "expected identifier after 'as'");
                    None
                }
            }
        } else { None };
        self.eat(&TokenKind::Newline);
        Some(ImportDecl { span, path, alias })
    }

    pub fn parse_dotted_path(&mut self) -> Vec<Ident> {
        let mut parts = Vec::new();
        match self.peek_kind().clone() {
            TokenKind::Ident(name) => {
                let s = self.current_span(); self.advance();
                parts.push(Ident::new(name, s));
            }
            _ => return parts,
        }
        while self.at(&TokenKind::Dot) {
            self.advance();
            match self.peek_kind().clone() {
                TokenKind::Ident(name) => {
                    let s = self.current_span(); self.advance();
                    parts.push(Ident::new(name, s));
                }
                _ => {
                    self.error(self.current_span(), "expected identifier after '.'");
                    break;
                }
            }
        }
        parts
    }

    // ── Struct / Enum ─────────────────────────────────────────

    fn parse_struct_decl(&mut self) -> Option<StructDecl> {
        let span     = self.current_span();
        self.expect(&TokenKind::Struct)?;
        let name     = self.parse_ident("struct name")?;
        let generics = vec![];
        self.expect(&TokenKind::Colon)?;
        self.eat(&TokenKind::Newline);
        self.eat(&TokenKind::Indent);
        let mut fields = Vec::new();
        while !self.at_eof() && !self.at(&TokenKind::Dedent) {
            self.skip_newlines();
            if self.at(&TokenKind::Dedent) { break; }
            if matches!(self.peek_kind(),
                TokenKind::Fn | TokenKind::Struct | TokenKind::Enum |
                TokenKind::Impl | TokenKind::Eof) { break; }
            if let Some(f) = self.parse_field_decl() { fields.push(f); }
            self.eat(&TokenKind::Newline);
        }
        self.eat(&TokenKind::Dedent);
        Some(StructDecl { span, name, generics, fields })
    }

    fn parse_field_decl(&mut self) -> Option<FieldDecl> {
        let span = self.current_span();
        let name = self.parse_ident("field name")?;
        self.expect(&TokenKind::Colon)?;
        let ty = self.parse_type()?;
        Some(FieldDecl { span, name, ty })
    }

    fn parse_enum_decl(&mut self) -> Option<EnumDecl> {
        let span     = self.current_span();
        self.expect(&TokenKind::Enum)?;
        let name     = self.parse_ident("enum name")?;
        let generics = vec![];
        self.expect(&TokenKind::Colon)?;
        self.eat(&TokenKind::Newline);
        self.eat(&TokenKind::Indent);
        let mut variants = Vec::new();
        while !self.at_eof() && !self.at(&TokenKind::Dedent) {
            self.skip_newlines();
            if self.at(&TokenKind::Dedent) { break; }
            let vspan = self.current_span();
            let vname = match self.parse_ident("variant name") {
                Some(n) => n, None => { self.advance(); continue; }
            };
            let mut fields = Vec::new();
            if self.at(&TokenKind::LParen) {
                self.advance();
                while !self.at_eof() && !self.at(&TokenKind::RParen) {
                    let fspan = self.current_span();
                    let fname = match self.parse_ident("field name") {
                        Some(n) => n, None => break,
                    };
                    self.expect(&TokenKind::Colon)?;
                    let fty = self.parse_type()?;
                    fields.push(FieldDecl { span: fspan, name: fname, ty: fty });
                    self.eat(&TokenKind::Comma);
                }
                self.expect(&TokenKind::RParen)?;
            }
            variants.push(VariantDecl { span: vspan, name: vname, fields });
            self.eat(&TokenKind::Newline);
        }
        self.eat(&TokenKind::Dedent);
        Some(EnumDecl { span, name, generics, variants })
    }

    // ── Decorators ────────────────────────────────────────────

    fn parse_decorators(&mut self) -> Vec<Decorator> {
        let mut decorators = Vec::new();
        while self.at(&TokenKind::At) {
            let span = self.current_span();
            self.advance();
            let mut name_parts = Vec::new();
            match self.peek_kind().clone() {
                TokenKind::Ident(n) => {
                    let s = self.current_span(); self.advance();
                    name_parts.push(Ident::new(n, s));
                }
                _ => { self.error(self.current_span(), "expected decorator name"); break; }
            }
            while self.at(&TokenKind::Dot) {
                self.advance();
                match self.peek_kind().clone() {
                    TokenKind::Ident(n) => {
                        let s = self.current_span(); self.advance();
                        name_parts.push(Ident::new(n, s));
                    }
                    _ => break,
                }
            }
            let mut args = Vec::new();
            if self.at(&TokenKind::LParen) {
                self.advance();
                while !self.at_eof() && !self.at(&TokenKind::RParen) {
                    let aspan = self.current_span();
                    let label = if matches!(self.peek_kind(), TokenKind::Ident(_)) {
                        let saved = self.pos;
                        let tok   = self.advance();
                        if self.at(&TokenKind::Colon) {
                            self.advance();
                            if let TokenKind::Ident(n) = tok.kind {
                                Some(Ident::new(n, tok.span))
                            } else { None }
                        } else { self.pos = saved; None }
                    } else { None };
                    let value = match self.peek_kind().clone() {
                        TokenKind::StrLit(s)  => { let vs=self.current_span(); self.advance(); Expr::Lit(Literal::Str(s,vs)) }
                        TokenKind::BoolLit(b) => { let vs=self.current_span(); self.advance(); Expr::Lit(Literal::Bool(b,vs)) }
                        TokenKind::IntLit(n)  => { let vs=self.current_span(); self.advance(); Expr::Lit(Literal::Int(n,vs)) }
                        _ => { let vs=self.current_span(); self.advance(); Expr::Lit(Literal::None(vs)) }
                    };
                    args.push(DecoratorArg { span: aspan, label, value });
                    self.eat(&TokenKind::Comma);
                }
                self.eat(&TokenKind::RParen);
            }
            decorators.push(Decorator { span, name: name_parts, args });
            self.eat(&TokenKind::Newline);
            self.skip_newlines();
        }
        decorators
    }

    // ── Fn / Task declarations ────────────────────────────────

    fn parse_fn_decl(&mut self, decorators: Vec<Decorator>) -> Option<FnDecl> {
        let span     = self.current_span();
        self.expect(&TokenKind::Fn)?;
        let name     = self.parse_ident("function name")?;
        let generics = vec![];
        let params = if self.at(&TokenKind::LParen) {
            self.advance();
            let p = self.parse_params();
            self.expect(&TokenKind::RParen)?;
            p
        } else { vec![] };
        let ret_type = if self.at(&TokenKind::Arrow) {
            self.advance(); self.parse_type()
        } else { None };
        let uses = self.parse_uses_clause();
        self.expect(&TokenKind::Colon)?;
        let body = self.parse_block();
        Some(FnDecl { span, decorators, name, generics, params, uses, ret_type, body })
    }

    fn parse_task_decl(&mut self, decorators: Vec<Decorator>) -> Option<TaskDecl> {
        let span     = self.current_span();
        self.expect(&TokenKind::Task)?;
        let name     = self.parse_ident("task name")?;
        let generics = vec![];
        let params = if self.at(&TokenKind::LParen) {
            self.advance();
            let p = self.parse_params();
            self.expect(&TokenKind::RParen)?;
            p
        } else { vec![] };
        let ret_type = if self.at(&TokenKind::Arrow) {
            self.advance(); self.parse_type()
        } else { None };
        let uses = self.parse_uses_clause();
        self.expect(&TokenKind::Colon)?;
        let body = self.parse_block();
        Some(TaskDecl { span, decorators, name, generics, params, uses, ret_type, body })
    }

    fn parse_params(&mut self) -> Vec<Param> {
        let mut params = Vec::new();
        while !self.at_eof() && !self.at(&TokenKind::RParen) {
            if let Some(p) = self.parse_param() { params.push(p); }
            else { break; }
            if self.eat(&TokenKind::Comma).is_none() { break; }
        }
        params
    }

    fn parse_param(&mut self) -> Option<Param> {
        let span = self.current_span();
        let mem_mode = match self.peek_kind() {
            TokenKind::Own       => { self.advance(); Some(MemMode::Own)      }
            TokenKind::Borrow    => { self.advance(); Some(MemMode::Borrow)   }
            TokenKind::MutBorrow => { self.advance(); Some(MemMode::MutBorrow)}
            TokenKind::Share     => { self.advance(); Some(MemMode::Share)    }
            _ => None,
        };
        let name = self.parse_ident("parameter name")?;
        self.expect(&TokenKind::Colon)?;
        let ty = self.parse_type()?;
        Some(Param { span, mem_mode, name, ty })
    }

    fn parse_uses_clause(&mut self) -> Option<UsesList> {
        if !self.at(&TokenKind::Uses) { return None; }
        let span = self.current_span();
        self.advance();
        let mut effects = Vec::new();
        if self.at(&TokenKind::LBracket) {
            self.advance();
            while !self.at_eof() && !self.at(&TokenKind::RBracket) {
                let espan = self.current_span();
                let parts = self.parse_dotted_path();
                if !parts.is_empty() { effects.push(EffectName { span: espan, parts }); }
                if self.eat(&TokenKind::Comma).is_none() { break; }
            }
            self.expect(&TokenKind::RBracket);
        }
        Some(UsesList { span, effects })
    }

    // ── Block and Statement parsing ───────────────────────────

    pub fn parse_block(&mut self) -> Block {
        let span = self.current_span();
        self.eat(&TokenKind::Newline);
        self.eat(&TokenKind::Indent);

        let mut stmts = Vec::new();
        while !self.at_eof() && !self.at(&TokenKind::Dedent) {
            self.skip_newlines();
            if self.at(&TokenKind::Dedent) || self.at_eof() { break; }

            if let Some(stmt) = self.parse_stmt() {
                stmts.push(stmt);
            }
        }

        self.eat(&TokenKind::Dedent);
        Block { span, stmts }
    }

    pub fn parse_stmt(&mut self) -> Option<Stmt> {
        let span = self.current_span();

        let stmt = match self.peek_kind() {

            // pass — empty statement
            TokenKind::Pass => {
                self.advance();
                self.eat(&TokenKind::Newline);
                return Some(Stmt::Pass(span));
            }

            // let name : Type = expr
            // let name = expr  (type inferred)
            TokenKind::Let => {
                self.advance();
                let name = self.parse_ident("variable name")?;
                let ty = if self.at(&TokenKind::Colon) {
                    self.advance(); self.parse_type()
                } else { None };
                self.expect(&TokenKind::Assign)?;
                let init = self.parse_expr()?;
                self.eat(&TokenKind::Newline);
                Stmt::Let(LetStmt { span, name, ty, init })
            }

            // let@ name = expr  — ephemeral binding
            TokenKind::LetAt => {
                self.advance();
                let name = self.parse_ident("ephemeral variable name")?;
                let ty = if self.at(&TokenKind::Colon) {
                    self.advance(); self.parse_type()
                } else { None };
                self.expect(&TokenKind::Assign)?;
                let init = self.parse_expr()?;
                self.eat(&TokenKind::Newline);
                Stmt::Ephemeral(EphemeralStmt { span, name, ty, init })
            }

            // mut name : Type = expr
            TokenKind::Mut => {
                self.advance();
                let name = self.parse_ident("mutable variable name")?;
                let ty = if self.at(&TokenKind::Colon) {
                    self.advance(); self.parse_type()
                } else { None };
                self.expect(&TokenKind::Assign)?;
                let init = self.parse_expr()?;
                self.eat(&TokenKind::Newline);
                Stmt::Mut(MutStmt { span, name, ty, init })
            }

            // return [expr]
            TokenKind::Return => {
                self.advance();
                let value = if !matches!(self.peek_kind(),
                    TokenKind::Newline | TokenKind::Dedent | TokenKind::Eof) {
                    self.parse_expr()
                } else { None };
                self.eat(&TokenKind::Newline);
                Stmt::Return(ReturnStmt { span, value })
            }

            // if expr: body [else: body]
            TokenKind::If => { self.parse_if_stmt()? }

            // for var in expr: body
            TokenKind::For => { self.parse_for_stmt()? }

            // while expr: body
            TokenKind::While => { self.parse_while_stmt()? }

            // defer call_expr
            TokenKind::Defer => {
                self.advance();
                let expr = self.parse_expr()?;
                self.eat(&TokenKind::Newline);
                Stmt::Defer(DeferStmt { span, expr })
            }

            // with expr as name: body
            TokenKind::With => { self.parse_with_stmt()? }

            // Expression statement or assignment
            _ => {
                let expr = self.parse_expr()?;

                // Check for assignment: expr = expr
                if self.at(&TokenKind::Assign) {
                    self.advance();
                    let value = self.parse_expr()?;
                    self.eat(&TokenKind::Newline);
                    // Wrap as assignment
                    let target = match expr {
                        Expr::Ident(id) => AssignTarget::Ident(id),
                        other => AssignTarget::Ident(Ident::new("_", span)), // fallback
                    };
                    Stmt::Assign(AssignStmt { span, target, op: AssignOp::Assign, value })
                } else {
                    self.eat(&TokenKind::Newline);
                    Stmt::Expr(ExprStmt { span, expr })
                }
            }
        };

        Some(stmt)
    }

    fn parse_if_stmt(&mut self) -> Option<Stmt> {
        let span = self.current_span();
        self.expect(&TokenKind::If)?;

        let condition = self.parse_expr()?;
        self.expect(&TokenKind::Colon)?;
        let then_body = self.parse_block();

        let mut else_body: Option<Block> = None;
        self.skip_newlines();
        if self.at(&TokenKind::Else) {
            self.advance(); // consume 'else'
            if self.at(&TokenKind::If) {
                // else if → wrap in another if
                // else if: 'if' token still pending, parse it
                let ei_span = self.current_span();
                self.advance(); // consume 'if'
                let ei_cond = self.parse_expr()?;
                self.expect(&TokenKind::Colon)?;
                let ei_body = self.parse_block();
                // For simplicity, wrap as else block containing if
                let inner_if = Stmt::If(IfStmt {
                    span: ei_span,
                    condition: ei_cond,
                    then_block: ei_body,
                    else_ifs: vec![],
                    else_block: None,
                });
                else_body = Some(Block { span: ei_span, stmts: vec![inner_if] });
            } else {
                self.expect(&TokenKind::Colon)?;
                else_body = Some(self.parse_block());
            }
        }

        Some(Stmt::If(IfStmt {
            span,
            condition,
            then_block: then_body,
            else_ifs: vec![],
            else_block: else_body,
        }))
    }

    fn parse_for_stmt(&mut self) -> Option<Stmt> {
        let span = self.current_span();
        self.expect(&TokenKind::For)?;
        let var = self.parse_ident("loop variable")?;
        self.expect(&TokenKind::In)?;
        let iter = self.parse_expr()?;
        self.expect(&TokenKind::Colon)?;
        let body = self.parse_block();
        Some(Stmt::For(ForStmt { span, binding: var, iterable: iter, body }))
    }

    fn parse_while_stmt(&mut self) -> Option<Stmt> {
        let span = self.current_span();
        self.expect(&TokenKind::While)?;
        let condition = self.parse_expr()?;
        self.expect(&TokenKind::Colon)?;
        let body = self.parse_block();
        Some(Stmt::While(WhileStmt { span, condition, body }))
    }

    fn parse_with_stmt(&mut self) -> Option<Stmt> {
        let span = self.current_span();
        self.expect(&TokenKind::With)?;
        let expr    = self.parse_expr()?;
        self.expect(&TokenKind::As)?;
        let binding = self.parse_ident("with binding name")?;
        self.expect(&TokenKind::Colon)?;
        let body    = self.parse_block();
        Some(Stmt::With(WithStmt { span, expr, binding, body }))
    }

    // ── Expression parsing (basic — P2-11 adds Pratt parser) ──

    pub fn parse_expr(&mut self) -> Option<Expr> {
        self.parse_expr_comparison()
    }

    fn parse_expr_comparison(&mut self) -> Option<Expr> {
        let mut left = self.parse_expr_additive()?;
        loop {
            let span = self.current_span();
            let op = match self.peek_kind() {
                TokenKind::EqEq    => { self.advance(); crate::ast::BinOp::Eq  }
                TokenKind::BangEq  => { self.advance(); crate::ast::BinOp::NotEq }
                TokenKind::Lt      => { self.advance(); crate::ast::BinOp::Lt  }
                TokenKind::Gt      => { self.advance(); crate::ast::BinOp::Gt  }
                TokenKind::LtEq    => { self.advance(); crate::ast::BinOp::LtEq}
                TokenKind::GtEq    => { self.advance(); crate::ast::BinOp::GtEq}
                _ => break,
            };
            let right = self.parse_expr_additive()?;
            left = Expr::BinOp(Box::new(crate::ast::BinOpExpr {
                span, op, lhs: left, rhs: right,
            }));
        }
        Some(left)
    }

    fn parse_expr_additive(&mut self) -> Option<Expr> {
        let mut left = self.parse_expr_multiplicative()?;
        loop {
            let span = self.current_span();
            let op = match self.peek_kind() {
                TokenKind::Plus  => { self.advance(); crate::ast::BinOp::Add }
                TokenKind::Minus => { self.advance(); crate::ast::BinOp::Sub }
                _ => break,
            };
            let right = self.parse_expr_multiplicative()?;
            left = Expr::BinOp(Box::new(crate::ast::BinOpExpr {
                span, op, lhs: left, rhs: right,
            }));
        }
        Some(left)
    }

    fn parse_expr_multiplicative(&mut self) -> Option<Expr> {
        let mut left = self.parse_expr_unary()?;
        loop {
            let span = self.current_span();
            let op = match self.peek_kind() {
                TokenKind::Star    => { self.advance(); crate::ast::BinOp::Mul }
                TokenKind::Slash   => { self.advance(); crate::ast::BinOp::Div }
                TokenKind::Percent => { self.advance(); crate::ast::BinOp::Mod }
                _ => break,
            };
            let right = self.parse_expr_unary()?;
            left = Expr::BinOp(Box::new(crate::ast::BinOpExpr {
                span, op, lhs: left, rhs: right,
            }));
        }
        Some(left)
    }

    fn parse_expr_unary(&mut self) -> Option<Expr> {
        let span = self.current_span();
        match self.peek_kind() {
            TokenKind::Minus => {
                self.advance();
                let expr = self.parse_expr_postfix()?;
                Some(Expr::UnaryOp(Box::new(crate::ast::UnaryOpExpr {
                    span, op: crate::ast::UnaryOp::Neg, expr,
                })))
            }
            TokenKind::Bang => {
                self.advance();
                let expr = self.parse_expr_postfix()?;
                Some(Expr::UnaryOp(Box::new(crate::ast::UnaryOpExpr {
                    span, op: crate::ast::UnaryOp::Not, expr,
                })))
            }
            _ => self.parse_expr_postfix(),
        }
    }

    fn parse_expr_postfix(&mut self) -> Option<Expr> {
        let mut expr = self.parse_expr_primary()?;
        loop {
            let span = self.current_span();
            match self.peek_kind() {
                // Method call or field access: expr.name
                TokenKind::Dot => {
                    self.advance();
                    let name = self.parse_ident("field or method name")?;
                    if self.at(&TokenKind::LParen) {
                        // Method call: expr.method(args)
                        self.advance();
                        let args = self.parse_call_args();
                        self.expect(&TokenKind::RParen)?;
                        expr = Expr::MethodCall(Box::new(crate::ast::MethodCallExpr {
                            span, receiver: Box::new(expr), method: name,
                            generics: vec![], args,
                        }));
                    } else {
                        // Field access: expr.field
                        expr = Expr::FieldAccess(Box::new(FieldAccessExpr {
                            span, object: Box::new(expr), field: name,
                        }));
                    }
                }
                // Function call: expr(args)
                TokenKind::LParen => {
                    self.advance();
                    let args = self.parse_call_args();
                    self.expect(&TokenKind::RParen)?;
                    // For now only handle direct ident calls
                    // P2-11 will handle complex callees
                    if let Expr::Ident(id) = expr {
                        expr = Expr::Call(Box::new(crate::ast::CallExpr {
                            span, callee: id, generics: vec![], args,
                        }));
                    } else {
                        // Skip complex callee for now
                        expr = Expr::Lit(Literal::None(span));
                    }
                }
                // Index: expr[idx]
                TokenKind::LBracket => {
                    self.advance();
                    let idx = self.parse_expr()?;
                    self.expect(&TokenKind::RBracket)?;
                    expr = Expr::Index(Box::new(crate::ast::IndexExpr {
                        span, object: Box::new(expr), index: Box::new(idx),
                    }));
                }
                // ? propagation
                TokenKind::Question => {
                    self.advance();
                    expr = Expr::Propagate(Box::new(expr), span);
                }
                _ => break,
            }
        }
        Some(expr)
    }

    fn parse_expr_primary(&mut self) -> Option<Expr> {
        let span = self.current_span();
        match self.peek_kind().clone() {
            // Integer literal
            TokenKind::IntLit(n) => {
                self.advance();
                Some(Expr::Lit(Literal::Int(n, span)))
            }
            // Float literal
            TokenKind::FloatLit(f) => {
                self.advance();
                Some(Expr::Lit(Literal::Float(f, span)))
            }
            // String literal
            TokenKind::StrLit(s) => {
                self.advance();
                Some(Expr::Lit(Literal::Str(s, span)))
            }
            // Bool literal
            TokenKind::BoolLit(b) => {
                self.advance();
                Some(Expr::Lit(Literal::Bool(b, span)))
            }
            // true / false
            TokenKind::True => {
                self.advance();
                Some(Expr::Lit(Literal::Bool(true, span)))
            }
            TokenKind::False => {
                self.advance();
                Some(Expr::Lit(Literal::Bool(false, span)))
            }
            // None
            TokenKind::None => {
                self.advance();
                Some(Expr::Lit(Literal::None(span)))
            }
            // Identifier or function call
            TokenKind::Ident(name) => {
                self.advance();
                Some(Expr::Ident(Ident::new(name, span)))
            }
            // Parenthesised expression
            TokenKind::LParen => {
                self.advance();
                let inner = self.parse_expr()?;
                self.expect(&TokenKind::RParen)?;
                Some(inner)
            }
            // List literal: [a, b, c]
            TokenKind::LBracket => {
                self.advance();
                let mut items = Vec::new();
                while !self.at_eof() && !self.at(&TokenKind::RBracket) {
                    if let Some(e) = self.parse_expr() { items.push(e); }
                    if self.eat(&TokenKind::Comma).is_none() { break; }
                }
                self.expect(&TokenKind::RBracket)?;
                Some(Expr::List(crate::ast::ListExpr { span, elements: items }))
            }
            _ => {
                // Don't error here — caller will decide
                None
            }
        }
    }

    fn parse_call_args(&mut self) -> Vec<crate::ast::Arg> {
        let mut args = Vec::new();
        while !self.at_eof() && !self.at(&TokenKind::RParen) {
            let span = self.current_span();
            // Check for named arg: name: expr
            let label = if matches!(self.peek_kind(), TokenKind::Ident(_)) {
                let saved = self.pos;
                let tok   = self.advance();
                if self.at(&TokenKind::Colon) {
                    self.advance();
                    if let TokenKind::Ident(n) = tok.kind {
                        Some(Ident::new(n, tok.span))
                    } else { None }
                } else { self.pos = saved; None }
            } else { None };
            let value = match self.parse_expr() {
                Some(e) => e,
                None => break,
            };
            args.push(crate::ast::Arg { span, label, value });
            if self.eat(&TokenKind::Comma).is_none() { break; }
        }
        args
    }

    // ── Type parsing ──────────────────────────────────────────

    pub fn parse_type(&mut self) -> Option<Type> {
        let span = self.current_span();
        match self.peek_kind().clone() {
            TokenKind::TInt     => { self.advance(); Some(Type::Primitive(PrimitiveType::Int,     span)) }
            TokenKind::TInt32   => { self.advance(); Some(Type::Primitive(PrimitiveType::Int32,   span)) }
            TokenKind::TInt64   => { self.advance(); Some(Type::Primitive(PrimitiveType::Int64,   span)) }
            TokenKind::TInt8    => { self.advance(); Some(Type::Primitive(PrimitiveType::Int8,    span)) }
            TokenKind::TUInt    => { self.advance(); Some(Type::Primitive(PrimitiveType::UInt,    span)) }
            TokenKind::TUInt32  => { self.advance(); Some(Type::Primitive(PrimitiveType::UInt32,  span)) }
            TokenKind::TUInt64  => { self.advance(); Some(Type::Primitive(PrimitiveType::UInt64,  span)) }
            TokenKind::TUInt8   => { self.advance(); Some(Type::Primitive(PrimitiveType::UInt8,   span)) }
            TokenKind::TFloat   => { self.advance(); Some(Type::Primitive(PrimitiveType::Float,   span)) }
            TokenKind::TFloat32 => { self.advance(); Some(Type::Primitive(PrimitiveType::Float32, span)) }
            TokenKind::TBool    => { self.advance(); Some(Type::Primitive(PrimitiveType::Bool,    span)) }
            TokenKind::TChar    => { self.advance(); Some(Type::Primitive(PrimitiveType::Char,    span)) }
            TokenKind::TStr     => { self.advance(); Some(Type::Primitive(PrimitiveType::Str,     span)) }
            TokenKind::TBytes   => { self.advance(); Some(Type::Primitive(PrimitiveType::Bytes,   span)) }
            TokenKind::TUnit    => { self.advance(); Some(Type::Unit(span)) }
            TokenKind::TOption  => {
                self.advance();
                self.expect(&TokenKind::Lt)?;
                let inner = self.parse_type()?;
                self.expect(&TokenKind::Gt)?;
                Some(Type::Option(Box::new(inner), span))
            }
            TokenKind::TResult  => {
                self.advance();
                self.expect(&TokenKind::Lt)?;
                let ok  = self.parse_type()?;
                self.expect(&TokenKind::Comma)?;
                let err = self.parse_type()?;
                self.expect(&TokenKind::Gt)?;
                Some(Type::Result(Box::new(ok), Box::new(err), span))
            }
            TokenKind::TList    => {
                self.advance();
                self.expect(&TokenKind::Lt)?;
                let inner = self.parse_type()?;
                self.expect(&TokenKind::Gt)?;
                Some(Type::List(Box::new(inner), span))
            }
            TokenKind::TCap     => {
                self.advance();
                self.expect(&TokenKind::Lt)?;
                let inner = self.parse_type()?;
                self.expect(&TokenKind::Gt)?;
                Some(Type::Cap(Box::new(inner), span))
            }
            TokenKind::Ident(name) => {
                let name = name.clone(); self.advance();
                if self.at(&TokenKind::Lt) {
                    self.advance();
                    let inner = self.parse_type()?;
                    self.expect(&TokenKind::Gt)?;
                    let id = Ident::new(name, span);
                    Some(Type::Generic(id, vec![inner], span))
                } else {
                    Some(Type::Named(vec![Ident::new(name, span)]))
                }
            }
            _ => {
                self.error(span, format!("expected type, found {}",
                    self.peek_kind().display_name()));
                None
            }
        }
    }

    pub fn parse_ident(&mut self, context: &str) -> Option<Ident> {
        match self.peek_kind().clone() {
            TokenKind::Ident(name) => {
                let span = self.current_span(); self.advance();
                Some(Ident::new(name, span))
            }
            _ => {
                let span = self.current_span();
                self.error(span, format!("expected {} (identifier), found {}",
                    context, self.peek_kind().display_name()));
                None
            }
        }
    }

    // ── Program entry point ───────────────────────────────────

    pub fn parse_program(&mut self) -> Program {
        let start = self.current_span();
        self.skip_newlines();
        let program_intent = self.parse_program_intent();
        self.skip_newlines();
        let module = self.parse_module_decl();
        self.skip_newlines();

        let mut imports = Vec::new();
        while self.at(&TokenKind::Import) {
            if let Some(imp) = self.parse_import_decl() { imports.push(imp); }
            self.skip_newlines();
        }

        let mut items: Vec<TopLevelItem> = Vec::new();
        while !self.at_eof() {
            self.skip_newlines();
            if self.at_eof() { break; }
            let decorators = self.parse_decorators();
            let item = match self.peek_kind() {
                TokenKind::Struct => self.parse_struct_decl().map(TopLevelItem::Struct),
                TokenKind::Enum   => self.parse_enum_decl().map(TopLevelItem::Enum),
                TokenKind::Fn     => self.parse_fn_decl(decorators).map(TopLevelItem::Fn),
                TokenKind::Task   => self.parse_task_decl(decorators).map(TopLevelItem::Task),
                _ => { self.advance(); None }
            };
            if let Some(i) = item { items.push(i); }
        }

        let end = self.current_span();
        Program {
            span: Self::merge(start, end),
            program_intent, module, imports, items,
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use axon_lexer::{lex, FileId, Span};
    use crate::ast::*;

    fn file() -> FileId { FileId(1) }

    fn make_parser(source: &str) -> Parser {
        let tokens = axon_lexer::inject_indentation(lex(source, file()));
        Parser::new(tokens, source, file())
    }

    // ── Existing tests (P2-05 through P2-09) ─────────────────

    #[test] fn test_parse_empty_source() {
        let r = crate::parse("", file());
        assert!(r.is_ok()); assert!(r.errors.is_empty());
    }

    #[test] fn test_parse_result_struct() {
        let r = crate::parse("", file());
        assert!(r.program.imports.is_empty());
        assert!(r.program.items.is_empty());
    }

    #[test] fn test_ast_program_has_program_intent_field() {
        assert!(crate::parse("", file()).program.program_intent.is_none());
    }

    #[test] fn test_ast_program_has_all_fields() {
        let r = crate::parse("", file());
        let _ = (&r.program.module, &r.program.imports,
                 &r.program.items,  &r.program.program_intent);
    }

    #[test] fn test_stmt_enum_has_defer_variant() {
        let span = Span::new(file(), 0, 0, 1, 1);
        let ds   = DeferStmt { span, expr: Expr::Lit(Literal::Bool(true, span)) };
        let _    = Stmt::Defer(ds);
    }

    #[test] fn test_stmt_enum_has_with_variant() {
        let span = Span::new(file(), 0, 0, 1, 1);
        let ws   = WithStmt {
            span, binding: Ident::new("f", span),
            expr: Expr::Lit(Literal::Bool(true, span)),
            body: Block { span, stmts: vec![] },
        };
        let _ = Stmt::With(ws);
    }

    #[test] fn test_ast_intent_modes_all_exist() {
        let _ = (IntentMode::Secure, IntentMode::Performant,
                 IntentMode::Auditable, IntentMode::Verifiable,
                 IntentMode::MinimalRuntime);
    }

    #[test] fn test_ast_mem_modes_all_exist() {
        let _ = (MemMode::Own, MemMode::Borrow, MemMode::MutBorrow, MemMode::Share);
    }

    #[test] fn test_ast_provenance_kinds_exist() {
        let _ = (ProvenanceKind::Tainted, ProvenanceKind::Clean,
                 ProvenanceKind::Network, ProvenanceKind::FileSystem,
                 ProvenanceKind::UserInput, ProvenanceKind::Trusted,
                 ProvenanceKind::Unknown);
    }

    #[test] fn test_ast_temporal_expr_variants_exist() {
        let span = Span::new(file(), 0, 0, 1, 1);
        let _ = (TemporalExpr::Now(span), TemporalExpr::Lifetime(span),
                 TemporalExpr::Epoch(span), TemporalOp::Add, TemporalOp::Sub);
    }

    #[test] fn test_ast_actor_decl_exists() {
        let span = Span::new(file(), 0, 0, 1, 1);
        let a    = ActorDecl { span, name: Ident::new("W", span), items: vec![] };
        assert_eq!(a.items.len(), 0);
    }

    #[test] fn test_ast_opaque_type_decl_exists() {
        let span   = Span::new(file(), 0, 0, 1, 1);
        let opaque = OpaqueTypeDecl {
            span, name: Ident::new("UserId", span),
            ty: Type::Primitive(PrimitiveType::Int, span),
        };
        assert_eq!(opaque.name.name, "UserId");
    }

    #[test] fn test_ast_bin_op_binding_power_ordering() {
        let (ml,_)  = BinOp::Mul.binding_power();
        let (al,_)  = BinOp::Add.binding_power();
        let (cl,_)  = BinOp::Eq.binding_power();
        let (al2,_) = BinOp::And.binding_power();
        let (ol,_)  = BinOp::Or.binding_power();
        assert!(ml > al); assert!(al > cl);
        assert!(cl > al2); assert!(al2 > ol);
    }

    #[test] fn test_ast_literal_span_extraction() {
        let span = Span::new(file(), 0, 5, 1, 1);
        assert_eq!(Literal::Int(42, span).span(), span);
        assert_eq!(Literal::Bool(true, span).span(), span);
        assert_eq!(Literal::None(span).span(), span);
    }

    #[test] fn test_cursor_peek_does_not_consume() {
        let mut p = make_parser("fn");
        assert_eq!(p.peek().kind.clone(), p.peek().kind.clone());
    }

    #[test] fn test_cursor_advance_consumes() {
        let mut p  = make_parser("fn task");
        let first  = p.advance().kind;
        let second = p.peek().kind.clone();
        assert_eq!(first,  TokenKind::Fn);
        assert_eq!(second, TokenKind::Task);
    }

    #[test] fn test_cursor_eat_matches() {
        let mut p = make_parser("fn");
        assert!(p.eat(&TokenKind::Fn).is_some());
    }

    #[test] fn test_cursor_eat_no_match() {
        let mut p = make_parser("fn");
        assert!(p.eat(&TokenKind::Task).is_none());
        assert_eq!(*p.peek_kind(), TokenKind::Fn);
    }

    #[test] fn test_cursor_expect_records_error() {
        let mut p  = make_parser("fn");
        let result = p.expect(&TokenKind::Task);
        assert!(result.is_none());
        assert_eq!(p.errors.len(), 1);
        match &p.errors[0] {
            ParseError::Custom { message, .. } =>
                assert!(message.contains("task"), "got: {}", message),
            ParseError::UnexpectedToken { expected, .. } =>
                assert!(expected.contains("task"), "got: {}", expected),
            other => panic!("unexpected: {:?}", other),
        }
    }

    #[test] fn test_cursor_at_eof() { assert!(make_parser("").at_eof()); }

    #[test] fn test_cursor_skip_newlines() {
        let mut p = make_parser("fn");
        p.skip_newlines();
        assert_eq!(*p.peek_kind(), TokenKind::Fn);
    }

    #[test] fn test_error_cap_at_20() {
        let mut p = make_parser("");
        let span  = Span::new(file(), 0, 0, 1, 1);
        for i in 0..25 { p.error(span, format!("e{}", i)); }
        assert_eq!(p.errors.len(), 20);
    }

    #[test] fn test_parse_module_decl() {
        let r = crate::parse("module hello\n", file());
        assert!(r.errors.is_empty(), "{:?}", r.errors);
        assert_eq!(r.program.module.unwrap().path[0].name, "hello");
    }

    #[test] fn test_parse_module_path_dotted() {
        let r = crate::parse("module aieonyx.aegis.monitor\n", file());
        assert!(r.errors.is_empty(), "{:?}", r.errors);
        let m = r.program.module.unwrap();
        assert_eq!(m.path.len(), 3);
    }

    #[test] fn test_parse_import_simple() {
        let r = crate::parse("module hello\nimport axon.sys\n", file());
        assert!(r.errors.is_empty(), "{:?}", r.errors);
        assert_eq!(r.program.imports.len(), 1);
    }

    #[test] fn test_parse_import_with_alias() {
        let r = crate::parse("module hello\nimport axon.sys.sel4.ipc as ipc\n", file());
        assert!(r.errors.is_empty(), "{:?}", r.errors);
        assert_eq!(r.program.imports[0].alias.as_ref().unwrap().name, "ipc");
    }

    #[test] fn test_parse_multiple_imports() {
        let r = crate::parse("module hello\nimport axon.sys\nimport axon.web\n", file());
        assert!(r.errors.is_empty(), "{:?}", r.errors);
        assert_eq!(r.program.imports.len(), 2);
    }

    #[test] fn test_parse_struct_with_fields() {
        let src = "struct Config:\n    host : Str\n    port : Int\n";
        let r   = crate::parse(src, file());
        assert!(r.errors.is_empty(), "{:?}", r.errors);
        match &r.program.items[0] {
            TopLevelItem::Struct(s) => {
                assert_eq!(s.name.name, "Config");
                assert_eq!(s.fields.len(), 2);
            }
            other => panic!("expected Struct, got {:?}", other),
        }
    }

    #[test] fn test_parse_enum_no_fields() {
        let src = "enum Status:\n    Active\n    Inactive\n";
        let r   = crate::parse(src, file());
        assert!(r.errors.is_empty(), "{:?}", r.errors);
        match &r.program.items[0] {
            TopLevelItem::Enum(e) => assert_eq!(e.variants.len(), 2),
            other => panic!("expected Enum, got {:?}", other),
        }
    }

    #[test] fn test_parse_enum_with_fields() {
        let src = "enum ThreatLevel:\n    Clear\n    Advisory(detail: Str)\n    Critical(layer: Int, detail: Str)\n";
        let r   = crate::parse(src, file());
        assert!(r.errors.is_empty(), "{:?}", r.errors);
        match &r.program.items[0] {
            TopLevelItem::Enum(e) => {
                assert_eq!(e.variants[0].fields.len(), 0);
                assert_eq!(e.variants[1].fields.len(), 1);
                assert_eq!(e.variants[2].fields.len(), 2);
            }
            other => panic!("expected Enum, got {:?}", other),
        }
    }

    #[test] fn test_parse_struct_and_enum_together() {
        let src = concat!(
            "module test\n",
            "struct Point:\n", "    x : Int\n", "    y : Int\n",
            "enum Color:\n",   "    Red\n", "    Green\n", "    Blue\n",
        );
        let r = crate::parse(src, file());
        assert!(r.errors.is_empty(), "{:?}", r.errors);
        assert_eq!(r.program.items.len(), 2);
    }

    #[test] fn test_parse_fn_minimal() {
        let src = "fn hello():\n    pass\n";
        let r   = crate::parse(src, file());
        assert!(r.errors.is_empty(), "{:?}", r.errors);
        match &r.program.items[0] {
            TopLevelItem::Fn(f) => assert_eq!(f.name.name, "hello"),
            other => panic!("expected Fn, got {:?}", other),
        }
    }

    #[test] fn test_parse_fn_with_params() {
        let src = "fn add(x : Int, y : Int) -> Int:\n    pass\n";
        let r   = crate::parse(src, file());
        assert!(r.errors.is_empty(), "{:?}", r.errors);
        match &r.program.items[0] {
            TopLevelItem::Fn(f) => {
                assert_eq!(f.params.len(), 2);
                assert!(f.ret_type.is_some());
            }
            other => panic!("expected Fn, got {:?}", other),
        }
    }

    #[test] fn test_parse_fn_with_mem_mode() {
        let src = "fn process(own data : Str) -> Int:\n    pass\n";
        let r   = crate::parse(src, file());
        assert!(r.errors.is_empty(), "{:?}", r.errors);
        match &r.program.items[0] {
            TopLevelItem::Fn(f) => {
                assert_eq!(f.params[0].mem_mode, Some(MemMode::Own));
            }
            other => panic!("expected Fn, got {:?}", other),
        }
    }

    #[test] fn test_parse_fn_with_effects() {
        let src = "fn read_file(path : Str) -> Str uses [io.read]:\n    pass\n";
        let r   = crate::parse(src, file());
        assert!(r.errors.is_empty(), "{:?}", r.errors);
        match &r.program.items[0] {
            TopLevelItem::Fn(f) => {
                assert!(f.uses.is_some());
                assert_eq!(f.uses.as_ref().unwrap().effects.len(), 1);
            }
            other => panic!("expected Fn, got {:?}", other),
        }
    }

    #[test] fn test_parse_fn_with_decorator() {
        let src = "@ai.intent(\"always returns positive\")\nfn abs(x : Int) -> Int:\n    pass\n";
        let r   = crate::parse(src, file());
        assert_eq!(r.program.items.len(), 1, "errors: {:?}", r.errors);
        match &r.program.items[0] {
            TopLevelItem::Fn(f) => assert_eq!(f.name.name, "abs"),
            other => panic!("expected Fn, got {:?}", other),
        }
    }

    #[test] fn test_parse_task_decl() {
        let src = "task monitor() -> Int:\n    pass\n";
        let r   = crate::parse(src, file());
        assert!(r.errors.is_empty(), "{:?}", r.errors);
        match &r.program.items[0] {
            TopLevelItem::Task(t) => assert_eq!(t.name.name, "monitor"),
            other => panic!("expected Task, got {:?}", other),
        }
    }

    #[test] fn test_parse_multiple_fns() {
        let src = concat!("fn foo():\n", "    pass\n", "fn bar():\n", "    pass\n");
        let r   = crate::parse(src, file());
        assert!(r.errors.is_empty(), "{:?}", r.errors);
        assert_eq!(r.program.items.len(), 2);
    }

    #[test] fn test_parse_fn_no_parens() {
        let src = "fn main:\n    pass\n";
        let r   = crate::parse(src, file());
        assert_eq!(r.program.items.len(), 1);
    }

    // ── P2-10: Statement tests ────────────────────────────────

    #[test] fn test_parse_let_stmt() {
        let src = "fn f():\n    let x = 42\n";
        let r   = crate::parse(src, file());
        assert!(r.errors.is_empty(), "{:?}", r.errors);
        match &r.program.items[0] {
            TopLevelItem::Fn(f) => {
                assert_eq!(f.body.stmts.len(), 1);
                assert!(matches!(f.body.stmts[0], Stmt::Let(_)));
            }
            other => panic!("{:?}", other),
        }
    }

    #[test] fn test_parse_let_with_type() {
        let src = "fn f():\n    let x : Int = 42\n";
        let r   = crate::parse(src, file());
        assert!(r.errors.is_empty(), "{:?}", r.errors);
        match &r.program.items[0] {
            TopLevelItem::Fn(f) => {
                match &f.body.stmts[0] {
                    Stmt::Let(s) => {
                        assert_eq!(s.name.name, "x");
                        assert!(s.ty.is_some());
                    }
                    other => panic!("{:?}", other),
                }
            }
            other => panic!("{:?}", other),
        }
    }

    #[test] fn test_parse_mut_stmt() {
        let src = "fn f():\n    mut count = 0\n";
        let r   = crate::parse(src, file());
        assert!(r.errors.is_empty(), "{:?}", r.errors);
        match &r.program.items[0] {
            TopLevelItem::Fn(f) => {
                assert!(matches!(f.body.stmts[0], Stmt::Mut(_)));
            }
            other => panic!("{:?}", other),
        }
    }

    #[test] fn test_parse_return_stmt() {
        let src = "fn f() -> Int:\n    return 42\n";
        let r   = crate::parse(src, file());
        assert!(r.errors.is_empty(), "{:?}", r.errors);
        match &r.program.items[0] {
            TopLevelItem::Fn(f) => {
                match &f.body.stmts[0] {
                    Stmt::Return(s) => assert!(s.value.is_some()),
                    other => panic!("{:?}", other),
                }
            }
            other => panic!("{:?}", other),
        }
    }

    #[test] fn test_parse_return_void() {
        let src = "fn f():\n    return\n";
        let r   = crate::parse(src, file());
        assert!(r.errors.is_empty(), "{:?}", r.errors);
        match &r.program.items[0] {
            TopLevelItem::Fn(f) => {
                match &f.body.stmts[0] {
                    Stmt::Return(s) => assert!(s.value.is_none()),
                    other => panic!("{:?}", other),
                }
            }
            other => panic!("{:?}", other),
        }
    }

    #[test] fn test_parse_if_stmt() {
        let src = "fn f():\n    if x:\n        pass\n";
        let r   = crate::parse(src, file());
        assert!(r.errors.is_empty(), "{:?}", r.errors);
        match &r.program.items[0] {
            TopLevelItem::Fn(f) => {
                assert!(matches!(f.body.stmts[0], Stmt::If(_)));
            }
            other => panic!("{:?}", other),
        }
    }

    #[test] fn test_parse_if_else_stmt() {
        let src = "fn f():\n    if x:\n        pass\n    else:\n        pass\n";
        let r   = crate::parse(src, file());
        assert!(r.errors.is_empty(), "{:?}", r.errors);
        match &r.program.items[0] {
            TopLevelItem::Fn(f) => {
                match &f.body.stmts[0] {
                    Stmt::If(s) => assert!(s.else_block.is_some()),
                    other => panic!("{:?}", other),
                }
            }
            other => panic!("{:?}", other),
        }
    }

    #[test] fn test_parse_for_stmt() {
        let src = "fn f():\n    for x in items:\n        pass\n";
        let r   = crate::parse(src, file());
        assert!(r.errors.is_empty(), "{:?}", r.errors);
        match &r.program.items[0] {
            TopLevelItem::Fn(f) => {
                match &f.body.stmts[0] {
                    Stmt::For(s) => assert_eq!(s.binding.name, "x"),
                    other => panic!("{:?}", other),
                }
            }
            other => panic!("{:?}", other),
        }
    }

    #[test] fn test_parse_while_stmt() {
        let src = "fn f():\n    while running:\n        pass\n";
        let r   = crate::parse(src, file());
        assert!(r.errors.is_empty(), "{:?}", r.errors);
        match &r.program.items[0] {
            TopLevelItem::Fn(f) => {
                assert!(matches!(f.body.stmts[0], Stmt::While(_)));
            }
            other => panic!("{:?}", other),
        }
    }

    #[test] fn test_parse_defer_stmt() {
        let src = "fn f():\n    defer cleanup()\n";
        let r   = crate::parse(src, file());
        assert!(r.errors.is_empty(), "{:?}", r.errors);
        match &r.program.items[0] {
            TopLevelItem::Fn(f) => {
                assert!(matches!(f.body.stmts[0], Stmt::Defer(_)));
            }
            other => panic!("{:?}", other),
        }
    }

    #[test] fn test_parse_expr_stmt() {
        let src = "fn f():\n    print(42)\n";
        let r   = crate::parse(src, file());
        assert!(r.errors.is_empty(), "{:?}", r.errors);
        match &r.program.items[0] {
            TopLevelItem::Fn(f) => {
                assert!(matches!(f.body.stmts[0], Stmt::Expr(_)));
            }
            other => panic!("{:?}", other),
        }
    }

    #[test] fn test_parse_multiple_stmts() {
        let src = concat!(
            "fn process(x : Int) -> Int:\n",
            "    let y = x\n",
            "    mut z = 0\n",
            "    return y\n",
        );
        let r = crate::parse(src, file());
        assert!(r.errors.is_empty(), "{:?}", r.errors);
        match &r.program.items[0] {
            TopLevelItem::Fn(f) => assert_eq!(f.body.stmts.len(), 3),
            other => panic!("{:?}", other),
        }
    }

    #[test] fn test_parse_int_literal_expr() {
        let src = "fn f():\n    let x = 99\n";
        let r   = crate::parse(src, file());
        assert!(r.errors.is_empty(), "{:?}", r.errors);
        match &r.program.items[0] {
            TopLevelItem::Fn(f) => {
                match &f.body.stmts[0] {
                    Stmt::Let(s) => {
                        assert!(matches!(s.init, Expr::Lit(Literal::Int(99, _))));
                    }
                    other => panic!("{:?}", other),
                }
            }
            other => panic!("{:?}", other),
        }
    }

    #[test] fn test_parse_bool_literal_expr() {
        let src = "fn f():\n    let flag = true\n";
        let r   = crate::parse(src, file());
        assert!(r.errors.is_empty(), "{:?}", r.errors);
        match &r.program.items[0] {
            TopLevelItem::Fn(f) => {
                match &f.body.stmts[0] {
                    Stmt::Let(s) => {
                        assert!(matches!(s.init, Expr::Lit(Literal::Bool(true, _))));
                    }
                    other => panic!("{:?}", other),
                }
            }
            other => panic!("{:?}", other),
        }
    }
}
