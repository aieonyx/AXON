// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
//
// axon_lex — AXONYX sovereign lexer.
// Internal lexer layer. Not ARPi-exposed.
// External exposure via ARPi boundary defined at AWP layer.
//
// P49 landmark: first real .ax source files live in axon/ directory.
// The Rust bridge (bridge.rs) mirrors lex.ax exactly and is excised at P55.

pub mod bridge;
pub mod error;
pub mod token;

pub use bridge::{lex_all, lex_next, Cursor};
pub use error::{LexError, LexResult};
pub use token::Token;
