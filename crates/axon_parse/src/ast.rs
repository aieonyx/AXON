// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
//
// AXONYX AST node definitions — source of truth for P50 parser.
// Mirrored in axon/ast.ax. P51 HIR lowering builds directly on this.

use axon_std_string::AxString;

#[derive(Debug, Clone, PartialEq)]
pub enum BinOpKind {
    Add, Sub, Mul, Div, Mod,
    Eq, Ne, Lt, Le, Gt, Ge,
    And, Or,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOpKind { Neg, Not }

#[derive(Debug, Clone, PartialEq)]
pub struct TypeExpr {
    pub name: AxString,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Param {
    pub name: AxString,
    pub ty:   TypeExpr,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Field {
    pub name: AxString,
    pub ty:   TypeExpr,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    IntLit(i64),
    FloatLit(f64),
    StringLit(AxString),
    BoolLit(bool),
    Nil,
    Ident(AxString),
    BinOp {
        op:  BinOpKind,
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },
    UnaryOp {
        op:   UnaryOpKind,
        expr: Box<Expr>,
    },
    Call {
        name: AxString,
        args: Vec<Expr>,
    },
    Block(Vec<Stmt>),
    If {
        cond:  Box<Expr>,
        then:  Box<Expr>,
        else_: Option<Box<Expr>>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Let { name: AxString, mutable: bool, value: Expr },
    Return(Expr),
    ExprStmt(Expr),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Item {
    Fn {
        name:   AxString,
        params: Vec<Param>,
        ret:    TypeExpr,
        body:   Vec<Stmt>,
    },
    Struct {
        name:   AxString,
        fields: Vec<Field>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub items: Vec<Item>,
}

impl Program {
    pub fn new() -> Self {
        Program { items: Vec::new() }
    }
}

impl Default for Program {
    fn default() -> Self {
        Self::new()
    }
}
