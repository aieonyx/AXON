// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
//
// AXONYX HIR node definitions — source of truth for P51 lowering.
// Mirrored in axon/hir.ax.
// P52 type inference fills HirTy::Infer slots.

use axon_std_string::AxString;

pub type HirId = usize;

// ── Types ──────────────────────────────────────────────────────────────────────

/// HIR type. Named types come from source annotations.
/// Infer slots are filled by P52 type inference.
#[derive(Debug, Clone, PartialEq)]
pub enum HirTy {
    Named(AxString),
    Infer,
}

// ── Operators ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum HirBinOp {
    Add, Sub, Mul, Div, Mod,
    Eq, Ne, Lt, Le, Gt, Ge,
    And, Or,
}

#[derive(Debug, Clone, PartialEq)]
pub enum HirUnaryOp { Neg, Not }

// ── Expressions ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum HirExpr {
    IntLit(i64),
    FloatLit(f64),
    StringLit(AxString),
    BoolLit(bool),
    Nil,
    Var(AxString),
    BinOp {
        op:  HirBinOp,
        lhs: Box<HirExpr>,
        rhs: Box<HirExpr>,
    },
    UnaryOp {
        op:   HirUnaryOp,
        expr: Box<HirExpr>,
    },
    Call {
        name: AxString,
        args: Vec<HirExpr>,
    },
    If {
        cond:  Box<HirExpr>,
        then:  Vec<HirStmt>,
        else_: Option<Vec<HirStmt>>,
    },
}

// ── Statements ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum HirStmt {
    Let {
        name:    AxString,
        mutable: bool,
        ty:      HirTy,
        value:   HirExpr,
    },
    Return(HirExpr),
    ExprStmt(HirExpr),
}

// ── Items ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub struct HirParam {
    pub name: AxString,
    pub ty:   HirTy,
}

#[derive(Debug, Clone, PartialEq)]
pub struct HirField {
    pub name: AxString,
    pub ty:   HirTy,
}

#[derive(Debug, Clone, PartialEq)]
pub struct HirFn {
    pub id:     HirId,
    pub name:   AxString,
    pub params: Vec<HirParam>,
    pub ret:    HirTy,
    pub body:   Vec<HirStmt>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct HirStruct {
    pub id:     HirId,
    pub name:   AxString,
    pub fields: Vec<HirField>,
}

// ── Program ───────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub struct HirProgram {
    pub fns:     Vec<HirFn>,
    pub structs: Vec<HirStruct>,
}

impl HirProgram {
    pub fn new() -> Self {
        HirProgram { fns: Vec::new(), structs: Vec::new() }
    }
}

impl Default for HirProgram {
    fn default() -> Self { Self::new() }
}
