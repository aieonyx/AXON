// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
// P55.7 M2 QA -- axon_ct_check static analysis tests.
// Pass bar: 10/10 before M2 closes.

use axon_ct_check::{check_program, CtError, CtViolation};
use axon_hir::hir::{
    HirDecorator, HirExpr, HirFn, HirParam, HirProgram, HirStmt, HirTy,
};

fn make_program(fns: Vec<HirFn>) -> HirProgram {
    HirProgram { fns, structs: vec![], actors: vec![] }
}

fn plain_fn(name: &str, body: Vec<HirStmt>) -> HirFn {
    HirFn {
        id: 0,
        decorators: vec![],
        name: name.to_string(),
        uses: vec![],
        params: vec![],
        ret: HirTy::Named("i32".to_string()),
        body,
    }
}

fn ct_fn(name: &str, params: Vec<HirParam>, body: Vec<HirStmt>) -> HirFn {
    HirFn {
        id: 0,
        decorators: vec![HirDecorator::ConstantTime],
        name: name.to_string(),
        uses: vec![],
        params,
        ret: HirTy::Named("i32".to_string()),
        body,
    }
}

fn secret_param(name: &str) -> HirParam {
    HirParam { name: name.to_string(), ty: HirTy::Secret(Box::new(HirTy::Named("i32".to_string()))) }
}

fn plain_param(name: &str) -> HirParam {
    HirParam { name: name.to_string(), ty: HirTy::Named("i32".to_string()) }
}

// T1: empty program passes
#[test]
fn test_empty_program_passes() {
    let prog = make_program(vec![]);
    assert!(check_program(&prog).is_ok());
}

// T2: non-CT function with secret branch is NOT checked (no violation)
#[test]
fn test_plain_fn_secret_branch_ignored() {
    let body = vec![
        HirStmt::ExprStmt(HirExpr::If {
            cond: Box::new(HirExpr::Var("s".to_string())),
            then: vec![HirStmt::Return(HirExpr::IntLit(1))],
            else_: None,
        })
    ];
    let f = plain_fn("non_ct", body);
    let prog = make_program(vec![f]);
    assert!(check_program(&prog).is_ok(), "non-CT fn should not be checked");
}

// T3: CT function with no secret params and no branches passes
#[test]
fn test_ct_fn_no_secrets_passes() {
    let body = vec![HirStmt::Return(HirExpr::IntLit(42))];
    let f = ct_fn("safe", vec![plain_param("x")], body);
    let prog = make_program(vec![f]);
    assert!(check_program(&prog).is_ok());
}

// T4: CT function with Secret<T> param but no branch passes
#[test]
fn test_ct_fn_secret_param_no_branch_passes() {
    let body = vec![HirStmt::Return(HirExpr::Var("s".to_string()))];
    let f = ct_fn("safe", vec![secret_param("s")], body);
    let prog = make_program(vec![f]);
    assert!(check_program(&prog).is_ok());
}

// T5: CT function branches on Secret<T> param — violation
#[test]
fn test_ct_fn_branch_on_secret_param_fails() {
    let body = vec![
        HirStmt::ExprStmt(HirExpr::If {
            cond: Box::new(HirExpr::Var("s".to_string())),
            then: vec![HirStmt::Return(HirExpr::IntLit(1))],
            else_: Some(vec![HirStmt::Return(HirExpr::IntLit(0))]),
        })
    ];
    let f = ct_fn("leaky", vec![secret_param("s")], body);
    let prog = make_program(vec![f]);
    let result = check_program(&prog);
    assert!(result.is_err(), "branch on secret should be a violation");
    if let Err(CtError::Violations(vs)) = result {
        assert!(!vs.is_empty());
        assert!(matches!(&vs[0], CtViolation::SecretParamBranch { fn_name, param }
            if fn_name == "leaky" && param == "s"));
    }
}

