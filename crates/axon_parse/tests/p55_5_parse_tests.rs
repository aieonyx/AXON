// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
//
// P55.5 — axon_parse AST node completeness tests
// Verifies every v0.3 + sovereign/security/finance AST node is declared in ast.ax

use axon_parse::ast::*;

// T1: SovereignTy variants exist
#[test]
fn test_sovereign_ty_tainted() {
    let _ty = SovereignTy::Tainted(Box::new(TypeExpr { name: "u8".to_string() }));
}

#[test]
fn test_sovereign_ty_clean() {
    let _ty = SovereignTy::Clean(Box::new(TypeExpr { name: "str".to_string() }));
}

#[test]
fn test_sovereign_ty_secret() {
    let _ty = SovereignTy::Secret(Box::new(TypeExpr { name: "u8".to_string() }));
}

#[test]
fn test_sovereign_ty_auditable() {
    let _ty = SovereignTy::Auditable(Box::new(TypeExpr { name: "Record".to_string() }));
}

#[test]
fn test_sovereign_ty_expires() {
    let _ty = SovereignTy::Expires {
        inner: Box::new(TypeExpr { name: "AuthToken".to_string() }),
        after: "15min".to_string(),
    };
}

#[test]
fn test_sovereign_ty_resident() {
    let _ty = SovereignTy::Resident {
        inner: Box::new(TypeExpr { name: "UserRecord".to_string() }),
        jurisdiction: "EU".to_string(),
    };
}

#[test]
fn test_sovereign_ty_money() {
    let _ty = SovereignTy::Money { currency: "EUR".to_string(), precision: 2 };
}

#[test]
fn test_sovereign_ty_safeint() {
    let _ty = SovereignTy::SafeInt { lo: 0, hi: 1_000_000 };
}

#[test]
fn test_sovereign_ty_refinement() {
    let _ty = SovereignTy::Refinement {
        base: Box::new(TypeExpr { name: "i64".to_string() }),
        pred: "_ > 0".to_string(),
    };
}

#[test]
fn test_sovereign_ty_opaque() {
    let _ty = SovereignTy::Opaque {
        name: "UserId".to_string(),
        inner: Box::new(TypeExpr { name: "i64".to_string() }),
    };
}

// T2: Decorator variants exist
#[test] fn test_decorator_deterministic()  { let _d = Decorator::Deterministic; }
#[test] fn test_decorator_constant_time()  { let _d = Decorator::ConstantTime; }
#[test] fn test_decorator_balanced()       { let _d = Decorator::Balanced; }
#[test] fn test_decorator_atomic_fin()     { let _d = Decorator::AtomicFinancial; }
#[test] fn test_decorator_sealed_memory()  { let _d = Decorator::SealedMemory; }

#[test]
fn test_decorator_requires_consent() {
    let _d = Decorator::RequiresConsent {
        user_id: "user_id".to_string(),
        purpose: "analytics".to_string(),
    };
}

#[test]
fn test_decorator_model_signed() {
    let _d = Decorator::ModelSigned("pubkey_hex".to_string());
}

#[test]
fn test_decorator_inference_budget() {
    let _d = Decorator::InferenceBudget { tokens: 1000, time_ms: 100 };
}

#[test]
fn test_decorator_requires_human() {
    let _d = Decorator::RequiresHumanApproval("10000 EUR".to_string());
}

// T3: v0.3 expression nodes exist
#[test]
fn test_expr_pipe() {
    let _e = Expr::Pipe {
        lhs: Box::new(Expr::Ident("x".to_string())),
        rhs: Box::new(Expr::Ident("hash".to_string())),
        contract: Some("@ensures(result > 0)".to_string()),
    };
}

#[test]
fn test_expr_morph() {
    let _e = Expr::Morph {
        expr: Box::new(Expr::Ident("req".to_string())),
        method: "sanitize".to_string(),
    };
}

#[test]
fn test_expr_cap_pin_required() {
    let _e = Expr::CapPinCall {
        expr: Box::new(Expr::Ident("session".to_string())),
        method: "open".to_string(),
        pin: CapPin::Required,
    };
}

#[test]
fn test_expr_temporal_now()      { let _e = Expr::Temporal(TemporalKind::Now); }
#[test] fn test_expr_temporal_lifetime() { let _e = Expr::Temporal(TemporalKind::Lifetime); }
#[test] fn test_expr_temporal_epoch()    { let _e = Expr::Temporal(TemporalKind::Epoch); }

#[test]
fn test_expr_intent_block() {
    let _e = Expr::IntentBlock {
        modes: vec!["secure".to_string(), "auditable".to_string()],
        body: vec![],
    };
}

#[test]
fn test_expr_yield() {
    let _e = Expr::Yield(Box::new(Expr::IntLit(42)));
}

// T4: v0.3 statement nodes
#[test]
fn test_stmt_let_at() {
    let _s = Stmt::LetAt {
        name: "session".to_string(),
        value: Expr::Ident("open_session".to_string()),
    };
}

// T5: Actor item node
#[test]
fn test_item_actor() {
    let _i = Item::Actor {
        name: "RequestHandler".to_string(),
        handles: vec![],
    };
}

// T6: Fn item with decorators and uses
#[test]
fn test_fn_with_decorators_and_uses() {
    let _i = Item::Fn {
        decorators: vec![Decorator::Deterministic, Decorator::ConstantTime],
        name: "verify_input".to_string(),
        uses: vec!["crypto".to_string()],
        params: vec![],
        ret: SovereignTy::Plain(TypeExpr { name: "bool".to_string() }),
        body: vec![],
    };
}
