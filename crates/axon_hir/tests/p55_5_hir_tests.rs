// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
//
// P55.5 — axon_hir HIR node completeness tests
// Verifies every v0.3 + sovereign/security/finance HIR node is declared in hir.ax

use axon_hir::hir::*;

// T1: HirTy sovereign variants
#[test] fn test_hir_ty_tainted()   { let _t = HirTy::Tainted(Box::new(HirTy::Named("u8".into()))); }
#[test] fn test_hir_ty_clean()     { let _t = HirTy::Clean(Box::new(HirTy::Named("str".into()))); }
#[test] fn test_hir_ty_secret()    { let _t = HirTy::Secret(Box::new(HirTy::Named("u8".into()))); }
#[test] fn test_hir_ty_auditable() { let _t = HirTy::Auditable(Box::new(HirTy::Named("Record".into()))); }

#[test]
fn test_hir_ty_expires() {
    let _t = HirTy::Expires {
        inner: Box::new(HirTy::Named("AuthToken".into())),
        after_ms: 900_000,
    };
}

#[test]
fn test_hir_ty_resident() {
    let _t = HirTy::Resident {
        inner: Box::new(HirTy::Named("UserRecord".into())),
        jurisdiction: "EU".into(),
    };
}

#[test]
fn test_hir_ty_money()   { let _t = HirTy::Money { currency: "EUR".into(), precision: 2 }; }
#[test]
fn test_hir_ty_safeint() { let _t = HirTy::SafeInt { lo: 0, hi: 1_000_000 }; }

#[test]
fn test_hir_ty_refinement() {
    let _t = HirTy::Refinement {
        base: Box::new(HirTy::Named("i64".into())),
        pred: "_ > 0".into(),
    };
}

// T2: HirDecorator variants
#[test] fn test_hir_dec_deterministic()   { let _d = HirDecorator::Deterministic; }
#[test] fn test_hir_dec_constant_time()   { let _d = HirDecorator::ConstantTime; }
#[test] fn test_hir_dec_balanced()        { let _d = HirDecorator::Balanced; }
#[test] fn test_hir_dec_atomic_fin()      { let _d = HirDecorator::AtomicFinancial; }
#[test] fn test_hir_dec_sealed_memory()   { let _d = HirDecorator::SealedMemory; }

#[test]
fn test_hir_dec_inference_budget() {
    let _d = HirDecorator::InferenceBudget { tokens: 1000, time_ms: 100 };
}

// T3: HirExpr v0.3 variants
#[test]
fn test_hir_expr_pipe() {
    let _e = HirExpr::Pipe {
        lhs: Box::new(HirExpr::Var("x".into())),
        rhs: Box::new(HirExpr::Var("hash".into())),
        contract: None,
    };
}

#[test]
fn test_hir_expr_morph() {
    let _e = HirExpr::Morph {
        expr: Box::new(HirExpr::Var("req".into())),
        method: "sanitize".into(),
    };
}

#[test]
fn test_hir_expr_temporal() { let _e = HirExpr::Temporal(HirTemporalKind::Now); }

#[test]
fn test_hir_expr_intent_block() {
    let _e = HirExpr::IntentBlock {
        modes: vec!["secure".into()],
        body: vec![],
    };
}

#[test]
fn test_hir_expr_yield() {
    let _e = HirExpr::Yield(Box::new(HirExpr::IntLit(42)));
}

// T4: HirStmt LetAt
#[test]
fn test_hir_stmt_let_at() {
    let _s = HirStmt::LetAt {
        name: "session".into(),
        ty: HirTy::Infer,
        value: HirExpr::Var("open_session".into()),
    };
}

// T5: HirActor
#[test]
fn test_hir_actor() {
    let _a = HirActor {
        id: 0,
        name: "RequestHandler".into(),
        handles: vec![],
    };
}

// T6: HirProgram includes actors
#[test]
fn test_hir_program_has_actors() {
    let p = hir_program_new();
    assert_eq!(p.actors.len(), 0);
    assert_eq!(p.fns.len(), 0);
    assert_eq!(p.structs.len(), 0);
}

// T7: HirFn with decorators and uses
#[test]
fn test_hir_fn_with_decorators() {
    let _f = HirFn {
        id: 0,
        decorators: vec![HirDecorator::Deterministic, HirDecorator::ConstantTime],
        name: "verify_input".into(),
        uses: vec!["crypto".into()],
        params: vec![],
        ret: HirTy::Named("bool".into()),
        body: vec![],
    };
}
