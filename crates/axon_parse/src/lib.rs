// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
//
// axon_parse — Recursive descent parser for AXONYX.
// Internal parser layer. Not ARPi-exposed.
// External exposure via ARPi boundary defined at AWP layer.
//
// P50 landmark: AXON grammar written in .ax source files.
// Bridge (bridge.rs) mirrors parse.ax exactly — excised at P55.

pub mod ast;
pub mod bridge;
pub mod error;

pub use ast::{
    BinOpKind, Expr, Field, Item, Param, Program,
    Stmt, TypeExpr, UnaryOpKind,
};
pub use bridge::parse;
pub use error::{ParseError, ParseResult};
