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
    MatchStmt, MatchArm,
    Pattern, PatternField, EnumPattern,
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

            // match expr: arms
            TokenKind::Match => { self.parse_match_stmt()? }

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

    fn parse_match_stmt(&mut self) -> Option<Stmt> {
        let span = self.current_span();
        self.expect(&TokenKind::Match)?;
        let subject = self.parse_expr()?;
        self.expect(&TokenKind::Colon)?;
        self.eat(&TokenKind::Newline);
        self.eat(&TokenKind::Indent);

        let mut arms = Vec::new();
        while !self.at_eof() && !self.at(&TokenKind::Dedent) {
            self.skip_newlines();
            if self.at(&TokenKind::Dedent) || self.at_eof() { break; }
            if let Some(arm) = self.parse_match_arm() {
                arms.push(arm);
            } else {
                // Skip bad line
                while !self.at_eof()
                    && !matches!(self.peek_kind(), TokenKind::Newline | TokenKind::Dedent) {
                    self.advance();
                }
                self.eat(&TokenKind::Newline);
            }
        }

        self.eat(&TokenKind::Dedent);
        Some(Stmt::Match(MatchStmt { span, subject, arms }))
    }

    fn parse_match_arm(&mut self) -> Option<MatchArm> {
        let span = self.current_span();

        // Parse pattern
        let pattern = self.parse_pattern()?;

        // Optional guard: if expr
        let guard = if self.at(&TokenKind::If) {
            self.advance();
            self.parse_expr()
        } else { None };

        // =>
        self.expect(&TokenKind::FatArrow)?;

        // Body — for single-line arms: one expression
        // For multi-line: parse until newline
        let body = if matches!(self.peek_kind(), TokenKind::Newline | TokenKind::Dedent) {
            // Empty arm body — use None literal
            Expr::Lit(Literal::None(span))
        } else {
            self.parse_expr().unwrap_or(Expr::Lit(Literal::None(span)))
        };

        self.eat(&TokenKind::Newline);
        Some(MatchArm { span, pattern, guard, body })
    }

    // ── Pattern parsing ───────────────────────────────────────

    pub fn parse_pattern(&mut self) -> Option<Pattern> {
        let span = self.current_span();

        // Try primary pattern first
        let mut pat = self.parse_pattern_primary()?;

        // Or pattern: pat | pat | pat
        if self.at(&TokenKind::Pipe) {
            let mut pats = vec![pat];
            while self.at(&TokenKind::Pipe) {
                self.advance();
                if let Some(p) = self.parse_pattern_primary() {
                    pats.push(p);
                }
            }
            pat = Pattern::Or(pats, span);
        }

        Some(pat)
    }

    fn parse_pattern_primary(&mut self) -> Option<Pattern> {
        let span = self.current_span();

        match self.peek_kind().clone() {

            // Wildcard: _
            TokenKind::Ident(ref n) if n == "_" => {
                self.advance();
                Some(Pattern::Wildcard(span))
            }

            // Identifier — could be binding or enum variant
            TokenKind::Ident(name) => {
                self.advance();
                let ident = Ident::new(name, span);

                // Check for enum variant with fields: Variant(pat, pat)
                if self.at(&TokenKind::LParen) {
                    self.advance();
                    let mut fields = Vec::new();
                    while !self.at_eof() && !self.at(&TokenKind::RParen) {
                        let fspan = self.current_span();
                        // named: ident: pat  OR  positional: pat
                        if matches!(self.peek_kind(), TokenKind::Ident(_)) {
                            let saved = self.pos;
                            let tok   = self.advance();
                            if self.at(&TokenKind::Colon) {
                                self.advance();
                                if let TokenKind::Ident(n) = tok.kind {
                                    let label = Ident::new(n, tok.span);
                                    let inner = self.parse_pattern()?;
                                    fields.push(PatternField::Named(label, inner));
                                }
                            } else {
                                self.pos = saved;
                                let inner = self.parse_pattern()?;
                                fields.push(PatternField::Positional(inner));
                            }
                        } else {
                            let inner = self.parse_pattern()?;
                            fields.push(PatternField::Positional(inner));
                        }
                        self.eat(&TokenKind::Comma);
                    }
                    self.expect(&TokenKind::RParen)?;
                    Some(Pattern::Enum(EnumPattern { span, name: ident, fields }))
                } else {
                    // Could be enum unit variant or binding
                    // In AXON, uppercase = variant, lowercase = binding
                    let first_char = ident.name.chars().next().unwrap_or('a');
                    if first_char.is_uppercase() {
                        Some(Pattern::Enum(EnumPattern { span, name: ident, fields: vec![] }))
                    } else {
                        Some(Pattern::Binding(ident))
                    }
                }
            }

            // Literal patterns
            TokenKind::IntLit(n) => {
                let n = n; self.advance();
                Some(Pattern::Literal(Literal::Int(n, span)))
            }
            TokenKind::FloatLit(f) => {
                let f = f; self.advance();
                Some(Pattern::Literal(Literal::Float(f, span)))
            }
            TokenKind::StrLit(s) => {
                let s = s; self.advance();
                Some(Pattern::Literal(Literal::Str(s, span)))
            }
            TokenKind::BoolLit(b) => {
                let b = b; self.advance();
                Some(Pattern::Literal(Literal::Bool(b, span)))
            }
            TokenKind::True => {
                self.advance();
                Some(Pattern::Literal(Literal::Bool(true, span)))
            }
            TokenKind::False => {
                self.advance();
                Some(Pattern::Literal(Literal::Bool(false, span)))
            }
            TokenKind::None => {
                self.advance();
                Some(Pattern::Literal(Literal::None(span)))
            }

            // Tuple pattern: (pat, pat)
            TokenKind::LParen => {
                self.advance();
                let mut pats = Vec::new();
                while !self.at_eof() && !self.at(&TokenKind::RParen) {
                    if let Some(p) = self.parse_pattern() { pats.push(p); }
                    if self.eat(&TokenKind::Comma).is_none() { break; }
                }
                self.expect(&TokenKind::RParen)?;
                if pats.len() == 1 {
                    Some(pats.remove(0)) // strip single parens
                } else {
                    Some(Pattern::Tuple(pats, span))
                }
            }

            _ => None,
        }
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

    // ── P2-11: Pratt Expression Parser ──────────────────────────

    /// Top-level expression entry point.
    /// min_bp: minimum binding power — only consume operators with bp > min_bp
    pub fn parse_expr(&mut self) -> Option<Expr> {
        self.pratt_expr(0)
    }

    fn pratt_expr(&mut self, min_bp: u8) -> Option<Expr> {
        // Null denotation (prefix / primary)
        let mut left = self.parse_nud()?;

        loop {
            // Check for pipe: expr |> fn(args)
            if self.at(&TokenKind::PipeForward) {
                if min_bp >= 5 { break; }
                left = self.parse_pipe_expr(left)?;
                continue;
            }

            // Get infix binding power of next token
            let (l_bp, r_bp) = match self.infix_bp() {
                Some(bp) => bp,
                None => break,
            };

            if l_bp <= min_bp { break; }

            left = self.parse_led(left, r_bp)?;
        }

        Some(left)
    }

    /// Null denotation — starts an expression
    fn parse_nud(&mut self) -> Option<Expr> {
        let span = self.current_span();

        match self.peek_kind().clone() {

            // ── Literals ───────────────────────────────────────
            TokenKind::IntLit(n) => {
                self.advance();
                Some(Expr::Lit(Literal::Int(n, span)))
            }
            TokenKind::FloatLit(f) => {
                self.advance();
                Some(Expr::Lit(Literal::Float(f, span)))
            }
            TokenKind::StrLit(s) => {
                self.advance();
                Some(Expr::Lit(Literal::Str(s, span)))
            }
            TokenKind::BoolLit(b) => {
                self.advance();
                Some(Expr::Lit(Literal::Bool(b, span)))
            }
            TokenKind::True => {
                self.advance();
                Some(Expr::Lit(Literal::Bool(true, span)))
            }
            TokenKind::False => {
                self.advance();
                Some(Expr::Lit(Literal::Bool(false, span)))
            }
            TokenKind::None => {
                self.advance();
                Some(Expr::Lit(Literal::None(span)))
            }

            // ── Identifier ─────────────────────────────────────
            TokenKind::Ident(name) => {
                self.advance();
                Some(Expr::Ident(Ident::new(name, span)))
            }

            // ── Unary operators ────────────────────────────────
            TokenKind::Minus => {
                self.advance();
                let expr = self.pratt_expr(70)?; // high bp for unary
                Some(Expr::UnaryOp(Box::new(crate::ast::UnaryOpExpr {
                    span, op: crate::ast::UnaryOp::Neg, expr,
                })))
            }
            TokenKind::Bang => {
                self.advance();
                let expr = self.pratt_expr(70)?;
                Some(Expr::UnaryOp(Box::new(crate::ast::UnaryOpExpr {
                    span, op: crate::ast::UnaryOp::Not, expr,
                })))
            }

            // ── Grouped expression ─────────────────────────────
            TokenKind::LParen => {
                self.advance();
                let inner = self.pratt_expr(0)?;
                self.expect(&TokenKind::RParen)?;
                Some(inner)
            }

            // ── List literal: [a, b, c] ────────────────────────
            TokenKind::LBracket => {
                self.advance();
                let mut elements = Vec::new();
                while !self.at_eof() && !self.at(&TokenKind::RBracket) {
                    if let Some(e) = self.pratt_expr(0) { elements.push(e); }
                    if self.eat(&TokenKind::Comma).is_none() { break; }
                }
                self.expect(&TokenKind::RBracket)?;
                Some(Expr::List(crate::ast::ListExpr { span, elements }))
            }

            // Return as expression (valid in match arm bodies)
            // return expr  OR  return (void)
            TokenKind::Return => {
                self.advance();
                let value = if !matches!(self.peek_kind(),
                    TokenKind::Newline | TokenKind::Dedent | TokenKind::Eof |
                    TokenKind::Comma   | TokenKind::RParen) {
                    self.pratt_expr(0)
                } else { None };
                Some(Expr::Return(Box::new(value), span))
            }

            // Break as expression
            TokenKind::Break => {
                self.advance();
                Some(Expr::Break_(Box::new(None), span))
            }

            _ => None,
        }
    }

    /// Infix binding power — returns (left_bp, right_bp) or None
    fn infix_bp(&self) -> Option<(u8, u8)> {
        match self.peek_kind() {
            // Logical
            TokenKind::Or         => Some((10, 11)),
            TokenKind::And        => Some((20, 21)),
            // Comparison — non-associative (30/31)
            TokenKind::EqEq       => Some((30, 31)),
            TokenKind::BangEq     => Some((30, 31)),
            TokenKind::Lt         => Some((30, 31)),
            TokenKind::Gt         => Some((30, 31)),
            TokenKind::LtEq       => Some((30, 31)),
            TokenKind::GtEq       => Some((30, 31)),
            // Range
            TokenKind::DotDot     => Some((35, 36)),
            TokenKind::DotDotEq   => Some((35, 36)),
            // Additive
            TokenKind::Plus       => Some((40, 41)),
            TokenKind::Minus      => Some((40, 41)),
            // Multiplicative
            TokenKind::Star       => Some((50, 51)),
            TokenKind::Slash      => Some((50, 51)),
            TokenKind::Percent    => Some((50, 51)),
            // Postfix (call, field, index, ?) — highest infix bp
            TokenKind::Dot        => Some((80, 81)),
            TokenKind::LParen     => Some((80, 81)),
            TokenKind::LBracket   => Some((80, 81)),
            TokenKind::Question   => Some((80, 81)),
            TokenKind::Bang       => Some((80, 81)),  // capability pin !
            _ => None,
        }
    }

    /// Left denotation — continues an expression with an infix/postfix operator
    fn parse_led(&mut self, left: Expr, r_bp: u8) -> Option<Expr> {
        let span = self.current_span();

        match self.peek_kind().clone() {

            // ── Binary operators ───────────────────────────────
            TokenKind::Or         => { self.advance(); let r = self.pratt_expr(r_bp)?; Some(self.bin(span, left, crate::ast::BinOp::Or,     r)) }
            TokenKind::And        => { self.advance(); let r = self.pratt_expr(r_bp)?; Some(self.bin(span, left, crate::ast::BinOp::And,    r)) }
            TokenKind::EqEq       => { self.advance(); let r = self.pratt_expr(r_bp)?; Some(self.bin(span, left, crate::ast::BinOp::Eq,     r)) }
            TokenKind::BangEq     => { self.advance(); let r = self.pratt_expr(r_bp)?; Some(self.bin(span, left, crate::ast::BinOp::NotEq,  r)) }
            TokenKind::Lt         => { self.advance(); let r = self.pratt_expr(r_bp)?; Some(self.bin(span, left, crate::ast::BinOp::Lt,     r)) }
            TokenKind::Gt         => { self.advance(); let r = self.pratt_expr(r_bp)?; Some(self.bin(span, left, crate::ast::BinOp::Gt,     r)) }
            TokenKind::LtEq       => { self.advance(); let r = self.pratt_expr(r_bp)?; Some(self.bin(span, left, crate::ast::BinOp::LtEq,   r)) }
            TokenKind::GtEq       => { self.advance(); let r = self.pratt_expr(r_bp)?; Some(self.bin(span, left, crate::ast::BinOp::GtEq,   r)) }
            TokenKind::DotDot     => { self.advance(); let r = self.pratt_expr(r_bp)?; Some(self.bin(span, left, crate::ast::BinOp::Range,  r)) }
            TokenKind::DotDotEq   => { self.advance(); let r = self.pratt_expr(r_bp)?; Some(self.bin(span, left, crate::ast::BinOp::RangeInclusive, r)) }
            TokenKind::Plus       => { self.advance(); let r = self.pratt_expr(r_bp)?; Some(self.bin(span, left, crate::ast::BinOp::Add,    r)) }
            TokenKind::Minus      => { self.advance(); let r = self.pratt_expr(r_bp)?; Some(self.bin(span, left, crate::ast::BinOp::Sub,    r)) }
            TokenKind::Star       => { self.advance(); let r = self.pratt_expr(r_bp)?; Some(self.bin(span, left, crate::ast::BinOp::Mul,    r)) }
            TokenKind::Slash      => { self.advance(); let r = self.pratt_expr(r_bp)?; Some(self.bin(span, left, crate::ast::BinOp::Div,    r)) }
            TokenKind::Percent    => { self.advance(); let r = self.pratt_expr(r_bp)?; Some(self.bin(span, left, crate::ast::BinOp::Mod,    r)) }

            // ── Field access: expr.field  or  expr.method(args) ─
            TokenKind::Dot => {
                self.advance();
                let name = self.parse_ident("field or method name")?;
                if self.at(&TokenKind::LParen) {
                    self.advance();
                    let args = self.parse_call_args();
                    self.expect(&TokenKind::RParen)?;
                    Some(Expr::MethodCall(Box::new(crate::ast::MethodCallExpr {
                        span, receiver: Box::new(left), method: name,
                        generics: vec![], args,
                    })))
                } else {
                    Some(Expr::FieldAccess(Box::new(FieldAccessExpr {
                        span, object: Box::new(left), field: name,
                    })))
                }
            }

            // ── Function call: expr(args) ──────────────────────
            TokenKind::LParen => {
                self.advance();
                let args = self.parse_call_args();
                self.expect(&TokenKind::RParen)?;
                // Callee must be an ident for CallExpr
                if let Expr::Ident(id) = left {
                    Some(Expr::Call(Box::new(crate::ast::CallExpr {
                        span, callee: id, generics: vec![], args,
                    })))
                } else {
                    // Method-style fallback — wrap as None for now
                    Some(Expr::Lit(Literal::None(span)))
                }
            }

            // ── Index: expr[idx] ───────────────────────────────
            TokenKind::LBracket => {
                self.advance();
                let idx = self.pratt_expr(0)?;
                self.expect(&TokenKind::RBracket)?;
                Some(Expr::Index(Box::new(crate::ast::IndexExpr {
                    span, object: Box::new(left), index: Box::new(idx),
                })))
            }

            // ── ? propagation ──────────────────────────────────
            TokenKind::Question => {
                self.advance();
                Some(Expr::Propagate(Box::new(left), span))
            }

            // ── ! capability pin ───────────────────────────────
            TokenKind::Bang => {
                self.advance();
                let method = self.parse_ident("capability method")?;
                if self.at(&TokenKind::LParen) {
                    self.advance();
                    let args = self.parse_call_args();
                    self.expect(&TokenKind::RParen)?;
                    Some(Expr::CapPin(Box::new(crate::ast::CapPinExpr {
                        span,
                        receiver   : left,
                        method,
                        args,
                        infallible : false,
                    })))
                } else {
                    Some(Expr::Lit(Literal::None(span)))
                }
            }

            _ => Some(left),
        }
    }

    /// Helper — build a BinOpExpr
    fn bin(&self, span: Span, left: Expr, op: crate::ast::BinOp, right: Expr) -> Expr {
        Expr::BinOp(Box::new(crate::ast::BinOpExpr { span, op, lhs: left, rhs: right }))
    }

    /// Parse a pipe expression: left |> fn(args) [as Contract]
    fn parse_pipe_expr(&mut self, head: Expr) -> Option<Expr> {
        let span = self.current_span();
        let mut stages = Vec::new();

        while self.at(&TokenKind::PipeForward) {
            let stage_span = self.current_span();
            self.advance(); // consume |>

            // Parse the pipe stage call
            let call_expr = self.parse_nud()?;
            let call = match call_expr {
                Expr::Call(c)       => crate::ast::PipeCall::FnCall(*c),
                Expr::MethodCall(m) => crate::ast::PipeCall::MethodCall(*m),
                _ => return None,
            };

            // Optional: as Contract
            let contract = if self.at(&TokenKind::As) {
                self.advance();
                self.parse_ident("contract type")
            } else { None };

            stages.push(crate::ast::PipeStage {
                span: stage_span, call, contract,
            });
        }

        Some(Expr::Pipe(Box::new(crate::ast::PipeExpr { span, head, stages })))
    }

    fn parse_call_args(&mut self) -> Vec<crate::ast::Arg> {
        let mut args = Vec::new();
        while !self.at_eof() && !self.at(&TokenKind::RParen)
            && !self.at(&TokenKind::RBracket) {
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
            let value = match self.pratt_expr(0) {
                Some(e) => e,
                None => break,
            };
            args.push(crate::ast::Arg { span, label, value });
            if self.eat(&TokenKind::Comma).is_none() { break; }
        }
        args
    }// ── Type parsing ──────────────────────────────────────────

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

    // ── P2-11: Expression tests ───────────────────────────────

    #[test] fn test_expr_addition() {
        let src = "fn f():
    let x = 1 + 2
";
        let r   = crate::parse(src, file());
        assert!(r.errors.is_empty(), "{:?}", r.errors);
        match &r.program.items[0] {
            TopLevelItem::Fn(f) => {
                match &f.body.stmts[0] {
                    Stmt::Let(s) => assert!(matches!(s.init, Expr::BinOp(_))),
                    other => panic!("{:?}", other),
                }
            }
            other => panic!("{:?}", other),
        }
    }

    #[test] fn test_expr_precedence_mul_over_add() {
        // 2 + 3 * 4 should parse as 2 + (3 * 4)
        let src = "fn f():
    let x = 2 + 3 * 4
