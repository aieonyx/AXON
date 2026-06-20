// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
//
// Rust mirror of ast.ax — updated at P55.5.
// DO NOT edit independently — ast.ax is the source of truth.

#[derive(Debug, Clone, PartialEq)]
pub struct TypeExpr { pub name: String }

#[derive(Debug, Clone, PartialEq)]
pub enum SovereignTy {
    Tainted(Box<TypeExpr>),
    Clean(Box<TypeExpr>),
    Secret(Box<TypeExpr>),
    Auditable(Box<TypeExpr>),
    Expires  { inner: Box<TypeExpr>, after: String },
    Resident { inner: Box<TypeExpr>, jurisdiction: String },
    Money    { currency: String, precision: i64 },
    SafeInt  { lo: i64, hi: i64 },
    Refinement { base: Box<TypeExpr>, pred: String },
    Opaque   { name: String, inner: Box<TypeExpr> },
    Plain(TypeExpr),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Decorator {
    Deterministic,
    ConstantTime,
    AiSpecialize(String),
    AiIntent(String),
    AiVerify { pre: String, post: String, invariant: String },
    Ensures(String),
    RequiresConsent { user_id: String, purpose: String },
    SealedMemory,
    Balanced,
    AtomicFinancial,
    ModelSigned(String),
    InferenceBudget { tokens: i64, time_ms: i64 },
    RequiresHumanApproval(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Param { pub name: String, pub ty: SovereignTy }

#[derive(Debug, Clone, PartialEq)]
pub struct Field { pub name: String, pub ty: SovereignTy }

#[derive(Debug, Clone, PartialEq)]
pub struct HandleBlock {
    pub msg_name: String,
    pub msg_ty:   SovereignTy,
    pub ret_ty:   SovereignTy,
    pub body:     Vec<Stmt>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CapPin { Required, Optional }

#[derive(Debug, Clone, PartialEq)]
pub enum TemporalKind { Now, Lifetime, Epoch }

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    IntLit(i64),
    FloatLit(f64),
    StringLit(String),
    BoolLit(bool),
    Nil,
    Ident(String),
    BinOp  { op: BinOpKind, lhs: Box<Expr>, rhs: Box<Expr> },
    UnaryOp{ op: UnaryOpKind, expr: Box<Expr> },
    Call   { name: String, args: Vec<Expr> },
    Block(Vec<Stmt>),
    If     { cond: Box<Expr>, then: Box<Expr>, else_: Option<Box<Expr>> },
    // v0.3
    Pipe       { lhs: Box<Expr>, rhs: Box<Expr>, contract: Option<String> },
    Morph      { expr: Box<Expr>, method: String },
    CapPinCall { expr: Box<Expr>, method: String, pin: CapPin },
    Temporal(TemporalKind),
    Foreach    { var: String, gen: Box<Expr>, body: Vec<Stmt> },
    Yield(Box<Expr>),
    IntentBlock{ modes: Vec<String>, body: Vec<Stmt> },
}

#[derive(Debug, Clone, PartialEq)]
pub enum BinOpKind {
    Add, Sub, Mul, Div, Mod,
    Eq, Ne, Lt, Le, Gt, Ge,
    And, Or,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOpKind { Neg, Not }

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Let   { name: String, mutable: bool, value: Expr },
    LetAt { name: String, value: Expr },
    Return(Expr),
    ExprStmt(Expr),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Item {
    Fn {
        decorators: Vec<Decorator>,
        name:       String,
        uses:       Vec<String>,
        params:     Vec<Param>,
        ret:        SovereignTy,
        body:       Vec<Stmt>,
    },
    Struct { name: String, fields: Vec<Field> },
    Actor  { name: String, handles: Vec<HandleBlock> },
}

#[derive(Debug, Clone, PartialEq)]
pub struct Program { pub items: Vec<Item> }

pub fn program_new() -> Program { Program { items: vec![] } }
