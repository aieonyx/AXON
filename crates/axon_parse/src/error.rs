// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0

use axon_std_string::AxString;

#[derive(Debug, PartialEq)]
pub enum ParseError {
    UnexpectedToken(AxString),
    UnexpectedEof,
    MaxDepthExceeded,
    InvalidLiteral(AxString),
}

impl core::fmt::Display for ParseError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            ParseError::UnexpectedToken(t) => write!(f, "parse: unexpected token: {}", t.as_str()),
            ParseError::UnexpectedEof      => write!(f, "parse: unexpected end of input"),
            ParseError::MaxDepthExceeded   => write!(f, "parse: maximum nesting depth exceeded"),
            ParseError::InvalidLiteral(s)  => write!(f, "parse: invalid literal: {}", s.as_str()),
        }
    }
}

pub type ParseResult<T> = Result<T, ParseError>;