";
        let r   = crate::parse(src, file());
        assert!(r.errors.is_empty(), "{:?}", r.errors);
        match &r.program.items[0] {
            TopLevelItem::Fn(f) => {
                match &f.body.stmts[0] {
                    Stmt::Let(s) => {
                        match &s.init {
                            Expr::BinOp(b) => {
                                // Outer op must be Add
                                assert_eq!(b.op, crate::ast::BinOp::Add);
                                // Right side must be Mul
                                assert!(matches!(b.rhs, Expr::BinOp(_)));
                            }
                            other => panic!("expected BinOp, got {:?}", other),
                        }
                    }
                    other => panic!("{:?}", other),
                }
            }
            other => panic!("{:?}", other),
        }
    }

    #[test] fn test_expr_comparison() {
        let src = "fn f():
    let ok = x == 42
";
        let r   = crate::parse(src, file());
        assert!(r.errors.is_empty(), "{:?}", r.errors);
        match &r.program.items[0] {
            TopLevelItem::Fn(f) => {
                match &f.body.stmts[0] {
                    Stmt::Let(s) => {
                        match &s.init {
                            Expr::BinOp(b) => assert_eq!(b.op, crate::ast::BinOp::Eq),
                            other => panic!("{:?}", other),
                        }
                    }
                    other => panic!("{:?}", other),
                }
            }
            other => panic!("{:?}", other),
        }
    }

    #[test] fn test_expr_unary_neg() {
        let src = "fn f():
    let x = -42
