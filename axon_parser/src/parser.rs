// ============================================================
// AXON Parser — parser.rs
// P2-08 complete — struct, enum, module, import parsing
// Copyright © 2026 Edison Lepiten — AIEONYX
// github.com/aieonyx/axon
// ============================================================

use axon_lexer::{FileId, Span, Token, TokenKind};
use crate::ast::{
    Program, TopLevelItem,
    ModuleDecl, ImportDecl,
    StructDecl, FieldDecl,
    EnumDecl, VariantDecl,
    ProgramIntent,
    Type, PrimitiveType,
    Ident,
};
use crate::error::ParseError;

// ── Parser struct ─────────────────────────────────────────────

pub struct Parser<'src> {
    tokens  : Vec<Token>,
    pos     : usize,
    source  : &'src str,
    file_id : FileId,
    pub errors  : Vec<ParseError>,
}

impl<'src> Parser<'src> {
    pub fn new(tokens: Vec<Token>, source: &'src str, file_id: FileId) -> Self {
        Parser { tokens, pos: 0, source, file_id, errors: Vec::new() }
    }

    pub fn into_errors(self) -> Vec<ParseError> { self.errors }

    // ── Span helper ───────────────────────────────────────────
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
            self.errors.push(ParseError::Custom { message: message.into(), span, hint: None });
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

