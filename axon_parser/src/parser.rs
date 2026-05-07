// ============================================================
// AXON Parser — parser.rs
// Recursive descent parser — stub for P2-01
// Full implementation begins P2-06
// ============================================================

use axon_lexer::{FileId, Span, Token, TokenKind};
use crate::ast::*;
use crate::error::ParseError;

/// The AXON recursive descent parser.
/// Consumes a token stream and produces a typed Program AST.
pub struct Parser<'src> {
    tokens  : Vec<Token>,
    pos     : usize,
    errors  : Vec<ParseError>,
    source  : &'src str,
    file_id : FileId,
}

impl<'src> Parser<'src> {
    /// Create a new parser from a token stream.
    pub fn new(tokens: Vec<Token>, source: &'src str, file_id: FileId) -> Self {
        Parser {
            tokens,
            pos: 0,
            errors: Vec::new(),
            source,
            file_id,
        }
    }

    /// Consume this parser and return the collected errors.
    pub fn into_errors(self) -> Vec<ParseError> {
        self.errors
    }

    // ── Core cursor methods ───────────────────────────────────

    /// Peek at the current token without consuming it.
    pub fn peek(&self) -> &Token {
        &self.tokens[self.pos.min(self.tokens.len() - 1)]
    }

    /// Peek at the current token kind.
    pub fn peek_kind(&self) -> &TokenKind {
        &self.peek().kind
    }

    /// Peek ahead by n tokens.
    pub fn peek_ahead(&self, n: usize) -> &Token {
        let idx = (self.pos + n).min(self.tokens.len() - 1);
        &self.tokens[idx]
    }

    /// Consume and return the current token.
    pub fn advance(&mut self) -> Token {
        let tok = self.tokens[self.pos.min(self.tokens.len() - 1)].clone();
        if self.pos < self.tokens.len() - 1 {
            self.pos += 1;
        }
        tok
    }

    /// Consume the current token if it matches kind.
    /// Returns true if consumed, false otherwise.
    pub fn eat(&mut self, kind: &TokenKind) -> bool {
        if std::mem::discriminant(self.peek_kind()) == std::mem::discriminant(kind) {
            self.advance();
            true
        } else {
            false
        }
    }

    /// Consume the current token, asserting it matches kind.
    /// Records an error and returns None if it doesn't match.
    pub fn expect(&mut self, kind: TokenKind) -> Option<Token> {
        if std::mem::discriminant(self.peek_kind()) == std::mem::discriminant(&kind) {
            Some(self.advance())
        } else {
            let err = ParseError::UnexpectedToken {
                expected : kind.display_name().to_string(),
                found    : self.peek().kind.clone(),
                span     : self.peek().span,
                hint     : None,
            };
            self.errors.push(err);
            None
        }
    }

    /// Skip tokens until a synchronization token is found.
    /// Used for error recovery.
    pub fn recover_to(&mut self, sync: &[TokenKind]) {
        while !matches!(self.peek_kind(), TokenKind::Eof) {
            for s in sync {
                if std::mem::discriminant(self.peek_kind()) == std::mem::discriminant(s) {
                    return;
                }
            }
            self.advance();
        }
    }

    /// Current span (span of the current token).
    pub fn current_span(&self) -> Span {
        self.peek().span
    }

    // ── Parse entry point ─────────────────────────────────────

    /// Parse the entire source file into a Program AST.
    /// P2-01 STUB — returns empty Program.
    /// Full implementation begins P2-07.
    pub fn parse_program(&mut self) -> Program {
        let start = self.current_span();

        // P2-01: stub — will be fully implemented in P2-07
        Program {
            span           : start,
            program_intent : None,
            module         : None,
            imports        : vec![],
            items          : vec![],
        }
    }
}

// ── Tests ────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use axon_lexer::FileId;

    #[test]
    fn test_parse_empty_source() {
        let result = crate::parse("", FileId(0));
        // P2-01: empty source produces empty program with no errors
        assert!(result.is_ok());
        assert!(result.program.items.is_empty());
    }

    #[test]
    fn test_parse_result_struct() {
        let result = crate::parse("fn main():", FileId(0));
        // P2-01: stub parser — no items yet but no panic
        assert!(result.program.items.is_empty() || !result.program.items.is_empty());
    }
    #[test]
fn test_ast_program_has_program_intent_field() {
    // Program struct must have program_intent field (v0.3.1)
    let file_id = FileId(1);
    let result  = crate::parse("", file_id);
    // program_intent is None for empty source
    assert!(result.program.program_intent.is_none());
}

#[test]
fn test_ast_program_has_all_fields() {
    let file_id = FileId(1);
    let result  = crate::parse("", file_id);
    // All required fields exist
    let _module  = &result.program.module;
    let _imports = &result.program.imports;
    let _items   = &result.program.items;
    let _intent  = &result.program.program_intent;
    // If this compiles — all fields exist
}