";
        let r   = crate::parse(src, file());
        assert!(r.errors.is_empty(), "{:?}", r.errors);
        match &r.program.items[0] {
            TopLevelItem::Fn(f) => {
                match &f.body.stmts[0] {
                    Stmt::Let(s) => assert!(matches!(s.init, Expr::UnaryOp(_))),
                    other => panic!("{:?}", other),
                }
            }
            other => panic!("{:?}", other),
        }
    }

    #[test] fn test_expr_function_call() {
        let src = "fn f():
    let x = add(1, 2)
";
        let r   = crate::parse(src, file());
        assert!(r.errors.is_empty(), "{:?}", r.errors);
        match &r.program.items[0] {
            TopLevelItem::Fn(f) => {
                match &f.body.stmts[0] {
                    Stmt::Let(s) => {
                        match &s.init {
                            Expr::Call(c) => {
                                assert_eq!(c.callee.name, "add");
                                assert_eq!(c.args.len(), 2);
                            }
                            other => panic!("{:?}", other),
                        }
                    }
                    other => panic!("{:?}", other),
                }
            }
            other => panic!("{:?}", other),
        }
    }

    #[test] fn test_expr_method_call() {
        let src = "fn f():
    let n = items.len()
";
        let r   = crate::parse(src, file());
        assert!(r.errors.is_empty(), "{:?}", r.errors);
        match &r.program.items[0] {
            TopLevelItem::Fn(f) => {
                match &f.body.stmts[0] {
                    Stmt::Let(s) => assert!(matches!(s.init, Expr::MethodCall(_))),
                    other => panic!("{:?}", other),
                }
            }
            other => panic!("{:?}", other),
        }
    }

    #[test] fn test_expr_field_access() {
        let src = "fn f():
    let h = config.host
