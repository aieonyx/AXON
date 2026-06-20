// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
// P50 QA -- axon_parse test suite
// Pass bar: 12/12 before P51 begins.
// P55.5: AxString -> String; TypeExpr -> SovereignTy::Plain.

use axon_parse::{
    parse, BinOpKind, Expr, Item, Param,
    Stmt, TypeExpr, SovereignTy,
};

fn s(v: &str) -> String { v.to_string() }
fn ty(v: &str) -> SovereignTy { SovereignTy::Plain(TypeExpr { name: v.to_string() }) }

// T1: empty input yields empty Program
#[test]
fn test_parse_empty() {
    let prog = parse("").unwrap();
    assert_eq!(prog.items.len(), 0);
}

// T2: integer literal expression
#[test]
fn test_parse_int_literal() {
    let prog = parse("fn f() -> i32 { return 42; }").unwrap();
    if let Item::Fn { body, .. } = &prog.items[0] {
        if let Stmt::Return(Expr::IntLit(n)) = &body[0] {
            assert_eq!(*n, 42);
            return;
        }
    }
    panic!("expected IntLit(42)");
}

// T3: float literal expression
#[test]
fn test_parse_float_literal() {
    let prog = parse("fn f() -> f64 { return 3.14; }").unwrap();
    if let Item::Fn { body, .. } = &prog.items[0] {
        if let Stmt::Return(Expr::FloatLit(f)) = &body[0] {
            assert!((*f - 3.14_f64).abs() < 1e-10);
            return;
        }
    }
    panic!("expected FloatLit(3.14)");
}

// T4: bool literal
#[test]
fn test_parse_bool_literal() {
    let prog = parse("fn f() -> bool { return true; }").unwrap();
    if let Item::Fn { body, .. } = &prog.items[0] {
        assert_eq!(body[0], Stmt::Return(Expr::BoolLit(true)));
        return;
    }
    panic!("expected BoolLit(true)");
}

// T5: binary operation
#[test]
fn test_parse_binary_op() {
    let prog = parse("fn f() -> i32 { return 1 + 2; }").unwrap();
    if let Item::Fn { body, .. } = &prog.items[0] {
        if let Stmt::Return(Expr::BinOp { op, lhs, rhs }) = &body[0] {
            assert_eq!(*op, BinOpKind::Add);
            assert_eq!(**lhs, Expr::IntLit(1));
            assert_eq!(**rhs, Expr::IntLit(2));
            return;
        }
    }
    panic!("expected BinOp Add");
}

// T6: operator precedence -- 1 + 2 * 3 = Add(1, Mul(2, 3))
#[test]
fn test_parse_precedence() {
    let prog = parse("fn f() -> i32 { return 1 + 2 * 3; }").unwrap();
    if let Item::Fn { body, .. } = &prog.items[0] {
        if let Stmt::Return(Expr::BinOp { op: add_op, lhs, rhs }) = &body[0] {
            assert_eq!(*add_op, BinOpKind::Add);
            assert_eq!(**lhs, Expr::IntLit(1));
            if let Expr::BinOp { op: mul_op, lhs: l2, rhs: r2 } = rhs.as_ref() {
                assert_eq!(*mul_op, BinOpKind::Mul);
                assert_eq!(**l2, Expr::IntLit(2));
                assert_eq!(**r2, Expr::IntLit(3));
                return;
            }
        }
    }
    panic!("expected Add(1, Mul(2, 3))");
}

// T7: let statement
#[test]
fn test_parse_let_stmt() {
    let prog = parse("fn f() -> i32 { let x = 42; return x; }").unwrap();
    if let Item::Fn { body, .. } = &prog.items[0] {
        if let Stmt::Let { name, mutable, value } = &body[0] {
            assert_eq!(name.as_str(), "x");
            assert!(!mutable);
            assert_eq!(*value, Expr::IntLit(42));
            return;
        }
    }
    panic!("expected Let statement");
}

// T8: return statement
#[test]
fn test_parse_return_stmt() {
    let prog = parse("fn f() -> i32 { return 0; }").unwrap();
    if let Item::Fn { body, .. } = &prog.items[0] {
        assert_eq!(body[0], Stmt::Return(Expr::IntLit(0)));
        return;
    }
    panic!("expected Return statement");
}

// T9: function declaration with params
#[test]
fn test_parse_fn_decl() {
    let prog = parse("fn add(a: i32, b: i32) -> i32 { return a; }").unwrap();
    assert_eq!(prog.items.len(), 1);
    if let Item::Fn { name, params, ret, .. } = &prog.items[0] {
        assert_eq!(name.as_str(), "add");
        assert_eq!(params.len(), 2);
        assert_eq!(params[0], Param { name: s("a"), ty: ty("i32") });
        assert_eq!(params[1], Param { name: s("b"), ty: ty("i32") });
        assert_eq!(*ret, ty("i32"));
        return;
    }
    panic!("expected Fn item");
}

// T10: function call expression
#[test]
fn test_parse_fn_call() {
    let prog = parse("fn f() -> i32 { return add(1, 2); }").unwrap();
    if let Item::Fn { body, .. } = &prog.items[0] {
        if let Stmt::Return(Expr::Call { name, args }) = &body[0] {
            assert_eq!(name.as_str(), "add");
            assert_eq!(args.len(), 2);
            assert_eq!(args[0], Expr::IntLit(1));
            assert_eq!(args[1], Expr::IntLit(2));
            return;
        }
    }
    panic!("expected Call expr");
}

// T11: if expression with else
#[test]
fn test_parse_if_expr() {
    let src = "fn f() -> i32 { if x { return 1; } else { return 0; } }";
    let prog = parse(src).unwrap();
    if let Item::Fn { body, .. } = &prog.items[0] {
        if let Stmt::ExprStmt(Expr::If { cond, then, else_ }) = &body[0] {
            assert_eq!(**cond, Expr::Ident("x".to_string()));
            assert!(matches!(**then, Expr::Block(_)));
            assert!(else_.is_some());
            return;
        }
    }
    panic!("expected If expression");
}

// T12: full two-function program
#[test]
fn test_parse_full_program() {
    let src = "fn square(n: i32) -> i32 { return n * n; } fn main() -> i32 { let result = square(5); return result; }";
    let prog = parse(src).unwrap();
    assert_eq!(prog.items.len(), 2);

    if let Item::Fn { name, params, .. } = &prog.items[0] {
        assert_eq!(name.as_str(), "square");
        assert_eq!(params.len(), 1);
        assert_eq!(params[0].name.as_str(), "n");
    } else { panic!("expected square fn"); }

    if let Item::Fn { name, params, body, .. } = &prog.items[1] {
        assert_eq!(name.as_str(), "main");
        assert_eq!(params.len(), 0);
        assert_eq!(body.len(), 2);
    } else { panic!("expected main fn"); }
}
