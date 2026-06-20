// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
// P52 QA -- axon_infer test suite
// Pass bar: 10/10 before P53 begins.
// P55.5: AxString -> String; ax() helper removed.

use axon_infer::{infer_source, Ty};

// T1: integer literal infers as I32
#[test]
fn test_infer_int_lit() {
    let r = infer_source("fn f() -> i32 { return 42; }").unwrap();
    assert_eq!(r.fns[0].stmt_tys[0], Ty::I32);
}

// T2: float literal infers as F64
#[test]
fn test_infer_float_lit() {
    let r = infer_source("fn f() -> f64 { return 3.14; }").unwrap();
    assert_eq!(r.fns[0].stmt_tys[0], Ty::F64);
}

// T3: bool literal infers as Bool
#[test]
fn test_infer_bool_lit() {
    let r = infer_source("fn f() -> bool { return true; }").unwrap();
    assert_eq!(r.fns[0].stmt_tys[0], Ty::Bool);
}

// T4: arithmetic BinOp infers operand type
#[test]
fn test_infer_binop_add() {
    let r = infer_source("fn f() -> i32 { return 1 + 2; }").unwrap();
    assert_eq!(r.fns[0].stmt_tys[0], Ty::I32);
}

// T5: comparison BinOp infers Bool
#[test]
fn test_infer_binop_eq() {
    let r = infer_source("fn f() -> bool { return 1 == 2; }").unwrap();
    assert_eq!(r.fns[0].stmt_tys[0], Ty::Bool);
}

// T6: let binding infers type from value
#[test]
fn test_infer_let_infer() {
    let r = infer_source("fn f() -> i32 { let x = 42; return x; }").unwrap();
    let f = &r.fns[0];
    assert_eq!(f.var_tys[0], ("x".to_string(), Ty::I32));
}

// T7: let binding from bool literal
#[test]
fn test_infer_let_bool() {
    let r = infer_source("fn f() -> bool { let b = true; return b; }").unwrap();
    let f = &r.fns[0];
    assert_eq!(f.var_tys[0], ("b".to_string(), Ty::Bool));
}

// T8: function return type correctly resolved from annotation
#[test]
fn test_infer_fn_return() {
    let r = infer_source("fn add(a: i32, b: i32) -> i32 { return a; }").unwrap();
    let f = &r.fns[0];
    assert_eq!(f.ret_ty, Ty::I32);
    assert_eq!(f.name.as_str(), "add");
}

// T9: function call args unified with param types
#[test]
fn test_infer_call() {
    let src = "fn add(a: i32, b: i32) -> i32 { return a; } fn main() -> i32 { return add(1, 2); }";
    let r = infer_source(src).unwrap();
    assert_eq!(r.fns[1].stmt_tys[0], Ty::I32);
    assert_eq!(r.fns[1].ret_ty, Ty::I32);
}

// T10: full two-function program correctly typed
#[test]
fn test_infer_full_program() {
    let src = "fn square(n: i32) -> i32 { return n * n; } fn main() -> i32 { return square(5); }";
    let r = infer_source(src).unwrap();
    assert_eq!(r.fns[0].name.as_str(), "square");
    assert_eq!(r.fns[0].ret_ty, Ty::I32);
    assert_eq!(r.fns[0].stmt_tys[0], Ty::I32);
    assert_eq!(r.fns[1].name.as_str(), "main");
    assert_eq!(r.fns[1].ret_ty, Ty::I32);
    assert_eq!(r.fns[1].stmt_tys[0], Ty::I32);
}
