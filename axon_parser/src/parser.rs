// ============================================================
// AXON Parser — parser.rs
// Parser struct + cursor methods — P2-06
// Copyright © 2026 Edison Lepiten — AIEONYX
// github.com/aieonyx/axon
//
// This file implements the Parser struct and all cursor methods.
// Grammar rule implementations begin in P2-07.
//
// Cursor methods:
//   peek()        — look at current token without consuming
//   peek_kind()   — look at current token kind
//   advance()     — consume and return current token
//   eat()         — consume if kind matches, else None
//   expect()      — consume if kind matches, else error
//   at()          — true if current token matches kind
//   at_eof()      — true if at end of file
//   skip_newlines() — skip over any Newline tokens
//   current_span()  — span of current token
// ============================================================

use axon_lexer::{FileId, Span, Token, TokenKind};
use crate::ast::Program;
use crate::error::ParseError;

// ── Parser struct ─────────────────────────────────────────────

pub struct Parser<'src> {
    tokens  : Vec<Token>,
    pos     : usize,
    source  : &'src str,
    file_id : FileId,
    errors  : Vec<ParseError>,
}

impl<'src> Parser<'src> {
    /// Create a new parser from a token stream.
    pub fn new(tokens: Vec<Token>, source: &'src str, file_id: FileId) -> Self {
        Parser {
            tokens,
            pos     : 0,
            source,
            file_id,
            errors  : Vec::new(),
        }
    }

    /// Consume collected errors — called after parse_program().
    pub fn into_errors(self) -> Vec<ParseError> {
        self.errors
    }

    // ── Cursor methods ─────────────────────────────────────────

    /// Return a reference to the current token without consuming it.
    pub fn peek(&self) -> &Token {
        // Find the next non-comment token
        let mut i = self.pos;
        while i < self.tokens.len() {
            match &self.tokens[i].kind {
                TokenKind::Comment(_) | TokenKind::DocComment(_) => i += 1,
                _ => return &self.tokens[i],
            }
        }
        // Return last token (Eof) if past end
        self.tokens.last().expect("token stream must end with Eof")
    }

    /// Return the kind of the current token.
    pub fn peek_kind(&self) -> &TokenKind {
        &self.peek().kind
    }

    /// Return the current token's span.
    pub fn current_span(&self) -> Span {
        self.peek().span
    }

