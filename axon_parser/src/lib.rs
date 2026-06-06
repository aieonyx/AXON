// axon_parser/src/lib.rs
// AXON Parser — Phase 8
// Copyright © 2026 Edison Lepiten — AIEONYX
// github.com/aieonyx/axon

pub mod lexer;
pub mod parser;
pub mod sec;
pub mod tvt;

pub use parser::{parse, Item, Expr, Stmt, Ty, Pat, FnSig, Contract, ContractKind, ParseError};

pub mod hir;

pub mod infer;

pub mod codegen;

pub mod axon_std;

pub mod profile;
pub mod capflow;
pub mod mono;
pub mod borrow;
#[cfg(test)]
mod verify_integration;
pub mod driver;
