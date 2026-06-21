// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
// check.rs -- HIR walker for @constant_time violation detection.
// Only functions decorated with @constant_time are checked.
// Non-@constant_time functions are skipped entirely.

use axon_hir::hir::{
    HirDecorator, HirExpr, HirFn, HirProgram, HirStmt, HirTy,
};
use crate::error::{CtError, CtResult, CtViolation};

/// Check all @constant_time functions in a HIR program.
/// Returns Ok(()) if no violations, Err(CtError::Violations(..)) otherwise.
pub fn check_program(program: &HirProgram) -> CtResult<()> {
    let mut violations = Vec::new();
    for f in &program.fns {
        if f.decorators.iter().any(|d| matches!(d, HirDecorator::ConstantTime)) {
            check_fn(f, &mut violations);
        }
    }
    if violations.is_empty() {
        Ok(())
    } else {
        Err(CtError::Violations(violations))
    }
}

/// Collect secret-typed parameter names for a function.
fn secret_params(f: &HirFn) -> Vec<String> {
    f.params.iter()
        .filter(|p| is_secret_ty(&p.ty))
        .map(|p| p.name.clone())
        .collect()
}

/// Check a single @constant_time function for violations.
fn check_fn(f: &HirFn, violations: &mut Vec<CtViolation>) {
    let secrets = secret_params(f);
    for stmt in &f.body {
        check_stmt(stmt, &f.name, &secrets, violations);
    }
}

/// Walk a statement for CT violations.
fn check_stmt(
    stmt: &HirStmt,
    fn_name: &str,
    secrets: &[String],
    violations: &mut Vec<CtViolation>,
) {
    match stmt {
        HirStmt::Let { value, .. } | HirStmt::LetAt { value, .. } => {
            check_expr_not_branch_target(value, fn_name, secrets, violations);
        }
        HirStmt::Return(expr) => {
            // Return itself is fine — it's unconditional.
            // But if the return value is an If expression conditioned on a secret,
            // that's a violation (it creates variable-time behavior).
            if let HirExpr::If { cond, .. } = expr {
                if expr_touches_secret(cond, secrets) {
                    violations.push(CtViolation::SecretEarlyReturn {
                        fn_name: fn_name.to_string(),
                    });
                }
            }
        }
        HirStmt::ExprStmt(expr) => {
            check_expr_not_branch_target(expr, fn_name, secrets, violations);
        }
    }
}

/// Walk an expression and flag any If/match that branches on a secret value.
fn check_expr_not_branch_target(
    expr: &HirExpr,
    fn_name: &str,
    secrets: &[String],
    violations: &mut Vec<CtViolation>,
) {
    match expr {
        HirExpr::If { cond, then, else_ } => {
            // Branch on a secret value is a CT violation
            if expr_touches_secret(cond, secrets) {
                // Identify which secret param is involved
                if let Some(param) = find_secret_var(cond, secrets) {
                    violations.push(CtViolation::SecretParamBranch {
                        fn_name: fn_name.to_string(),
                        param,
                    });
                } else {
                    violations.push(CtViolation::SecretBranch {
                        fn_name: fn_name.to_string(),
                        detail: "condition involves secret-typed expression".to_string(),
                    });
                }
            }
            // Recurse into branches
            for stmt in then {
                check_stmt(stmt, fn_name, secrets, violations);
            }
            if let Some(else_stmts) = else_ {
                for stmt in else_stmts {
                    check_stmt(stmt, fn_name, secrets, violations);
                }
            }
        }
        HirExpr::BinOp { lhs, rhs, .. } => {
            check_expr_not_branch_target(lhs, fn_name, secrets, violations);
            check_expr_not_branch_target(rhs, fn_name, secrets, violations);
        }
        HirExpr::UnaryOp { expr, .. } => {
            check_expr_not_branch_target(expr, fn_name, secrets, violations);
        }
        HirExpr::Call { args, .. } => {
            for arg in args {
                check_expr_not_branch_target(arg, fn_name, secrets, violations);
            }
        }
        HirExpr::Pipe { lhs, rhs, .. } => {
            check_expr_not_branch_target(lhs, fn_name, secrets, violations);
            check_expr_not_branch_target(rhs, fn_name, secrets, violations);
        }
        // Literals, vars, temporal — no branch, no violation
        _ => {}
    }
}

/// Returns true if an expression involves a secret-typed variable.
fn expr_touches_secret(expr: &HirExpr, secrets: &[String]) -> bool {
    match expr {
        HirExpr::Var(name) => secrets.contains(name),
        HirExpr::BinOp { lhs, rhs, .. } =>
            expr_touches_secret(lhs, secrets) || expr_touches_secret(rhs, secrets),
        HirExpr::UnaryOp { expr, .. } => expr_touches_secret(expr, secrets),
        HirExpr::Call { args, .. } =>
            args.iter().any(|a| expr_touches_secret(a, secrets)),
        _ => false,
    }
}

/// Find the name of a secret variable referenced in an expression.
fn find_secret_var(expr: &HirExpr, secrets: &[String]) -> Option<String> {
    match expr {
        HirExpr::Var(name) if secrets.contains(name) => Some(name.clone()),
        HirExpr::BinOp { lhs, rhs, .. } =>
            find_secret_var(lhs, secrets).or_else(|| find_secret_var(rhs, secrets)),
        HirExpr::UnaryOp { expr, .. } => find_secret_var(expr, secrets),
        _ => None,
    }
}

/// Returns true if a HIR type is Secret<T>.
fn is_secret_ty(ty: &HirTy) -> bool {
    matches!(ty, HirTy::Secret(_))
}