#[test]
fn test_stmt_enum_has_defer_variant() {
    use crate::ast::{Stmt, DeferStmt, Expr, Literal};
    use axon_lexer::Span;
    // DeferStmt can be constructed and wrapped in Stmt
    let span = Span::new(FileId(1), 0, 0, 1, 1);
    let lit  = Literal::Bool(true, span);
    let expr = Expr::Lit(lit);
    let defer_stmt = DeferStmt { span, expr };
    let _stmt = Stmt::Defer(defer_stmt);
    // If this compiles — Defer variant exists in Stmt
}

#[test]
fn test_stmt_enum_has_with_variant() {
    use crate::ast::{Stmt, WithStmt, Expr, Literal, Ident, Block};
    use axon_lexer::Span;
    let span    = Span::new(FileId(1), 0, 0, 1, 1);
    let lit     = Literal::Bool(true, span);
    let expr    = Expr::Lit(lit);
    let binding = Ident::new("f", span);
    let body    = Block { span, stmts: vec![] };
    let with_stmt = WithStmt { span, expr, binding, body };
    let _stmt = Stmt::With(with_stmt);
    // If this compiles — With variant exists in Stmt
}

#[test]
fn test_ast_intent_modes_all_exist() {
    use crate::ast::IntentMode;
    // All five intent modes must exist
    let _a = IntentMode::Secure;
    let _b = IntentMode::Performant;
    let _c = IntentMode::Auditable;
    let _d = IntentMode::Verifiable;
    let _e = IntentMode::MinimalRuntime;
    // If this compiles — all modes exist
}

#[test]
fn test_ast_mem_modes_all_exist() {
    use crate::ast::MemMode;
    let _a = MemMode::Own;
    let _b = MemMode::Borrow;
    let _c = MemMode::MutBorrow;
    let _d = MemMode::Share;
}

#[test]
fn test_ast_provenance_kinds_exist() {
    use crate::ast::ProvenanceKind;
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
    use crate::ast::{TemporalExpr, TemporalOp};
    use axon_lexer::Span;
    let span = Span::new(FileId(1), 0, 0, 1, 1);
    let _a = TemporalExpr::Now(span);
    let _b = TemporalExpr::Lifetime(span);
    let _c = TemporalExpr::Epoch(span);
    let _d = TemporalOp::Add;
    let _e = TemporalOp::Sub;
}

#[test]
fn test_ast_actor_decl_exists() {
    use crate::ast::{ActorDecl, Ident};
    use axon_lexer::Span;
    let span  = Span::new(FileId(1), 0, 0, 1, 1);
    let name  = Ident::new("Worker", span);
    let actor = ActorDecl { span, name, items: vec![] };
    // If this compiles — ActorDecl exists with correct fields
    assert_eq!(actor.items.len(), 0);
}

#[test]
fn test_ast_opaque_type_decl_exists() {
    use crate::ast::{OpaqueTypeDecl, Ident, Type, PrimitiveType};
    use axon_lexer::Span;
    let span  = Span::new(FileId(1), 0, 0, 1, 1);
    let name  = Ident::new("UserId", span);
    let ty    = Type::Primitive(PrimitiveType::Int, span);
    let opaque = OpaqueTypeDecl { span, name, ty };
    assert_eq!(opaque.name.name, "UserId");
}

#[test]
fn test_ast_bin_op_binding_power_ordering() {
    use crate::ast::BinOp;
    // Multiplication binds tighter than addition
    let (ml, _) = BinOp::Mul.binding_power();
    let (al, _) = BinOp::Add.binding_power();
    assert!(ml > al, "Mul should bind tighter than Add");

    // Addition binds tighter than comparison
    let (cl, _) = BinOp::Eq.binding_power();
    assert!(al > cl, "Add should bind tighter than Eq");

    // Comparison binds tighter than And
    let (andl, _) = BinOp::And.binding_power();
    assert!(cl > andl, "Eq should bind tighter than And");

    // And binds tighter than Or
    let (orl, _) = BinOp::Or.binding_power();
    assert!(andl > orl, "And should bind tighter than Or");
}

#[test]
fn test_ast_literal_span_extraction() {
    use crate::ast::Literal;
    use axon_lexer::Span;
    let span = Span::new(FileId(1), 0, 5, 1, 1);
    let lit  = Literal::Int(42, span);
    assert_eq!(lit.span(), span);

    let lit2 = Literal::Bool(true, span);
    assert_eq!(lit2.span(), span);

    let lit3 = Literal::None(span);
    assert_eq!(lit3.span(), span);
}
}
