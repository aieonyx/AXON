// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
// axon_layout error types.
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum LayoutError {
    InvalidSize { width: f32, height: f32 },
    InvalidRect,
    TextTooLong(usize),
    LayoutOverflow,
    InvalidNode(String),
}

impl fmt::Display for LayoutError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            LayoutError::InvalidSize { width, height } =>
                write!(f, "invalid size: {}x{}", width, height),
            LayoutError::InvalidRect    => write!(f, "invalid rect"),
            LayoutError::TextTooLong(n) => write!(f, "text too long: {} chars", n),
            LayoutError::LayoutOverflow => write!(f, "layout overflow"),
            LayoutError::InvalidNode(s) => write!(f, "invalid node: {}", s),
        }
    }
}

pub type LayoutResult<T> = Result<T, LayoutError>;