";
        let r   = crate::parse(src, file());
        assert!(r.errors.is_empty(), "{:?}", r.errors);
        match &r.program.items[0] {
            TopLevelItem::Fn(f) => {
                match &f.body.stmts[0] {
                    Stmt::Let(s) => {
                        match &s.init {
                            Expr::FieldAccess(fa) => assert_eq!(fa.field.name, "host"),
                            other => panic!("{:?}", other),
                        }
                    }
                    other => panic!("{:?}", other),
                }
            }
            other => panic!("{:?}", other),
        }
    }

    #[test] fn test_expr_chained_field_access() {
        // config.network.host should parse left to right
        let src = "fn f():
    let h = config.network.host
";
        let r   = crate::parse(src, file());
        assert!(r.errors.is_empty(), "{:?}", r.errors);
        match &r.program.items[0] {
            TopLevelItem::Fn(f) => {
                match &f.body.stmts[0] {
                    Stmt::Let(s) => {
                        // outer: .host  inner: config.network
                        match &s.init {
                            Expr::FieldAccess(fa) => {
                                assert_eq!(fa.field.name, "host");
                                assert!(matches!(*fa.object, Expr::FieldAccess(_)));
                            }
                            other => panic!("{:?}", other),
                        }
                    }
                    other => panic!("{:?}", other),
                }
            }
            other => panic!("{:?}", other),
        }
    }

    #[test] fn test_expr_index() {
        let src = "fn f():
    let x = items[0]
