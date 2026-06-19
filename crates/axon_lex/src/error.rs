// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0

use axon_std_string::AxString;

#[derive(Debug, PartialEq)]
pub enum LexError {
    UnterminatedString,
    UnterminatedComment,
    InvalidLiteral(AxString),
}

impl core::fmt::Display for LexError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            LexError::UnterminatedString   => write!(f, "lex: unterminated string literal"),
            LexError::UnterminatedComment  => write!(f, "lex: unterminated block comment"),
            LexError::InvalidLiteral(s)    => write!(f, "lex: invalid literal: {}", s.as_str()),
        }
    }
}

pub type LexResult<T> = Result<T, LexError>;
