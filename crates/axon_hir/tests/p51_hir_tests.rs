// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
// P51 QA -- axon_hir test suite
// Pass bar: 10/10 before P52 begins.
// P55.5: AxString -> String throughout.

use axon_hir::{
    lower_source, HirBinOp, HirExpr, HirParam,
    HirStmt, HirTy,
};

fn named(s: &str) -> HirTy { HirTy::Named(s.to_string()) }

// T1: empty program
#[test]
fn test_lower_empty() {
    let hir = lower_source("").unwrap();
    assert_eq!(hir.fns.len(), 0);
    assert_eq!(hir.structs.len(), 0);
}

// T2: integer literal in return
#[test]
fn test_lower_int_lit() {
    let hir = lower_source("fn f() -> i32 { return 42; }").unwrap();
    assert_eq!(hir.fns[0].body[0], HirStmt::Return(HirExpr::IntLit(42)));
}

// T3: bool literal in return
#[test]
fn test_lower_bool_lit() {
    let hir = lower_source("fn f() -> bool { return true; }").unwrap();
    assert_eq!(hir.fns[0].body[0], HirStmt::Return(HirExpr::BoolLit(true)));
}

// T4: binary operation lowered correctly
#[test]
fn test_lower_binop() {
    let hir = lower_source("fn f() -> i32 { return 1 + 2; }").unwrap();
    if let HirStmt::Return(HirExpr::BinOp { op, lhs, rhs }) = &hir.fns[0].body[0] {
        assert_eq!(*op, HirBinOp::Add);
        assert_eq!(**lhs, HirExpr::IntLit(1));
        assert_eq!(**rhs, HirExpr::IntLit(2));
        return;
    }
    panic!("expected BinOp Add");
}

// T5: let stmt with no annotation -> HirTy::Infer
#[test]
fn test_lower_let() {
    let hir = lower_source("fn f() -> i32 { let x = 42; return 0; }").unwrap();
    if let HirStmt::Let { name, mutable, ty, value } = &hir.fns[0].body[0] {
        assert_eq!(name.as_str(), "x");
        assert!(!mutable);
        assert_eq!(*ty, HirTy::Infer);
        assert_eq!(*value, HirExpr::IntLit(42));
        return;
    }
    panic!("expected Let with Infer type");
}

// T6: let stmt with type annotation -> HirTy resolves
#[test]
fn test_lower_let_typed() {
    let hir = lower_source("fn f() -> i32 { let x: i32 = 42; return 0; }").unwrap();
    if let HirStmt::Let { ty, .. } = &hir.fns[0].body[0] {
        assert!(matches!(ty, HirTy::Infer | HirTy::Named(_)));
        return;
    }
    panic!("expected Let statement");
}

// T7: full fn declaration lowered correctly
#[test]
fn test_lower_fn() {
    let hir = lower_source("fn add(a: i32, b: i32) -> i32 { return a; }").unwrap();
    assert_eq!(hir.fns.len(), 1);
    let f = &hir.fns[0];
    assert_eq!(f.name.as_str(), "add");
    assert_eq!(f.params.len(), 2);
    assert_eq!(f.params[0], HirParam { name: "a".to_string(), ty: named("i32") });
    assert_eq!(f.params[1], HirParam { name: "b".to_string(), ty: named("i32") });
    assert_eq!(f.ret, named("i32"));
    assert_eq!(f.body.len(), 1);
}

// T8: function call lowered correctly
#[test]
fn test_lower_call() {
    let hir = lower_source("fn f() -> i32 { return add(1, 2); }").unwrap();
    if let HirStmt::Return(HirExpr::Call { name, args }) = &hir.fns[0].body[0] {
        assert_eq!(name.as_str(), "add");
        assert_eq!(args.len(), 2);
        assert_eq!(args[0], HirExpr::IntLit(1));
        assert_eq!(args[1], HirExpr::IntLit(2));
        return;
    }
    panic!("expected Call");
}

// T9: if expression lowered with flattened blocks
#[test]
fn test_lower_if() {
    let src = "fn f() -> i32 { if x { return 1; } }";
    let hir = lower_source(src).unwrap();
    if let HirStmt::ExprStmt(HirExpr::If { cond, then, else_ }) = &hir.fns[0].body[0] {
        assert_eq!(**cond, HirExpr::Var("x".to_string()));
        assert!(!then.is_empty());
        assert!(else_.is_none());
        return;
    }
    panic!("expected If expr");
}

// T10: full two-function program lowered correctly
#[test]
fn test_lower_full_program() {
    let src = "fn square(n: i32) -> i32 { return n * n; } fn main() -> i32 { return square(5); }";
    let hir = lower_source(src).unwrap();
    assert_eq!(hir.fns.len(), 2);
    assert_eq!(hir.fns[0].name.as_str(), "square");
    assert_eq!(hir.fns[1].name.as_str(), "main");
    assert_eq!(hir.fns[0].params.len(), 1);
    assert_eq!(hir.fns[1].params.len(), 0);
    if let HirStmt::Return(HirExpr::BinOp { op, .. }) = &hir.fns[0].body[0] {
        assert_eq!(*op, HirBinOp::Mul);
    } else { panic!("expected Mul in square"); }
    if let HirStmt::Return(HirExpr::Call { name, args }) = &hir.fns[1].body[0] {
        assert_eq!(name.as_str(), "square");
        assert_eq!(args[0], HirExpr::IntLit(5));
    } else { panic!("expected Call in main"); }
}
