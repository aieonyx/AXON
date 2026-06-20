// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
// Text rasterizer -- renders text strings to pixel buffers.
// P62.0: 8x8 monochrome bitmap rasterization.
// P62.1: GPU-accelerated subpixel rendering via axon_gpu.
use crate::font::Font;
use crate::glyph::{GLYPH_WIDTH, GLYPH_HEIGHT};
use crate::error::{FontError, FontResult};

#[derive(Debug, Clone)]
pub struct RasterConfig {
    pub fg:      [u8; 4],  // foreground RGBA
    pub bg:      [u8; 4],  // background RGBA
    pub scale:   u32,      // pixel scale factor (1=normal, 2=2x, etc.)
}

impl RasterConfig {
    pub fn default() -> Self {
        RasterConfig { fg: [255,255,255,255], bg: [0,0,0,255], scale: 1 }
    }
    pub fn white_on_black() -> Self { Self::default() }
    pub fn black_on_white() -> Self {
        RasterConfig { fg: [0,0,0,255], bg: [255,255,255,255], scale: 1 }
    }
    pub fn with_scale(mut self, scale: u32) -> Self { self.scale = scale; self }
}

pub struct TextRaster {
    font:   Font,
    config: RasterConfig,
}

impl TextRaster {
    pub fn new(font: Font, config: RasterConfig) -> Self {
        TextRaster { font, config }
    }

    pub fn with_builtin() -> Self {
        Self::new(Font::builtin(), RasterConfig::default())
    }

    /// Raster a single line of text to an RGBA pixel buffer.
    /// Returns (buffer, width_px, height_px).
    pub fn raster_line(&self, text: &str) -> FontResult<(Vec<u8>, usize, usize)> {
        let chars: Vec<char> = text.chars().collect();
        let s = self.config.scale as usize;
        let gw = GLYPH_WIDTH  * s;
        let gh = GLYPH_HEIGHT * s;
        let width  = gw * chars.len();
        let height = gh;
        let mut buf = vec![0u8; width * height * 4];

        for (ci, &ch) in chars.iter().enumerate() {
            let glyph = self.font.glyph(ch as u32)?;
            let gx_base = ci * gw;
            for gy in 0..GLYPH_HEIGHT {
                for gx in 0..GLYPH_WIDTH {
                    let set = (glyph.bitmap[gy] >> (7 - gx)) & 1 == 1;
                    let color = if set { self.config.fg } else { self.config.bg };
                    for sy in 0..s {
                        for sx in 0..s {
                            let px = gx_base + gx * s + sx;
                            let py = gy * s + sy;
                            let idx = (py * width + px) * 4;
                            if idx + 3 < buf.len() {
                                buf[idx..idx+4].copy_from_slice(&color);
                            }
                        }
                    }
                }
            }
        }
        Ok((buf, width, height))
    }

    /// Measure text width in pixels.
    pub fn measure_width(&self, text: &str) -> usize {
        text.chars().count() * GLYPH_WIDTH * self.config.scale as usize
    }

    /// Measure text height in pixels.
    pub fn measure_height(&self) -> usize {
        GLYPH_HEIGHT * self.config.scale as usize
    }

    /// Check coverage — fraction of chars available in font.
    pub fn coverage(&self, text: &str) -> f32 {
        self.font.coverage(text)
    }
}
