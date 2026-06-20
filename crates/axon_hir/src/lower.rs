// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
//
// AST -> HIR lowering bridge.
// Mirrors lower.ax exactly. Excised at P55 bootstrap.
// Each AST node maps to exactly one HIR node (1:1 lowering).
// P55.5: updated for SovereignTy, Decorator, uses, LetAt, Actor,
//        and v0.3 expression nodes. All existing logic preserved unchanged.

use axon_parse::{
    BinOpKind, CapPin, Decorator, Expr, Item, Program,
    Stmt, SovereignTy, TemporalKind, TypeExpr, UnaryOpKind,
};
use axon_std_string::AxString;
use crate::error::{HirError, HirResult};
use crate::hir::*;

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
    let mut hir = hir_program_new();
    for item in &program.items {
        match item {
            Item::Fn { decorators, name, uses, params, ret, body } => {
                hir.fns.push(lower_fn(decorators, name, uses, params, ret, body)?);
            }
            Item::Struct { name, fields } => {
                hir.structs.push(lower_struct(name, fields)?);
            }
            Item::Actor { name, handles } => {
                hir.actors.push(HirActor {
                    id:      next_id(),
                    name:    name.clone(),
                    handles: handles.iter().map(|h| HirHandleBlock {
                        msg_name: h.msg_name.clone(),
                        msg_ty:   lower_sovereign_ty(&h.msg_ty),
                        ret_ty:   lower_sovereign_ty(&h.ret_ty),
                        body:     h.body.iter()
                                    .map(lower_stmt)
                                    .collect::<HirResult<Vec<_>>>()
                                    .unwrap_or_default(),
                    }).collect(),
                });
            }
        }
    }
    Ok(hir)
}

fn lower_fn(
    decorators: &[Decorator],
    name:       &str,
    uses:       &[String],
    params:     &[axon_parse::Param],
    ret:        &SovereignTy,
    body:       &[Stmt],
) -> HirResult<HirFn> {
    let hir_params = params.iter()
        .map(|p| Ok(HirParam {
            name: p.name.clone(),
            ty:   lower_sovereign_ty(&p.ty),
        }))
        .collect::<HirResult<Vec<_>>>()?;

    let hir_body = body.iter()
        .map(lower_stmt)
        .collect::<HirResult<Vec<_>>>()?;

    Ok(HirFn {
        id:         next_id(),
        decorators: decorators.iter().map(lower_decorator).collect(),
        name:       name.to_string(),
        uses:       uses.to_vec(),
        params:     hir_params,
        ret:        lower_sovereign_ty(ret),
        body:       hir_body,
    })
}

fn lower_struct(
    name:   &str,
    fields: &[axon_parse::Field],
) -> HirResult<HirStruct> {
    let hir_fields = fields.iter()
        .map(|f| Ok(HirField {
            name: f.name.clone(),
            ty:   lower_sovereign_ty(&f.ty),
        }))
        .collect::<HirResult<Vec<_>>>()?;

    Ok(HirStruct {
        id:     next_id(),
        name:   name.to_string(),
        fields: hir_fields,
    })
}

// Lower SovereignTy to HirTy — preserves all wrappers.
fn lower_sovereign_ty(ty: &SovereignTy) -> HirTy {
    match ty {
        SovereignTy::Plain(t)              => lower_ty(t),
        SovereignTy::Tainted(inner)        => HirTy::Tainted(Box::new(lower_ty(inner))),
        SovereignTy::Clean(inner)          => HirTy::Clean(Box::new(lower_ty(inner))),
        SovereignTy::Secret(inner)         => HirTy::Secret(Box::new(lower_ty(inner))),
        SovereignTy::Auditable(inner)      => HirTy::Auditable(Box::new(lower_ty(inner))),
        SovereignTy::Expires { inner, after } => HirTy::Expires {
            inner:    Box::new(lower_ty(inner)),
            after_ms: parse_duration_ms(after),
        },
        SovereignTy::Resident { inner, jurisdiction } => HirTy::Resident {
            inner:        Box::new(lower_ty(inner)),
            jurisdiction: jurisdiction.clone(),
        },
        SovereignTy::Money { currency, precision } => HirTy::Money {
            currency:  currency.clone(),
            precision: *precision,
        },
        SovereignTy::SafeInt { lo, hi }    => HirTy::SafeInt { lo: *lo, hi: *hi },
        SovereignTy::Refinement { base, pred } => HirTy::Refinement {
            base: Box::new(lower_ty(base)),
            pred: pred.clone(),
        },
        SovereignTy::Opaque { name, inner } => HirTy::Opaque {
            name:  name.clone(),
            inner: Box::new(lower_ty(inner)),
        },
    }
}

// Parse simple duration strings to milliseconds.
// P55.5: basic cases only — full parser in P55.6.
fn parse_duration_ms(s: &str) -> i64 {
    if s.ends_with("min") {
        s.trim_end_matches("min").trim().parse::<i64>().unwrap_or(0) * 60_000
    } else if s.ends_with("ms") {
        s.trim_end_matches("ms").trim().parse::<i64>().unwrap_or(0)
    } else if s.ends_with('s') {
        s.trim_end_matches('s').trim().parse::<i64>().unwrap_or(0) * 1_000
    } else if s.ends_with('h') {
        s.trim_end_matches('h').trim().parse::<i64>().unwrap_or(0) * 3_600_000
    } else {
        0
    }
}

