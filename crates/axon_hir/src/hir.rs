// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
//
// Rust mirror of hir.ax — updated at P55.5.
// DO NOT edit independently — hir.ax is the source of truth.

pub type HirId = usize;

#[derive(Debug, Clone, PartialEq)]
pub enum HirTy {
    Named(String),
    Infer,
    // v0.3 sovereign types (P55.5)
    Tainted(Box<HirTy>),
    Clean(Box<HirTy>),
    Secret(Box<HirTy>),
    Auditable(Box<HirTy>),
    Expires  { inner: Box<HirTy>, after_ms: i64 },
    Resident { inner: Box<HirTy>, jurisdiction: String },
    Money    { currency: String, precision: i64 },
    SafeInt  { lo: i64, hi: i64 },
    Refinement { base: Box<HirTy>, pred: String },
    Opaque   { name: String, inner: Box<HirTy> },
}

#[derive(Debug, Clone, PartialEq)]
pub enum HirDecorator {
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
pub enum HirBinOp {
    Add, Sub, Mul, Div, Mod,
    Eq, Ne, Lt, Le, Gt, Ge,
    And, Or,
}

#[derive(Debug, Clone, PartialEq)]
pub enum HirUnaryOp { Neg, Not }

#[derive(Debug, Clone, PartialEq)]
pub enum HirCapPin { Required, Optional }

#[derive(Debug, Clone, PartialEq)]
pub enum HirTemporalKind { Now, Lifetime, Epoch }

#[derive(Debug, Clone, PartialEq)]
pub enum HirExpr {
    IntLit(i64),
    FloatLit(f64),
    StringLit(String),
    BoolLit(bool),
    Nil,
    Var(String),
    BinOp  { op: HirBinOp, lhs: Box<HirExpr>, rhs: Box<HirExpr> },
    UnaryOp{ op: HirUnaryOp, expr: Box<HirExpr> },
    Call   { name: String, args: Vec<HirExpr> },
    If     { cond: Box<HirExpr>, then: Vec<HirStmt>, else_: Option<Vec<HirStmt>> },
    // v0.3
    Pipe       { lhs: Box<HirExpr>, rhs: Box<HirExpr>, contract: Option<String> },
    Morph      { expr: Box<HirExpr>, method: String },
    CapPinCall { expr: Box<HirExpr>, method: String, pin: HirCapPin },
    Temporal(HirTemporalKind),
    Foreach    { var: String, gen: Box<HirExpr>, body: Vec<HirStmt> },
    Yield(Box<HirExpr>),
    IntentBlock{ modes: Vec<String>, body: Vec<HirStmt> },
}

#[derive(Debug, Clone, PartialEq)]
pub enum HirStmt {
    Let   { name: String, mutable: bool, ty: HirTy, value: HirExpr },
    LetAt { name: String, ty: HirTy, value: HirExpr },
    Return(HirExpr),
    ExprStmt(HirExpr),
}

#[derive(Debug, Clone, PartialEq)]
pub struct HirParam { pub name: String, pub ty: HirTy }

#[derive(Debug, Clone, PartialEq)]
pub struct HirField { pub name: String, pub ty: HirTy }

#[derive(Debug, Clone, PartialEq)]
pub struct HirHandleBlock {
    pub msg_name: String,
    pub msg_ty:   HirTy,
    pub ret_ty:   HirTy,
    pub body:     Vec<HirStmt>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct HirFn {
    pub id:         HirId,
    pub decorators: Vec<HirDecorator>,
    pub name:       String,
    pub uses:       Vec<String>,
    pub params:     Vec<HirParam>,
    pub ret:        HirTy,
    pub body:       Vec<HirStmt>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct HirStruct {
    pub id:     HirId,
    pub name:   String,
    pub fields: Vec<HirField>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct HirActor {
    pub id:      HirId,
    pub name:    String,
    pub handles: Vec<HirHandleBlock>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct HirProgram {
    pub fns:     Vec<HirFn>,
    pub structs: Vec<HirStruct>,
    pub actors:  Vec<HirActor>,
}

pub fn hir_program_new() -> HirProgram {
    HirProgram { fns: vec![], structs: vec![], actors: vec![] }
}