    /// Advance past the current token and return it.
    pub fn advance(&mut self) -> Token {
        // Skip comments
        while self.pos < self.tokens.len() {
            match &self.tokens[self.pos].kind {
                TokenKind::Comment(_) | TokenKind::DocComment(_) => {
                    self.pos += 1;
                }
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

    /// If the current token matches `kind` — consume and return it.
    /// Otherwise return None without consuming.
    pub fn eat(&mut self, kind: &TokenKind) -> Option<Token> {
        if self.at(kind) {
            Some(self.advance())
        } else {
            None
        }
    }

    /// If the current token matches `kind` — consume and return it.
    /// Otherwise record an error and return None.
    pub fn expect(&mut self, kind: &TokenKind) -> Option<Token> {
        if self.at(kind) {
            Some(self.advance())
        } else {
            let span    = self.current_span();
            let found   = self.peek_kind().display_name().to_string();
            let expected = kind.display_name().to_string();
            self.error(
                span,
                format!("expected {} but found {}", expected, found),
            );
            None
        }
    }

    /// True if the current token's kind matches.
    pub fn at(&self, kind: &TokenKind) -> bool {
        self.peek_kind_matches(kind)
    }

    /// True if we have reached end of file.
    pub fn at_eof(&self) -> bool {
        matches!(self.peek_kind(), TokenKind::Eof)
    }

    /// Skip over any Newline tokens.
    pub fn skip_newlines(&mut self) {
        while matches!(self.peek_kind(), TokenKind::Newline) {
            self.advance();
        }
    }

    /// Skip over Newline, Indent, and Dedent tokens.
    pub fn skip_whitespace_tokens(&mut self) {
        loop {
            match self.peek_kind() {
                TokenKind::Newline
                | TokenKind::Indent
                | TokenKind::Dedent => { self.advance(); }
                _ => break,
            }
        }
    }

    /// Record a parse error.
    pub fn error(&mut self, span: Span, message: impl Into<String>) {
        // Cap at 20 errors per the Error Message Quality Specification
        if self.errors.len() < 20 {
        self.errors.push(ParseError::Custom {
    message : message.into(),
    span,
    hint    : None,
});
        }
    }

    /// Record an error and attempt to synchronize.
    /// Synchronization skips tokens until a known recovery point.
    pub fn error_and_recover(&mut self, span: Span, message: impl Into<String>) {
        self.error(span, message);
        self.synchronize();
    }

    /// Synchronization — skip tokens until a safe recovery point.
    /// Used after a parse error to continue collecting more errors.
    fn synchronize(&mut self) {
        while !self.at_eof() {
            match self.peek_kind() {
                // Stop at declaration starters
                TokenKind::Fn
                | TokenKind::Task
                | TokenKind::Struct
                | TokenKind::Enum
                | TokenKind::Impl
                | TokenKind::Trait
                | TokenKind::Actor
                | TokenKind::Opaque
                | TokenKind::Module
                | TokenKind::Import => return,
                // Stop at statement starters after a newline
                TokenKind::Dedent => return,
                _ => { self.advance(); }
            }
        }
    }

    // ── Kind matching helpers ──────────────────────────────────

    /// Match token kind — handles variants with data.
    fn peek_kind_matches(&self, kind: &TokenKind) -> bool {
        use std::mem::discriminant;
        match (self.peek_kind(), kind) {
            // For variants with data — match by discriminant only
            (TokenKind::Ident(_),   TokenKind::Ident(_))   => true,
            (TokenKind::IntLit(_),  TokenKind::IntLit(_))  => true,
            (TokenKind::FloatLit(_),TokenKind::FloatLit(_))=> true,
            (TokenKind::StrLit(_),  TokenKind::StrLit(_))  => true,
            (TokenKind::BoolLit(_), TokenKind::BoolLit(_)) => true,
            (TokenKind::Error(_),   TokenKind::Error(_))   => true,
            // For all other variants — exact match
            (a, b) => discriminant(a) == discriminant(b),
        }
    }
     // ── P2-07: Module and Import parsing ──────────────────────────

/// Parse @program_intent declaration if present.
/// Syntax: @program_intent NEWLINE """ ... """
fn parse_program_intent(&mut self) -> Option<crate::ast::ProgramIntent> {
    use crate::ast::ProgramIntent;

    if !matches!(self.peek_kind(), TokenKind::ProgramIntentDecl) {
        return None;
    }

    let span = self.current_span();
    self.advance(); // consume @program_intent

    self.skip_newlines();

    // Expect the triple-quoted string
    let description = match self.peek_kind().clone() {
        TokenKind::StrLit(s) => {
            self.advance();
            s
        }
        _ => {
            self.error(
                self.current_span(),
                "@program_intent must be followed by a triple-quoted string",
            );
            return None;
        }
    };

    self.skip_newlines();

    Some(ProgramIntent {
        span,
        description,
        constraints : vec![],  // AI pass extracts these at Stage 5
    })
}

/// Parse module declaration if present.
/// Syntax: module path.to.name NEWLINE
fn parse_module_decl(&mut self) -> Option<crate::ast::ModuleDecl> {
    use crate::ast::ModuleDecl;

    if !self.at(&TokenKind::Module) {
        return None;
    }

    let span = self.current_span();
    self.advance(); // consume 'module'

    let path = self.parse_dotted_ident_path();
    if path.is_empty() {
        self.error(
            self.current_span(),
            "expected module path after 'module'",
        );
        return None;
    }

    self.eat(&TokenKind::Newline);

    Some(ModuleDecl { span, path })
}

/// Parse import declaration.
/// Syntax: import path.to.module [as alias] NEWLINE
fn parse_import_decl(&mut self) -> Option<crate::ast::ImportDecl> {
    use crate::ast::ImportDecl;

    let span = self.current_span();
    self.expect(&TokenKind::Import)?; // consume 'import'

    let path = self.parse_dotted_ident_path();
    if path.is_empty() {
        self.error(
            self.current_span(),
            "expected module path after 'import'",
        );
        return None;
    }

    // Optional 'as alias'
    let alias = if self.at(&TokenKind::As) {
        self.advance(); // consume 'as'
        match self.peek_kind().clone() {
            TokenKind::Ident(name) => {
                let s = self.current_span();
                self.advance();
                Some(crate::ast::Ident::new(name, s))
            }
            _ => {
                self.error(
                    self.current_span(),
                    "expected identifier after 'as'",
                );
                None
            }
        }
    } else {
        None
    };

    self.eat(&TokenKind::Newline);

    Some(ImportDecl { span, path, alias })
}

/// Parse a dotted identifier path: foo.bar.baz
/// Returns a Vec of Idents.
fn parse_dotted_ident_path(&mut self) -> Vec<crate::ast::Ident> {
    let mut parts = Vec::new();

    // First segment
    match self.peek_kind().clone() {
        TokenKind::Ident(name) => {
            let s = self.current_span();
            self.advance();
            parts.push(crate::ast::Ident::new(name, s));
        }
        _ => return parts,
    }

    // Subsequent .segment parts
    while self.at(&TokenKind::Dot) {
        self.advance(); // consume '.'
        match self.peek_kind().clone() {
            TokenKind::Ident(name) => {
                let s = self.current_span();
                self.advance();
                parts.push(crate::ast::Ident::new(name, s));
            }
            _ => {
                self.error(
                    self.current_span(),
                    "expected identifier after '.' in module path",
                );
                break;
            }
        }
    }

    parts
}


    // ── Parse entry point ──────────────────────────────────────

    /// Parse the entire program.
    /// P2-07 will implement this fully.
    /// For now — returns an empty Program.
pub fn parse_program(&mut self) -> Program {
    let start = self.current_span();

    self.skip_newlines();

    // Parse optional @program_intent
    let program_intent = self.parse_program_intent();

    self.skip_newlines();

    // Parse optional module declaration
    let module = self.parse_module_decl();

    self.skip_newlines();

    // Parse imports
    let mut imports = Vec::new();
    while self.at(&TokenKind::Import) {
        if let Some(imp) = self.parse_import_decl() {
            imports.push(imp);
        }
        self.skip_newlines();
    }

    // P2-08: parse top-level items
    // For now skip everything remaining
    while !self.at_eof() {
        self.advance();
    }

    let end = self.current_span();
    Program {
        span : Self::merge_spans(start, end),
        program_intent,
        module,
        imports,
        items          : vec![],
    }
}
    /// Merge two spans into one covering both.
fn merge_spans(a: Span, b: Span) -> Span {
    Span::new(
        a.file,           // ← fixed
        a.start.min(b.start),
        a.end.max(b.end),
        a.line.min(b.line),
        a.col.min(b.col),
    )
}
} // closes impl<'src> Parser<'src>

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
        let _module  = &result.program.module;
        let _imports = &result.program.imports;
        let _items   = &result.program.items;
        let _intent  = &result.program.program_intent;
    }

    #[test]
    fn test_stmt_enum_has_defer_variant() {
        use crate::ast::{Stmt, DeferStmt, Expr, Literal};
        let span      = Span::new(file(), 0, 0, 1, 1);
        let lit       = Literal::Bool(true, span);
        let expr      = Expr::Lit(lit);
        let defer_stmt = DeferStmt { span, expr };
        let _stmt     = Stmt::Defer(defer_stmt);
    }

    #[test]
    fn test_stmt_enum_has_with_variant() {
        use crate::ast::{Stmt, WithStmt, Expr, Literal, Ident, Block};
        let span      = Span::new(file(), 0, 0, 1, 1);
        let lit       = Literal::Bool(true, span);
        let expr      = Expr::Lit(lit);
        let binding   = Ident::new("f", span);
        let body      = Block { span, stmts: vec![] };
        let with_stmt = WithStmt { span, expr, binding, body };
        let _stmt     = Stmt::With(with_stmt);
    }

    #[test]
    fn test_ast_intent_modes_all_exist() {
        let _a = IntentMode::Secure;
        let _b = IntentMode::Performant;
        let _c = IntentMode::Auditable;
        let _d = IntentMode::Verifiable;
        let _e = IntentMode::MinimalRuntime;
    }

    #[test]
    fn test_ast_mem_modes_all_exist() {
        let _a = MemMode::Own;
        let _b = MemMode::Borrow;
        let _c = MemMode::MutBorrow;
        let _d = MemMode::Share;
    }

    #[test]
    fn test_ast_provenance_kinds_exist() {
        let _a = ProvenanceKind::Tainted;
        let _b = ProvenanceKind::Clean;
        let _c = ProvenanceKind::Network;
        let _d = ProvenanceKind::FileSystem;
        let _e = ProvenanceKind::UserInput;
        let _f = ProvenanceKind::Trusted;
        let _g = ProvenanceKind::Unknown;
    }

    #[test]
    fn test_ast_temporal_expr_variants_exist() {
        let span = Span::new(file(), 0, 0, 1, 1);
        let _a   = TemporalExpr::Now(span);
        let _b   = TemporalExpr::Lifetime(span);
        let _c   = TemporalExpr::Epoch(span);
        let _d   = TemporalOp::Add;
        let _e   = TemporalOp::Sub;
    }

    #[test]
    fn test_ast_actor_decl_exists() {
        let span  = Span::new(file(), 0, 0, 1, 1);
        let name  = Ident::new("Worker", span);
        let actor = ActorDecl { span, name, items: vec![] };
        assert_eq!(actor.items.len(), 0);
    }

    #[test]
    fn test_ast_opaque_type_decl_exists() {
        let span   = Span::new(file(), 0, 0, 1, 1);
        let name   = Ident::new("UserId", span);
        let ty     = Type::Primitive(PrimitiveType::Int, span);
        let opaque = OpaqueTypeDecl { span, name, ty };
        assert_eq!(opaque.name.name, "UserId");
    }

    #[test]
    fn test_ast_bin_op_binding_power_ordering() {
        let (ml, _)   = BinOp::Mul.binding_power();
        let (al, _)   = BinOp::Add.binding_power();
        let (cl, _)   = BinOp::Eq.binding_power();
        let (andl, _) = BinOp::And.binding_power();
        let (orl, _)  = BinOp::Or.binding_power();
        assert!(ml > al,   "Mul > Add");
        assert!(al > cl,   "Add > Eq");
        assert!(cl > andl, "Eq > And");
        assert!(andl > orl,"And > Or");
    }

    #[test]
    fn test_ast_literal_span_extraction() {
        let span = Span::new(file(), 0, 5, 1, 1);
        assert_eq!(Literal::Int(42, span).span(), span);
        assert_eq!(Literal::Bool(true, span).span(), span);
        assert_eq!(Literal::None(span).span(), span);
    }

    // ── Cursor method tests ───────────────────────────────────

    #[test]
    fn test_cursor_peek_does_not_consume() {
        let mut p = make_parser("fn");
        let first = p.peek().kind.clone();
        let again = p.peek().kind.clone();
        assert_eq!(first, again);
    }

    #[test]
    fn test_cursor_advance_consumes() {
        let mut p   = make_parser("fn task");
        let first   = p.advance().kind;
        let second  = p.peek().kind.clone();
        assert_eq!(first,  TokenKind::Fn);
        assert_eq!(second, TokenKind::Task);
    }

    #[test]
    fn test_cursor_eat_matches() {
        let mut p = make_parser("fn");
        let tok   = p.eat(&TokenKind::Fn);
        assert!(tok.is_some());
    }

    #[test]
    fn test_cursor_eat_no_match() {
        let mut p = make_parser("fn");
        let tok   = p.eat(&TokenKind::Task);
        assert!(tok.is_none());
        // Token was not consumed
        assert_eq!(*p.peek_kind(), TokenKind::Fn);
    }

    #[test]
    fn test_cursor_expect_records_error() {
        let mut p   = make_parser("fn");
        let result  = p.expect(&TokenKind::Task);
        assert!(result.is_none());
        assert_eq!(p.errors.len(), 1);
        match &p.errors[0] {
    ParseError::Custom { message, .. } => {
        assert!(message.contains("task"),
            "expected error about 'task', got: {}", message);
    }
    ParseError::UnexpectedToken { expected, .. } => {
        assert!(expected.contains("task"),
            "expected error about 'task', got: {}", expected);
    }
    other => panic!("unexpected error variant: {:?}", other),
}
    }

    #[test]
    fn test_cursor_at_eof() {
        let mut p = make_parser("");
        assert!(p.at_eof());
    }

    #[test]
    fn test_cursor_skip_newlines() {
        let mut p = make_parser("fn");
        p.skip_newlines();
        // No newlines — should still be at fn
        assert_eq!(*p.peek_kind(), TokenKind::Fn);
    }

    #[test]
    fn test_error_cap_at_20() {
        let mut p = make_parser("");
        let span  = Span::new(file(), 0, 0, 1, 1);
        for i in 0..25 {
            p.error(span, format!("error {}", i));
        }
        assert_eq!(p.errors.len(), 20,
            "errors should be capped at 20");
    }

     #[test]
    fn test_parse_module_decl() {
        let result = crate::parse("module hello\n", file());
         assert!(result.is_ok(), "errors: {:?}", result.errors);
         assert!(result.program.module.is_some());
        let m = result.program.module.unwrap();
        assert_eq!(m.path.len(), 1);
        assert_eq!(m.path[0].name, "hello");
    }

    #[test]
    fn test_parse_module_path_dotted() {
        let result = crate::parse("module aieonyx.aegis.monitor\n", file());
        assert!(result.is_ok(), "errors: {:?}", result.errors);
        let m = result.program.module.unwrap();
        assert_eq!(m.path.len(), 3);
        assert_eq!(m.path[0].name, "aieonyx");
        assert_eq!(m.path[1].name, "aegis");
        assert_eq!(m.path[2].name, "monitor");
    }

    #[test]
    fn test_parse_import_simple() {
        let src = "module hello\nimport axon.sys\n";
        let result = crate::parse(src, file());
        assert!(result.is_ok(), "errors: {:?}", result.errors);
        assert_eq!(result.program.imports.len(), 1);
        assert_eq!(result.program.imports[0].path[0].name, "axon");
        assert_eq!(result.program.imports[0].path[1].name, "sys");
    }

    #[test]
    fn test_parse_import_with_alias() {
        let src = "module hello\nimport axon.sys.sel4.ipc as ipc\n";
        let result = crate::parse(src, file());
        assert!(result.is_ok(), "errors: {:?}", result.errors);
        let imp = &result.program.imports[0];
        assert!(imp.alias.is_some());
        assert_eq!(imp.alias.as_ref().unwrap().name, "ipc");
    }

    #[test]
        fn test_parse_multiple_imports() {
        let src = "module hello\nimport axon.sys\nimport axon.web\n";
        let result = crate::parse(src, file());
        assert!(result.is_ok(), "errors: {:?}", result.errors);
        assert_eq!(result.program.imports.len(), 2);
}
}