// Lower a plain TypeExpr to HirTy — unchanged from P51.
fn lower_ty(ty: &TypeExpr) -> HirTy {
    if ty.name.is_empty() { HirTy::Infer }
    else { HirTy::Named(ty.name.clone()) }
}

// Lower AST Decorator to HirDecorator.
fn lower_decorator(d: &Decorator) -> HirDecorator {
    match d {
        Decorator::Deterministic              => HirDecorator::Deterministic,
        Decorator::ConstantTime               => HirDecorator::ConstantTime,
        Decorator::AiSpecialize(s)            => HirDecorator::AiSpecialize(s.clone()),
        Decorator::AiIntent(s)                => HirDecorator::AiIntent(s.clone()),
        Decorator::AiVerify { pre, post, invariant } => HirDecorator::AiVerify {
            pre: pre.clone(), post: post.clone(), invariant: invariant.clone(),
        },
        Decorator::Ensures(s)                 => HirDecorator::Ensures(s.clone()),
        Decorator::RequiresConsent { user_id, purpose } => HirDecorator::RequiresConsent {
            user_id: user_id.clone(), purpose: purpose.clone(),
        },
        Decorator::SealedMemory               => HirDecorator::SealedMemory,
        Decorator::Balanced                   => HirDecorator::Balanced,
        Decorator::AtomicFinancial            => HirDecorator::AtomicFinancial,
        Decorator::ModelSigned(s)             => HirDecorator::ModelSigned(s.clone()),
        Decorator::InferenceBudget { tokens, time_ms } => HirDecorator::InferenceBudget {
            tokens: *tokens, time_ms: *time_ms,
        },
        Decorator::RequiresHumanApproval(s)   => HirDecorator::RequiresHumanApproval(s.clone()),
    }
}

fn lower_stmt(stmt: &Stmt) -> HirResult<HirStmt> {
    match stmt {
        Stmt::Let { name, mutable, value } => {
            Ok(HirStmt::Let {
                name:    name.clone(),
                mutable: *mutable,
                ty:      HirTy::Infer,
                value:   lower_expr(value)?,
            })
        }
        // P55.5: ephemeral binding — zeroized on drop at P55.6 runtime
        Stmt::LetAt { name, value } => {
            Ok(HirStmt::LetAt {
                name:  name.clone(),
                ty:    HirTy::Infer,
                value: lower_expr(value)?,
            })
        }
        Stmt::Return(expr)    => Ok(HirStmt::Return(lower_expr(expr)?)),
        Stmt::ExprStmt(expr)  => Ok(HirStmt::ExprStmt(lower_expr(expr)?)),
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
            if stmts.is_empty() { return Ok(HirExpr::Nil); }
            if let Some(Stmt::Return(e)) = stmts.last() {
                return lower_expr(e);
            }
            Ok(HirExpr::Nil)
        }
        Expr::If { cond, then, else_ } => {
            Ok(HirExpr::If {
                cond:  Box::new(lower_expr(cond)?),
                then:  lower_block_expr(then)?,
                else_: match else_ {
                    Some(e) => Some(lower_block_expr(e)?),
                    None    => None,
                },
            })
        }
        // P55.5: v0.3 expression nodes
        Expr::Pipe { lhs, rhs, contract } => {
            Ok(HirExpr::Pipe {
                lhs:      Box::new(lower_expr(lhs)?),
                rhs:      Box::new(lower_expr(rhs)?),
                contract: contract.clone(),
            })
        }
        Expr::Morph { expr, method } => {
            Ok(HirExpr::Morph {
                expr:   Box::new(lower_expr(expr)?),
                method: method.clone(),
            })
        }
        Expr::CapPinCall { expr, method, pin } => {
            Ok(HirExpr::CapPinCall {
                expr:   Box::new(lower_expr(expr)?),
                method: method.clone(),
                pin:    match pin {
                    CapPin::Required => HirCapPin::Required,
                    CapPin::Optional => HirCapPin::Optional,
                },
            })
        }
        Expr::Temporal(kind) => {
            Ok(HirExpr::Temporal(match kind {
                TemporalKind::Now      => HirTemporalKind::Now,
                TemporalKind::Lifetime => HirTemporalKind::Lifetime,
                TemporalKind::Epoch    => HirTemporalKind::Epoch,
            }))
        }
        Expr::Foreach { var, gen, body } => {
            Ok(HirExpr::Foreach {
                var:  var.clone(),
                gen:  Box::new(lower_expr(gen)?),
                body: body.iter().map(lower_stmt).collect::<HirResult<Vec<_>>>()?,
            })
        }
        Expr::Yield(expr) => {
            Ok(HirExpr::Yield(Box::new(lower_expr(expr)?)))
        }
        Expr::IntentBlock { modes, body } => {
            Ok(HirExpr::IntentBlock {
                modes: modes.clone(),
                body:  body.iter().map(lower_stmt).collect::<HirResult<Vec<_>>>()?,
            })
        }
    }
}

fn lower_block_expr(expr: &Expr) -> HirResult<Vec<HirStmt>> {
    match expr {
        Expr::Block(stmts) => stmts.iter().map(lower_stmt).collect(),
        other => Ok(vec![HirStmt::ExprStmt(lower_expr(other)?)]),
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
