// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
// P53 QA -- axon_codegen test suite
// Pass bar: 10/10 before P54 begins.

use axon_codegen::codegen_source;

// T1: integer literal → ret i32 42
#[test]
fn test_emit_int_lit() {
    let ir = codegen_source("fn f() -> i32 { return 42; }").unwrap();
    assert!(ir.contains("ret i32 42"), "expected 'ret i32 42' in:\n{}", ir);
}

// T2: bool literal → ret i1 1
#[test]
fn test_emit_bool_lit() {
    let ir = codegen_source("fn f() -> bool { return true; }").unwrap();
    assert!(ir.contains("ret i1 1"), "expected 'ret i1 1' in:\n{}", ir);
}

// T3: binary add → add i32 instruction
#[test]
fn test_emit_binop_add() {
    let ir = codegen_source("fn f() -> i32 { return 1 + 2; }").unwrap();
    assert!(ir.contains("add i32"), "expected 'add i32' in:\n{}", ir);
    assert!(ir.contains("ret i32"), "expected 'ret i32' in:\n{}", ir);
}

// T4: equality comparison → icmp eq i32
#[test]
fn test_emit_binop_eq() {
    let ir = codegen_source("fn f() -> bool { return 1 == 2; }").unwrap();
    assert!(ir.contains("icmp eq i32"), "expected 'icmp eq i32' in:\n{}", ir);
    assert!(ir.contains("ret i1"), "expected 'ret i1' in:\n{}", ir);
}

// T5: let binding → alloca + store
#[test]
fn test_emit_let() {
    let ir = codegen_source("fn f() -> i32 { let x = 42; return x; }").unwrap();
    assert!(ir.contains("alloca i32"), "expected 'alloca i32' in:\n{}", ir);
    assert!(ir.contains("store i32 42"), "expected 'store i32 42' in:\n{}", ir);
}

// T6: variable access after let → load
#[test]
fn test_emit_var() {
    let ir = codegen_source("fn f() -> i32 { let x = 42; return x; }").unwrap();
    assert!(ir.contains("load i32"), "expected 'load i32' in:\n{}", ir);
    assert!(ir.contains("ret i32"), "expected 'ret i32' in:\n{}", ir);
}

// T7: function with params → correct define header
#[test]
fn test_emit_fn_decl() {
    let ir = codegen_source("fn add(a: i32, b: i32) -> i32 { return a; }").unwrap();
    assert!(ir.contains("define i32 @add"), "expected 'define i32 @add' in:\n{}", ir);
    assert!(ir.contains("i32 %a"), "expected 'i32 %a' in:\n{}", ir);
    assert!(ir.contains("i32 %b"), "expected 'i32 %b' in:\n{}", ir);
}

// T8: function call → call i32 @add
#[test]
fn test_emit_call() {
    let src = "fn add(a: i32, b: i32) -> i32 { return a; } fn main() -> i32 { return add(1, 2); }";
    let ir = codegen_source(src).unwrap();
    assert!(ir.contains("call i32 @add"), "expected 'call i32 @add' in:\n{}", ir);
}

// T9: float return → ret double
#[test]
fn test_emit_float_ret() {
    let ir = codegen_source("fn f() -> f64 { return 3.14; }").unwrap();
    assert!(ir.contains("define double @f"), "expected 'define double @f' in:\n{}", ir);
    assert!(ir.contains("ret double"), "expected 'ret double' in:\n{}", ir);
}

// T10: full two-function program → two define blocks
#[test]
fn test_emit_full_program() {
    let src = "fn square(n: i32) -> i32 { return n * n; } fn main() -> i32 { return square(5); }";
    let ir = codegen_source(src).unwrap();
    assert!(ir.contains("define i32 @square"), "expected square fn in:\n{}", ir);
    assert!(ir.contains("define i32 @main"),   "expected main fn in:\n{}", ir);
    assert!(ir.contains("mul i32"),             "expected mul i32 in:\n{}", ir);
    assert!(ir.contains("call i32 @square"),    "expected call to square in:\n{}", ir);
    // Verify AXONYX header is present
    assert!(ir.contains("AXONYX sovereign codegen output"), "expected header in:\n{}", ir);
}
