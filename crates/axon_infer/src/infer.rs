// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
//
// Top-level type inference pass — mirrors constraint.ax logic.
// Two-pass: collect signatures, then infer bodies.
// Excised at P55 bootstrap when axonc compiles the .ax sources natively.
// P55.5: String throughout; new HirStmt/HirExpr variants handled.

use axon_hir::{
    HirBinOp, HirExpr, HirFn, HirProgram, HirStmt, HirTy, HirUnaryOp,
};
use axon_std_string::AxString;
use crate::error::{InferError, InferResult};
use crate::ty::{ty_from_hir, ty_from_name, Ty};
use crate::unify::unify_types;

// ── Output types ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub struct InferredFn {
    pub name:     String,
    pub ret_ty:   Ty,
    pub var_tys:  Vec<(String, Ty)>,
    pub stmt_tys: Vec<Ty>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct InferredProgram {
    pub fns: Vec<InferredFn>,
}

// ── Inference context ─────────────────────────────────────────────────────────

struct InferCtx {
    var_env: Vec<(String, Ty)>,
    fn_env:  Vec<(String, Vec<Ty>, Ty)>,
    ret_ty:  Ty,
}

impl InferCtx {
    fn new() -> Self {
        InferCtx { var_env: Vec::new(), fn_env: Vec::new(), ret_ty: Ty::Unknown }
    }

    fn lookup_var(&self, name: &str) -> InferResult<Ty> {
        for (n, ty) in self.var_env.iter().rev() {
            if n == name { return Ok(ty.clone()); }
        }
        Err(InferError::UndefinedName(
            AxString::ax_from_str(name)
        ))
    }

    fn lookup_fn(&self, name: &str) -> (Vec<Ty>, Ty) {
        for (n, params, ret) in &self.fn_env {
            if n == name { return (params.clone(), ret.clone()); }
        }
        (vec![], Ty::Unknown)
    }
}

// ── Public entry points ───────────────────────────────────────────────────────

pub fn infer_source(source: &str) -> InferResult<InferredProgram> {
    let ast = axon_parse::parse(source).map_err(|e| {
        InferError::PipelineError(AxString::ax_from_str(&format!("parse: {}", e)))
    })?;
    let hir = axon_hir::lower_program(&ast).map_err(|e| {
        InferError::PipelineError(AxString::ax_from_str(&format!("hir: {}", e)))
    })?;
    infer_program(&hir)
}

pub fn infer_program(hir: &HirProgram) -> InferResult<InferredProgram> {
    let mut ctx = InferCtx::new();

    // Pass 1: collect all function signatures
    for f in &hir.fns {
        let ret_ty    = ty_from_hir(&f.ret);
        let param_tys = f.params.iter().map(|p| ty_from_hir(&p.ty)).collect();
        ctx.fn_env.push((f.name.clone(), param_tys, ret_ty));
    }

    // Pass 2: infer each function body
    let fns = hir.fns.iter()
        .map(|f| infer_fn(&mut ctx, f))
        .collect::<InferResult<Vec<_>>>()?;

    Ok(InferredProgram { fns })
}

// ── Internal inference ────────────────────────────────────────────────────────

fn infer_fn(ctx: &mut InferCtx, f: &HirFn) -> InferResult<InferredFn> {
    ctx.var_env.clear();
    ctx.ret_ty = ty_from_hir(&f.ret);

    for p in &f.params {
        ctx.var_env.push((p.name.clone(), ty_from_hir(&p.ty)));
    }

    let mut var_tys:  Vec<(String, Ty)> = Vec::new();
    let mut stmt_tys: Vec<Ty>           = Vec::new();

    for stmt in &f.body {
        let (stmt_ty, new_bindings) = infer_stmt(ctx, stmt)?;
        stmt_tys.push(stmt_ty);
        for (name, ty) in new_bindings {
            ctx.var_env.push((name.clone(), ty.clone()));
            var_tys.push((name, ty));
        }
    }

    Ok(InferredFn {
        name:     f.name.clone(),
        ret_ty:   ctx.ret_ty.clone(),
        var_tys,
        stmt_tys,
    })
}