    // ── P2-07: Module / Import ────────────────────────────────

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
                _ => { self.error(self.current_span(), "expected identifier after 'as'"); None }
            }
        } else { None };
        self.eat(&TokenKind::Newline);
        Some(ImportDecl { span, path, alias })
    }

    fn parse_dotted_path(&mut self) -> Vec<Ident> {
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
                _ => { self.error(self.current_span(), "expected identifier after '.'"); break; }
            }
        }
        parts
    }

    // ── P2-08: Struct / Enum ──────────────────────────────────

    fn parse_struct_decl(&mut self) -> Option<StructDecl> {
        let span = self.current_span();
        self.expect(&TokenKind::Struct)?;
        let name = self.parse_ident("struct name")?;
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
        let span = self.current_span();
        self.expect(&TokenKind::Enum)?;
        let name = self.parse_ident("enum name")?;
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
                    let fname = match self.parse_ident("field name") { Some(n) => n, None => break };
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
                let name = name.clone();
                self.advance();
                if self.at(&TokenKind::Lt) {
                    self.advance();
                    let inner = self.parse_type()?;
                    self.expect(&TokenKind::Gt)?;
                    let id = Ident::new(name, span);
                    Some(Type::Generic(id, vec![inner], span))
                } else {
                    let id = Ident::new(name, span);
                    Some(Type::Named(vec![id]))
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
            let item = match self.peek_kind() {
                TokenKind::Struct => self.parse_struct_decl().map(TopLevelItem::Struct),
                TokenKind::Enum   => self.parse_enum_decl().map(TopLevelItem::Enum),
                _ => { self.advance(); None }
            };
            if let Some(i) = item { items.push(i); }
        }
        let end = self.current_span();
        Program {
            span           : Self::merge(start, end),
            program_intent,
            module,
            imports,
            items,
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
        let tokens = lex(source, file());
        Parser::new(tokens, source, file())
    }

    #[test]
    fn test_parse_empty_source() {
        let result = crate::parse("", file());
        assert!(result.is_ok());
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_parse_result_struct() {
        let result = crate::parse("", file());
        assert!(result.program.imports.is_empty());
        assert!(result.program.items.is_empty());
    }

    #[test]
    fn test_ast_program_has_program_intent_field() {
        let result = crate::parse("", file());
        assert!(result.program.program_intent.is_none());
    }

    #[test]
    fn test_ast_program_has_all_fields() {
        let result = crate::parse("", file());
        let _m = &result.program.module;
        let _i = &result.program.imports;
        let _t = &result.program.items;
        let _p = &result.program.program_intent;
    }

    #[test]
    fn test_stmt_enum_has_defer_variant() {
        use crate::ast::{Stmt, DeferStmt, Expr, Literal};
        let span = Span::new(file(), 0, 0, 1, 1);
        let lit  = Literal::Bool(true, span);
        let expr = Expr::Lit(lit);
        let ds   = DeferStmt { span, expr };
        let _s   = Stmt::Defer(ds);
    }

    #[test]
    fn test_stmt_enum_has_with_variant() {
        use crate::ast::{Stmt, WithStmt, Expr, Literal, Ident, Block};
        let span = Span::new(file(), 0, 0, 1, 1);
        let lit  = Literal::Bool(true, span);
        let expr = Expr::Lit(lit);
        let b    = Ident::new("f", span);
        let body = Block { span, stmts: vec![] };
        let ws   = WithStmt { span, expr, binding: b, body };
        let _s   = Stmt::With(ws);
    }

    #[test] fn test_ast_intent_modes_all_exist() {
        let _a = IntentMode::Secure; let _b = IntentMode::Performant;
        let _c = IntentMode::Auditable; let _d = IntentMode::Verifiable;
        let _e = IntentMode::MinimalRuntime;
    }

    #[test] fn test_ast_mem_modes_all_exist() {
        let _a = MemMode::Own; let _b = MemMode::Borrow;
        let _c = MemMode::MutBorrow; let _d = MemMode::Share;
    }

    #[test] fn test_ast_provenance_kinds_exist() {
        let _a = ProvenanceKind::Tainted; let _b = ProvenanceKind::Clean;
        let _c = ProvenanceKind::Network; let _d = ProvenanceKind::FileSystem;
        let _e = ProvenanceKind::UserInput; let _f = ProvenanceKind::Trusted;
        let _g = ProvenanceKind::Unknown;
    }

    #[test] fn test_ast_temporal_expr_variants_exist() {
        let span = Span::new(file(), 0, 0, 1, 1);
        let _a = TemporalExpr::Now(span); let _b = TemporalExpr::Lifetime(span);
        let _c = TemporalExpr::Epoch(span);
        let _d = TemporalOp::Add; let _e = TemporalOp::Sub;
    }

    #[test] fn test_ast_actor_decl_exists() {
        let span = Span::new(file(), 0, 0, 1, 1);
        let name = Ident::new("Worker", span);
        let a    = ActorDecl { span, name, items: vec![] };
        assert_eq!(a.items.len(), 0);
    }

    #[test] fn test_ast_opaque_type_decl_exists() {
        let span   = Span::new(file(), 0, 0, 1, 1);
        let name   = Ident::new("UserId", span);
        let ty     = Type::Primitive(PrimitiveType::Int, span);
        let opaque = OpaqueTypeDecl { span, name, ty };
        assert_eq!(opaque.name.name, "UserId");
    }

    #[test] fn test_ast_bin_op_binding_power_ordering() {
        let (ml,_) = BinOp::Mul.binding_power();
        let (al,_) = BinOp::Add.binding_power();
        let (cl,_) = BinOp::Eq.binding_power();
        let (al2,_) = BinOp::And.binding_power();
        let (ol,_) = BinOp::Or.binding_power();
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
        let a = p.peek().kind.clone();
        let b = p.peek().kind.clone();
        assert_eq!(a, b);
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

    #[test] fn test_cursor_at_eof() {
        let p = make_parser("");
        assert!(p.at_eof());
    }

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
        assert_eq!(m.path[0].name, "aieonyx");
        assert_eq!(m.path[1].name, "aegis");
        assert_eq!(m.path[2].name, "monitor");
    }

    #[test] fn test_parse_import_simple() {
        let r = crate::parse("module hello\nimport axon.sys\n", file());
        assert!(r.errors.is_empty(), "{:?}", r.errors);
        assert_eq!(r.program.imports.len(), 1);
        assert_eq!(r.program.imports[0].path[0].name, "axon");
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
        assert_eq!(r.program.items.len(), 1);
        match &r.program.items[0] {
            TopLevelItem::Struct(s) => {
                assert_eq!(s.name.name, "Config");
                assert_eq!(s.fields.len(), 2);
                assert_eq!(s.fields[0].name.name, "host");
                assert_eq!(s.fields[1].name.name, "port");
            }
            other => panic!("expected Struct, got {:?}", other),
        }
    }

    #[test] fn test_parse_enum_no_fields() {
        let src = "enum Status:\n    Active\n    Inactive\n";
        let r   = crate::parse(src, file());
        assert!(r.errors.is_empty(), "{:?}", r.errors);
        match &r.program.items[0] {
            TopLevelItem::Enum(e) => {
                assert_eq!(e.name.name, "Status");
                assert_eq!(e.variants.len(), 2);
            }
            other => panic!("expected Enum, got {:?}", other),
        }
    }

    #[test] fn test_parse_enum_with_fields() {
        let src = "enum ThreatLevel:\n    Clear\n    Advisory(detail: Str)\n    Critical(layer: Int, detail: Str)\n";
        let r   = crate::parse(src, file());
        assert!(r.errors.is_empty(), "{:?}", r.errors);
        match &r.program.items[0] {
            TopLevelItem::Enum(e) => {
                assert_eq!(e.variants.len(), 3);
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
            "enum Color:\n",   "    Red\n",      "    Green\n", "    Blue\n",
        );
        let r = crate::parse(src, file());
        assert!(r.errors.is_empty(), "{:?}", r.errors);
        assert_eq!(r.program.items.len(), 2);
    }
}