// T6: CT function branches on expression containing secret — violation
#[test]
fn test_ct_fn_branch_on_secret_expr_fails() {
    // if s == 0 { ... } — s is secret
    let cond = HirExpr::BinOp {
        op: axon_hir::hir::HirBinOp::Eq,
        lhs: Box::new(HirExpr::Var("s".to_string())),
        rhs: Box::new(HirExpr::IntLit(0)),
    };
    let body = vec![
        HirStmt::ExprStmt(HirExpr::If {
            cond: Box::new(cond),
            then: vec![HirStmt::Return(HirExpr::IntLit(1))],
            else_: None,
        })
    ];
    let f = ct_fn("leaky2", vec![secret_param("s")], body);
    let prog = make_program(vec![f]);
    assert!(check_program(&prog).is_err());
}

// T7: CT function with non-secret branch passes
#[test]
fn test_ct_fn_branch_on_plain_passes() {
    let body = vec![
        HirStmt::ExprStmt(HirExpr::If {
            cond: Box::new(HirExpr::Var("x".to_string())),
            then: vec![HirStmt::Return(HirExpr::IntLit(1))],
            else_: None,
        })
    ];
    let f = ct_fn("ok", vec![plain_param("x")], body);
    let prog = make_program(vec![f]);
    assert!(check_program(&prog).is_ok(), "branch on plain param should pass");
}

// T8: violation description is meaningful
#[test]
fn test_violation_description() {
    let v = CtViolation::SecretParamBranch {
        fn_name: "foo".to_string(),
        param: "key".to_string(),
    };
    let desc = v.description();
    assert!(desc.contains("foo"));
    assert!(desc.contains("key"));
    assert!(desc.contains("constant_time"));
}

// T9: multiple violations reported
#[test]
fn test_multiple_violations() {
    let body = vec![
        HirStmt::ExprStmt(HirExpr::If {
            cond: Box::new(HirExpr::Var("s".to_string())),
            then: vec![HirStmt::Return(HirExpr::IntLit(1))],
            else_: None,
        }),
        HirStmt::ExprStmt(HirExpr::If {
            cond: Box::new(HirExpr::Var("s".to_string())),
            then: vec![HirStmt::Return(HirExpr::IntLit(2))],
            else_: None,
        }),
    ];
    let f = ct_fn("double_leak", vec![secret_param("s")], body);
    let prog = make_program(vec![f]);
    if let Err(CtError::Violations(vs)) = check_program(&prog) {
        assert!(vs.len() >= 2, "expected at least 2 violations, got {}", vs.len());
    } else {
        panic!("expected violations");
    }
}

// T10: mixed program — CT fn with violation, plain fn without — only CT fn reported
#[test]
fn test_mixed_program_only_ct_checked() {
    let ct_body = vec![
        HirStmt::ExprStmt(HirExpr::If {
            cond: Box::new(HirExpr::Var("s".to_string())),
            then: vec![HirStmt::Return(HirExpr::IntLit(1))],
            else_: None,
        })
    ];
    let plain_body = vec![
        HirStmt::ExprStmt(HirExpr::If {
            cond: Box::new(HirExpr::Var("x".to_string())),
            then: vec![HirStmt::Return(HirExpr::IntLit(1))],
            else_: None,
        })
    ];
    let ct = ct_fn("leaky_ct", vec![secret_param("s")], ct_body);
    let plain = plain_fn("fine_plain", plain_body);
    let prog = make_program(vec![ct, plain]);
    if let Err(CtError::Violations(vs)) = check_program(&prog) {
        // All violations should be in leaky_ct, not fine_plain
        for v in &vs {
            match v {
                CtViolation::SecretParamBranch { fn_name, .. } => assert_eq!(fn_name, "leaky_ct"),
                CtViolation::SecretBranch { fn_name, .. } => assert_eq!(fn_name, "leaky_ct"),
                CtViolation::SecretEarlyReturn { fn_name } => assert_eq!(fn_name, "leaky_ct"),
            }
        }
    } else {
        panic!("expected violations from CT fn");
    }
}
