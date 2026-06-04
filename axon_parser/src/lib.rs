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