";
        let r   = crate::parse(src, file());
        assert!(r.errors.is_empty(), "{:?}", r.errors);
        match &r.program.items[0] {
            TopLevelItem::Fn(f) => {
                match &f.body.stmts[0] {
                    Stmt::Let(s) => assert!(matches!(s.init, Expr::Index(_))),
                    other => panic!("{:?}", other),
                }
            }
            other => panic!("{:?}", other),
        }
    }

    #[test] fn test_expr_propagate() {
        let src = "fn f():
    let x = open(path)?
";
        let r   = crate::parse(src, file());
        assert!(r.errors.is_empty(), "{:?}", r.errors);
        match &r.program.items[0] {
            TopLevelItem::Fn(f) => {
                match &f.body.stmts[0] {
                    Stmt::Let(s) => assert!(matches!(s.init, Expr::Propagate(_, _))),
                    other => panic!("{:?}", other),
                }
            }
            other => panic!("{:?}", other),
        }
    }

    #[test] fn test_expr_list_literal() {
        let src = "fn f():
    let xs = [1, 2, 3]
";
        let r   = crate::parse(src, file());
        assert!(r.errors.is_empty(), "{:?}", r.errors);
        match &r.program.items[0] {
            TopLevelItem::Fn(f) => {
                match &f.body.stmts[0] {
                    Stmt::Let(s) => {
                        match &s.init {
                            Expr::List(l) => assert_eq!(l.elements.len(), 3),
                            other => panic!("{:?}", other),
                        }
                    }
                    other => panic!("{:?}", other),
                }
            }
            other => panic!("{:?}", other),
        }
    }

    #[test] fn test_expr_grouped() {
        // (1 + 2) * 3 — parens force addition first
        let src = "fn f():
    let x = (1 + 2) * 3