fn infer_stmt(
    ctx:  &mut InferCtx,
    stmt: &HirStmt,
) -> InferResult<(Ty, Vec<(String, Ty)>)> {
    match stmt {
        HirStmt::Let { name, ty: hir_ty, value, .. } => {
            let val_ty = infer_expr(ctx, value)?;
            let resolved = match hir_ty {
                HirTy::Named(n) => {
                    let expected = ty_from_name(n);
                    unify_types(&expected, &val_ty)?;
                    expected
                }
                _ => val_ty.clone(),
            };
            Ok((resolved.clone(), vec![(name.clone(), resolved)]))
        }
        // P55.5: ephemeral binding — inferred same as Let
        HirStmt::LetAt { name, ty: hir_ty, value } => {
            let val_ty = infer_expr(ctx, value)?;
            let resolved = match hir_ty {
                HirTy::Named(n) => {
                    let expected = ty_from_name(n);
                    unify_types(&expected, &val_ty)?;
                    expected
                }
                _ => val_ty.clone(),
            };
            Ok((resolved.clone(), vec![(name.clone(), resolved)]))
        }
        HirStmt::Return(expr) => {
            let ty  = infer_expr(ctx, expr)?;
            let ret = ctx.ret_ty.clone();
            unify_types(&ty, &ret)?;
            Ok((ty, vec![]))
        }
        HirStmt::ExprStmt(expr) => {
            let ty = infer_expr(ctx, expr)?;
            Ok((ty, vec![]))
        }
    }
}

fn infer_expr(ctx: &mut InferCtx, expr: &HirExpr) -> InferResult<Ty> {
    match expr {
        HirExpr::IntLit(_)    => Ok(Ty::I32),
        HirExpr::FloatLit(_)  => Ok(Ty::F64),
        HirExpr::BoolLit(_)   => Ok(Ty::Bool),
        HirExpr::StringLit(_) => Ok(Ty::Str),
        HirExpr::Nil          => Ok(Ty::Nil),
        HirExpr::Var(name)    => ctx.lookup_var(name),
        HirExpr::BinOp { op, lhs, rhs } => {
            let lhs_ty = infer_expr(ctx, lhs.as_ref())?;
            let rhs_ty = infer_expr(ctx, rhs.as_ref())?;
            unify_types(&lhs_ty, &rhs_ty)?;
            Ok(match op {
                HirBinOp::Eq  | HirBinOp::Ne |
                HirBinOp::Lt  | HirBinOp::Le |
                HirBinOp::Gt  | HirBinOp::Ge |
                HirBinOp::And | HirBinOp::Or => Ty::Bool,
                _ => lhs_ty,
            })
        }
        HirExpr::UnaryOp { op, expr } => {
            let ty = infer_expr(ctx, expr.as_ref())?;
            Ok(match op {
                HirUnaryOp::Not => Ty::Bool,
                HirUnaryOp::Neg => ty,
            })
        }
        HirExpr::Call { name, args } => {
            let (param_tys, ret_ty) = ctx.lookup_fn(name);
            for (i, arg) in args.iter().enumerate() {
                let arg_ty = infer_expr(ctx, arg)?;
                if let Some(param_ty) = param_tys.get(i) {
                    unify_types(&arg_ty, param_ty)?;
                }
            }
            Ok(ret_ty)
        }
        HirExpr::If { cond, then, else_ } => {
            let cond_ty = infer_expr(ctx, cond.as_ref())?;
            unify_types(&cond_ty, &Ty::Bool)?;
            for stmt in then { infer_stmt(ctx, stmt)?; }
            if let Some(else_stmts) = else_ {
                for stmt in else_stmts { infer_stmt(ctx, stmt)?; }
            }
            Ok(Ty::Nil)
        }
        // P55.5: v0.3 expression nodes — type-checked structurally
        HirExpr::Pipe { lhs, rhs, .. } => {
            infer_expr(ctx, lhs)?;
            infer_expr(ctx, rhs)
        }
        HirExpr::Morph { expr, .. } => infer_expr(ctx, expr),
        HirExpr::CapPinCall { expr, .. } => infer_expr(ctx, expr),
        HirExpr::Temporal(_) => Ok(Ty::I64), // timestamps as i64 epoch ms
        HirExpr::Foreach { gen, body, var } => {
            let gen_ty = infer_expr(ctx, gen)?;
            ctx.var_env.push((var.clone(), gen_ty));
            for stmt in body { infer_stmt(ctx, stmt)?; }
            ctx.var_env.pop();
            Ok(Ty::Nil)
        }
        HirExpr::Yield(expr) => infer_expr(ctx, expr),
        HirExpr::IntentBlock { body, .. } => {
            for stmt in body { infer_stmt(ctx, stmt)?; }
            Ok(Ty::Nil)
        }
    }
}
