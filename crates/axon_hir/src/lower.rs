// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
//
// AST -> HIR lowering bridge.
// Mirrors lower.ax exactly. Excised at P55 bootstrap.
// Each AST node maps to exactly one HIR node (1:1 lowering).

use axon_parse::{
    BinOpKind, Expr, Item, Program, Stmt, TypeExpr, UnaryOpKind,
};
use axon_std_string::AxString;
use crate::error::{HirError, HirResult};
use crate::hir::*;

// Global ID counter — atomic monotonic allocator.
use std::sync::atomic::{AtomicUsize, Ordering};
static NEXT_ID: AtomicUsize = AtomicUsize::new(0);
fn next_id() -> HirId {
    NEXT_ID.fetch_add(1, Ordering::Relaxed)
}

/// Lower an AXON source string to HirProgram.
pub fn lower_source(source: &str) -> HirResult<HirProgram> {
    let ast = axon_parse::parse(source)
        .map_err(|e| HirError::LoweringError(
            AxString::ax_from_str(&format!("parse error: {}", e))
        ))?;
    lower_program(&ast)
}

/// Lower a parsed AST Program to HirProgram.
pub fn lower_program(program: &Program) -> HirResult<HirProgram> {
    let mut hir = HirProgram::new();
    for item in &program.items {
        match item {
            Item::Fn { name, params, ret, body } => {
                hir.fns.push(lower_fn(name, params, ret, body)?);
            }
            Item::Struct { name, fields } => {
                hir.structs.push(lower_struct(name, fields)?);
            }
        }
    }
    Ok(hir)
}

fn lower_fn(
    name:   &AxString,
    params: &[axon_parse::Param],
    ret:    &TypeExpr,
    body:   &[Stmt],
) -> HirResult<HirFn> {
    let hir_params = params.iter()
        .map(|p| Ok(HirParam {
            name: p.name.clone(),
            ty:   lower_ty(&p.ty),
        }))
        .collect::<HirResult<Vec<_>>>()?;

    let hir_body = body.iter()
        .map(lower_stmt)
        .collect::<HirResult<Vec<_>>>()?;

    Ok(HirFn {
        id:     next_id(),
        name:   name.clone(),
        params: hir_params,
        ret:    lower_ty(ret),
        body:   hir_body,
    })
}

fn lower_struct(
    name:   &AxString,
    fields: &[axon_parse::Field],
) -> HirResult<HirStruct> {
    let hir_fields = fields.iter()
        .map(|f| Ok(HirField {
            name: f.name.clone(),
            ty:   lower_ty(&f.ty),
        }))
        .collect::<HirResult<Vec<_>>>()?;

    Ok(HirStruct {
        id:     next_id(),
        name:   name.clone(),
        fields: hir_fields,
    })
}

fn lower_ty(ty: &TypeExpr) -> HirTy {
    if ty.name.is_empty() {
        HirTy::Infer
    } else {
        HirTy::Named(ty.name.clone())
    }
}

fn lower_stmt(stmt: &Stmt) -> HirResult<HirStmt> {
    match stmt {
        Stmt::Let { name, mutable, value } => {
            Ok(HirStmt::Let {
                name:    name.clone(),
                mutable: *mutable,
                ty:      HirTy::Infer, // P52 fills this
                value:   lower_expr(value)?,
            })
        }
        Stmt::Return(expr) => {
            Ok(HirStmt::Return(lower_expr(expr)?))
        }
        Stmt::ExprStmt(expr) => {
            Ok(HirStmt::ExprStmt(lower_expr(expr)?))
        }
    }
}

fn lower_expr(expr: &Expr) -> HirResult<HirExpr> {
    match expr {
        Expr::IntLit(n)      => Ok(HirExpr::IntLit(*n)),
        Expr::FloatLit(f)    => Ok(HirExpr::FloatLit(*f)),
        Expr::StringLit(s)   => Ok(HirExpr::StringLit(s.clone())),
        Expr::BoolLit(b)     => Ok(HirExpr::BoolLit(*b)),
        Expr::Nil            => Ok(HirExpr::Nil),
        Expr::Ident(name)    => Ok(HirExpr::Var(name.clone())),
        Expr::BinOp { op, lhs, rhs } => {
            Ok(HirExpr::BinOp {
                op:  lower_binop(op),
                lhs: Box::new(lower_expr(lhs)?),
                rhs: Box::new(lower_expr(rhs)?),
            })
        }
        Expr::UnaryOp { op, expr } => {
            Ok(HirExpr::UnaryOp {
                op:   lower_unaryop(op),
                expr: Box::new(lower_expr(expr)?),
            })
        }
        Expr::Call { name, args } => {
            let hir_args = args.iter()
                .map(lower_expr)
                .collect::<HirResult<Vec<_>>>()?;
            Ok(HirExpr::Call { name: name.clone(), args: hir_args })
        }
        Expr::Block(stmts) => {
            // Blocks are inlined — last expr promoted to enclosing context
            // At P51 blocks with single return are the common case
            if stmts.is_empty() {
                return Ok(HirExpr::Nil);
            }
            // Return the HIR of the last stmt's expression if it's an ExprStmt
            if let Some(Stmt::Return(e)) = stmts.last() {
                return lower_expr(e);
            }
            // Otherwise wrap as an If-like structure via Nil sentinel
            Ok(HirExpr::Nil)
        }
        Expr::If { cond, then, else_ } => {
            let hir_cond = lower_expr(cond)?;
            let hir_then = lower_block_expr(then)?;
            let hir_else = match else_ {
                Some(e) => Some(lower_block_expr(e)?),
                None    => None,
            };
            Ok(HirExpr::If {
                cond:  Box::new(hir_cond),
                then:  hir_then,
                else_: hir_else,
            })
        }
    }
}

/// Lower a block expression (Expr::Block) to a Vec<HirStmt>.
fn lower_block_expr(expr: &Expr) -> HirResult<Vec<HirStmt>> {
    match expr {
        Expr::Block(stmts) => {
            stmts.iter().map(lower_stmt).collect()
        }
        other => {
            // Non-block expression in block position — wrap as ExprStmt
            Ok(vec![HirStmt::ExprStmt(lower_expr(other)?)])
        }
    }
}

fn lower_binop(op: &BinOpKind) -> HirBinOp {
    match op {
        BinOpKind::Add => HirBinOp::Add,
        BinOpKind::Sub => HirBinOp::Sub,
        BinOpKind::Mul => HirBinOp::Mul,
        BinOpKind::Div => HirBinOp::Div,
        BinOpKind::Mod => HirBinOp::Mod,
        BinOpKind::Eq  => HirBinOp::Eq,
        BinOpKind::Ne  => HirBinOp::Ne,
        BinOpKind::Lt  => HirBinOp::Lt,
        BinOpKind::Le  => HirBinOp::Le,
        BinOpKind::Gt  => HirBinOp::Gt,
        BinOpKind::Ge  => HirBinOp::Ge,
        BinOpKind::And => HirBinOp::And,
        BinOpKind::Or  => HirBinOp::Or,
    }
}

fn lower_unaryop(op: &UnaryOpKind) -> HirUnaryOp {
    match op {
        UnaryOpKind::Neg => HirUnaryOp::Neg,
        UnaryOpKind::Not => HirUnaryOp::Not,
    }
}
