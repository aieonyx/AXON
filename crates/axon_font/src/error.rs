// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
// axon_font error types.
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum FontError {
    GlyphNotFound(u32),
    InvalidBitmap,
    RasterFailed(String),
    FontTooLarge(usize),
    InvalidCodepoint(u32),
}

impl fmt::Display for FontError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            FontError::GlyphNotFound(cp)  => write!(f, "glyph not found: U+{:04X}", cp),
            FontError::InvalidBitmap      => write!(f, "invalid glyph bitmap"),
            FontError::RasterFailed(s)    => write!(f, "raster failed: {}", s),
            FontError::FontTooLarge(n)    => write!(f, "font too large: {} glyphs", n),
            FontError::InvalidCodepoint(c)=> write!(f, "invalid codepoint: U+{:04X}", c),
        }
    }
}

pub type FontResult<T> = Result<T, FontError>;
