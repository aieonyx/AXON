// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
// Glyph bitmap and metrics -- sovereign implementation.
// Clean-room: BDF font format concepts from Adobe BDF spec only.
// P62.0: 8x8 monochrome bitmap glyphs.
// P62.1: TrueType outline glyphs via axon_gpu rasterizer.
use crate::error::{FontError, FontResult};

pub const GLYPH_WIDTH:  usize = 8;
pub const GLYPH_HEIGHT: usize = 8;

#[derive(Debug, Clone, PartialEq)]
pub struct GlyphMetrics {
    pub width:       usize,
    pub height:      usize,
    pub advance_x:   i32,
    pub bearing_x:   i32,
    pub bearing_y:   i32,
}

impl GlyphMetrics {
    pub fn monospace_8x8() -> Self {
        GlyphMetrics {
            width:     GLYPH_WIDTH,
            height:    GLYPH_HEIGHT,
            advance_x: GLYPH_WIDTH as i32,
            bearing_x: 0,
            bearing_y: GLYPH_HEIGHT as i32,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Glyph {
    pub codepoint: u32,
    pub metrics:   GlyphMetrics,
    /// Bitmap: one byte per row, MSB = leftmost pixel.
    /// 8 bytes for an 8x8 glyph.
    pub bitmap:    [u8; GLYPH_HEIGHT],
}

impl Glyph {
    pub fn new(codepoint: u32, bitmap: [u8; GLYPH_HEIGHT]) -> Self {
        Glyph {
            codepoint,
            metrics: GlyphMetrics::monospace_8x8(),
            bitmap,
        }
    }

    /// Returns true if pixel at (x, y) is set.
    pub fn pixel(&self, x: usize, y: usize) -> FontResult<bool> {
        if x >= GLYPH_WIDTH || y >= GLYPH_HEIGHT {
            return Err(FontError::InvalidBitmap);
        }
        Ok((self.bitmap[y] >> (7 - x)) & 1 == 1)
    }

    /// Render glyph to an RGBA pixel buffer (4 bytes per pixel).
    /// fg: foreground color [r,g,b,a], bg: background color [r,g,b,a]
    pub fn render_rgba(&self, fg: [u8;4], bg: [u8;4]) -> Vec<u8> {
        let mut buf = vec![0u8; GLYPH_WIDTH * GLYPH_HEIGHT * 4];
        for y in 0..GLYPH_HEIGHT {
            for x in 0..GLYPH_WIDTH {
                let set = (self.bitmap[y] >> (7 - x)) & 1 == 1;
                let color = if set { fg } else { bg };
                let idx = (y * GLYPH_WIDTH + x) * 4;
                buf[idx..idx+4].copy_from_slice(&color);
            }
        }
        buf
    }

    /// Count set pixels in the glyph bitmap.
    pub fn pixel_count(&self) -> usize {
        self.bitmap.iter().map(|b| b.count_ones() as usize).sum()
    }
}