";
        let r   = crate::parse(src, file());
        assert!(r.errors.is_empty(), "{:?}", r.errors);
        match &r.program.items[0] {
            TopLevelItem::Fn(f) => {
                match &f.body.stmts[0] {
                    Stmt::Let(s) => {
                        match &s.init {
                            Expr::BinOp(b) => {
                                assert_eq!(b.op, crate::ast::BinOp::Mul);
                                // Left side must be Add (from parens)
                                assert!(matches!(b.lhs, Expr::BinOp(_)));
                            }
                            other => panic!("{:?}", other),
                        }
                    }
                    other => panic!("{:?}", other),
                }
            }
            other => panic!("{:?}", other),
        }
    }

    #[test] fn test_expr_complex_condition() {
        // x > 0 and y < 10
        let src = "fn f():
    if x > 0 and y < 10:
        pass
";
        let r   = crate::parse(src, file());
        assert!(r.errors.is_empty(), "{:?}", r.errors);
        match &r.program.items[0] {
            TopLevelItem::Fn(f) => {
                match &f.body.stmts[0] {
                    Stmt::If(s) => assert!(matches!(s.condition, Expr::BinOp(_))),
                    other => panic!("{:?}", other),
                }
            }
            other => panic!("{:?}", other),
        }
    }

    // ── P2-12: Match statement tests ─────────────────────────

    #[test] fn test_parse_match_basic() {
        let src = concat!(
            "fn f():
",
            "    match level:
",
            "        Clear => 0
",
            "        _ => 1
",
        );
        let r = crate::parse(src, file());
        assert!(r.errors.is_empty(), "{:?}", r.errors);
        match &r.program.items[0] {
            TopLevelItem::Fn(f) => {
                match &f.body.stmts[0] {
                    Stmt::Match(m) => {
                        assert_eq!(m.arms.len(), 2);
                    }
                    other => panic!("expected Match, got {:?}", other),
                }
            }
            other => panic!("{:?}", other),
        }
    }

    #[test] fn test_parse_match_wildcard() {
        let src = concat!(
            "fn f():
",
            "    match x:
",
            "        _ => 0
",
        );
        let r = crate::parse(src, file());
        assert!(r.errors.is_empty(), "{:?}", r.errors);
        match &r.program.items[0] {
            TopLevelItem::Fn(f) => {
                match &f.body.stmts[0] {
                    Stmt::Match(m) => {
                        assert!(matches!(m.arms[0].pattern, Pattern::Wildcard(_)));
                    }
                    other => panic!("{:?}", other),
                }
            }
            other => panic!("{:?}", other),
        }
    }

    #[test] fn test_parse_match_binding() {
        let src = concat!(
            "fn f():
",
            "    match x:
",
            "        value => value
",
        );
        let r = crate::parse(src, file());
        assert!(r.errors.is_empty(), "{:?}", r.errors);
        match &r.program.items[0] {
            TopLevelItem::Fn(f) => {
                match &f.body.stmts[0] {
                    Stmt::Match(m) => {
                        assert!(matches!(m.arms[0].pattern, Pattern::Binding(_)));
                    }
                    other => panic!("{:?}", other),
                }
            }
            other => panic!("{:?}", other),
        }
    }

    #[test] fn test_parse_match_enum_variant_unit() {
        let src = concat!(
            "fn f():
",
            "    match level:
",
            "        Clear => 0
",
            "        _ => 1
",
        );
        let r = crate::parse(src, file());
        assert!(r.errors.is_empty(), "{:?}", r.errors);
        match &r.program.items[0] {
            TopLevelItem::Fn(f) => {
                match &f.body.stmts[0] {
                    Stmt::Match(m) => {
                        match &m.arms[0].pattern {
                            Pattern::Enum(e) => {
                                assert_eq!(e.name.name, "Clear");
                                assert_eq!(e.fields.len(), 0);
                            }
                            other => panic!("{:?}", other),
                        }
                    }
                    other => panic!("{:?}", other),
                }
            }
            other => panic!("{:?}", other),
        }
    }

    #[test] fn test_parse_match_enum_variant_with_fields() {
        let src = concat!(
            "fn f():
",
            "    match level:
",
            "        Advisory(detail) => detail
",
            "        _ => 0
",
        );
        let r = crate::parse(src, file());
        assert!(r.errors.is_empty(), "{:?}", r.errors);
        match &r.program.items[0] {
            TopLevelItem::Fn(f) => {
                match &f.body.stmts[0] {
                    Stmt::Match(m) => {
                        match &m.arms[0].pattern {
                            Pattern::Enum(e) => {
                                assert_eq!(e.name.name, "Advisory");
                                assert_eq!(e.fields.len(), 1);
                            }
                            other => panic!("{:?}", other),
                        }
                    }
                    other => panic!("{:?}", other),
                }
            }
            other => panic!("{:?}", other),
        }
    }

    #[test] fn test_parse_match_literal_pattern() {
        let src = concat!(
            "fn f():
",
            "    match x:
",
            "        0 => false
",
            "        1 => true
",
            "        _ => false
",
        );
        let r = crate::parse(src, file());
        assert!(r.errors.is_empty(), "{:?}", r.errors);
        match &r.program.items[0] {
            TopLevelItem::Fn(f) => {
                match &f.body.stmts[0] {
                    Stmt::Match(m) => {
                        assert_eq!(m.arms.len(), 3);
                        assert!(matches!(m.arms[0].pattern, Pattern::Literal(_)));
                    }
                    other => panic!("{:?}", other),
                }
            }
            other => panic!("{:?}", other),
        }
    }

    #[test] fn test_parse_match_guard() {
        let src = concat!(
            "fn f():
",
            "    match x:
",
            "        n if n > 0 => true
",
            "        _ => false
",
        );
        let r = crate::parse(src, file());
        assert!(r.errors.is_empty(), "{:?}", r.errors);
        match &r.program.items[0] {
            TopLevelItem::Fn(f) => {
                match &f.body.stmts[0] {
                    Stmt::Match(m) => {
                        assert!(m.arms[0].guard.is_some());
                    }
                    other => panic!("{:?}", other),
                }
            }
            other => panic!("{:?}", other),
        }
    }

    #[test] fn test_parse_match_or_pattern() {
        let src = concat!(
            "fn f():
",
            "    match x:
",
            "        A | B => 1
",
            "        _ => 0
",
        );
        let r = crate::parse(src, file());
        assert!(r.errors.is_empty(), "{:?}", r.errors);
        match &r.program.items[0] {
            TopLevelItem::Fn(f) => {
                match &f.body.stmts[0] {
                    Stmt::Match(m) => {
                        assert!(matches!(m.arms[0].pattern, Pattern::Or(_, _)));
                    }
                    other => panic!("{:?}", other),
                }
            }
            other => panic!("{:?}", other),
        }
    }

    #[test] fn test_parse_full_threat_level_match() {
        // The actual Aegis Monitor pattern
        let src = concat!(
            "fn classify(signal : Int) -> Int:
",
            "    match signal:
",
            "        Clear => 0
",
            "        Advisory(detail) => 1
",
            "        Critical(layer, detail) => 2
",
            "        _ => 0
",
        );
        let r = crate::parse(src, file());
        assert!(r.errors.is_empty(), "{:?}", r.errors);
        match &r.program.items[0] {
            TopLevelItem::Fn(f) => {
                match &f.body.stmts[0] {
                    Stmt::Match(m) => {
                        assert_eq!(m.arms.len(), 4);
                        // Clear — unit variant
                        assert!(matches!(m.arms[0].pattern, Pattern::Enum(_)));
                        // Advisory(detail) — variant with 1 field
                        assert!(matches!(m.arms[1].pattern, Pattern::Enum(_)));
                        // Critical(layer, detail) — variant with 2 fields
                        match &m.arms[2].pattern {
                            Pattern::Enum(e) => assert_eq!(e.fields.len(), 2),
                            other => panic!("{:?}", other),
                        }
                        // _ — wildcard
                        assert!(matches!(m.arms[3].pattern, Pattern::Wildcard(_)));
                    }
                    other => panic!("expected Match, got {:?}", other),
                }
            }
            other => panic!("{:?}", other),
        }
    }

    // ══════════════════════════════════════════════════════════
    // P2-19: INTEGRATION TEST — AEGIS MONITOR
    // The finish line for Phase 2.
    // When this passes — AXON stops being documents
    // and becomes a language.
    // ══════════════════════════════════════════════════════════

    const AEGIS_MONITOR: &str = concat!(
        "module aieonyx.aegis.monitor\n",
        "\n",
        "import axon.sys.sel4.ipc as ipc\n",
        "import axon.mesh.collective as collective\n",
        "\n",
        "enum ThreatLevel:\n",
        "    Clear\n",
        "    Advisory(detail: Str)\n",
        "    Critical(layer: Int, detail: Str)\n",
        "\n",
        "struct Signal:\n",
        "    severity : Int\n",
        "    message  : Str\n",
        "    layer    : Int\n",
        "\n",
        "fn classify(signal : Signal) -> ThreatLevel:\n",
        "    match signal.severity:\n",
        "        0 => return ThreatLevel.Clear\n",
        "        1 => return ThreatLevel.Advisory(signal.message)\n",
        "        _ => return ThreatLevel.Critical(signal.layer, signal.message)\n",
        "\n",
        "task monitor() uses [ipc.read, collective.emit]:\n",
        "    let@ channel = ipc.open_channel()?\n",
        "    defer channel.close()\n",
        "    for signal in channel.signals:\n",
        "        let level = classify(signal)\n",
        "        collective.emit(level)\n",
    );

    #[test]
    fn p2_19_aegis_monitor_zero_errors() {
        let r = crate::parse(AEGIS_MONITOR, file());

        // ── THE GATE ─────────────────────────────────────────
        // Zero parse errors. No exceptions. No excuses.
        assert!(
            r.errors.is_empty(),
            "Aegis Monitor produced {} parse error(s):\n{}",
            r.errors.len(),
            r.errors.iter().enumerate()
                .map(|(i, e)| format!("  [{}] {:?}", i+1, e))
                .collect::<Vec<_>>()
                .join("\n")
        );

        // ── Module ────────────────────────────────────────────
        let module = r.program.module.as_ref()
            .expect("expected module declaration");
        assert_eq!(module.path[0].name, "aieonyx",   "module path[0]");
        assert_eq!(module.path[1].name, "aegis",     "module path[1]");
        assert_eq!(module.path[2].name, "monitor",   "module path[2]");

        // ── Imports ───────────────────────────────────────────
        assert_eq!(r.program.imports.len(), 2, "expected 2 imports");
        assert_eq!(r.program.imports[0].alias.as_ref().unwrap().name, "ipc");
        assert_eq!(r.program.imports[1].alias.as_ref().unwrap().name, "collective");

        // ── Top-level items ───────────────────────────────────
        // ThreatLevel enum, Signal struct, classify fn, monitor task
        assert_eq!(r.program.items.len(), 4,
            "expected 4 top-level items (enum, struct, fn, task), got {}",
            r.program.items.len());

        // ── ThreatLevel enum ──────────────────────────────────
        match &r.program.items[0] {
            TopLevelItem::Enum(e) => {
                assert_eq!(e.name.name, "ThreatLevel");
                assert_eq!(e.variants.len(), 3, "ThreatLevel should have 3 variants");
                assert_eq!(e.variants[0].name.name, "Clear");
                assert_eq!(e.variants[1].name.name, "Advisory");
                assert_eq!(e.variants[1].fields.len(), 1);
                assert_eq!(e.variants[2].name.name, "Critical");
                assert_eq!(e.variants[2].fields.len(), 2);
            }
            other => panic!("item[0] should be ThreatLevel enum, got {:?}", other),
        }

        // ── Signal struct ─────────────────────────────────────
        match &r.program.items[1] {
            TopLevelItem::Struct(s) => {
                assert_eq!(s.name.name, "Signal");
                assert_eq!(s.fields.len(), 3, "Signal should have 3 fields");
                assert_eq!(s.fields[0].name.name, "severity");
                assert_eq!(s.fields[1].name.name, "message");
                assert_eq!(s.fields[2].name.name, "layer");
            }
            other => panic!("item[1] should be Signal struct, got {:?}", other),
        }

        // ── classify function ─────────────────────────────────
        match &r.program.items[2] {
            TopLevelItem::Fn(f) => {
                assert_eq!(f.name.name, "classify");
                assert_eq!(f.params.len(), 1);
                assert_eq!(f.params[0].name.name, "signal");
                assert!(f.ret_type.is_some(), "classify should have return type");
                assert_eq!(f.body.stmts.len(), 1,
                    "classify body should have 1 match statement");
                assert!(matches!(f.body.stmts[0], Stmt::Match(_)),
                    "classify body should be a match statement");
            }
            other => panic!("item[2] should be classify fn, got {:?}", other),
        }

        // ── monitor task ──────────────────────────────────────
        match &r.program.items[3] {
            TopLevelItem::Task(t) => {
                assert_eq!(t.name.name, "monitor");
                assert!(t.uses.is_some(), "monitor should have uses clause");
                let uses = t.uses.as_ref().unwrap();
                assert_eq!(uses.effects.len(), 2,
                    "monitor should declare 2 effects");
                assert_eq!(uses.effects[0].parts[0].name, "ipc");
                assert_eq!(uses.effects[1].parts[0].name, "collective");
                // Body: let@, defer, for
                assert!(t.body.stmts.len() >= 3,
                    "monitor body should have at least 3 statements");
                assert!(matches!(t.body.stmts[0], Stmt::Ephemeral(_)),
                    "first stmt should be let@ (Ephemeral)");
                assert!(matches!(t.body.stmts[1], Stmt::Defer(_)),
                    "second stmt should be defer");
                assert!(matches!(t.body.stmts[2], Stmt::For(_)),
                    "third stmt should be for");
            }
            other => panic!("item[3] should be monitor task, got {:?}", other),
        }

        // ── PHASE 2 COMPLETE ──────────────────────────────────
        // If you are reading this assertion — Phase 2 is done.
        // AXON is no longer just documents.
        // AXON is a language.
        println!("\n  ✓ Aegis Monitor parsed with zero errors.");
        println!("  ✓ Module: aieonyx.aegis.monitor");
        println!("  ✓ 2 imports verified");
        println!("  ✓ ThreatLevel enum — 3 variants");
        println!("  ✓ Signal struct — 3 fields");
        println!("  ✓ classify fn — match statement body");
        println!("  ✓ monitor task — let@, defer, for");
        println!("\n  PHASE 2 COMPLETE. AXON IS A LANGUAGE.\n");
    }
}