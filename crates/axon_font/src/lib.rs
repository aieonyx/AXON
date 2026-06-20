// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
// axon_font -- sovereign font rendering engine.
// P62.0: 8x8 bitmap glyphs, ASCII coverage, RGBA rasterization.
// P62.1: TrueType outlines, GPU subpixel rendering.
pub mod builtin;
pub mod error;
pub mod font;
pub mod glyph;
pub mod raster;
pub use error::{FontError, FontResult};
pub use font::Font;
pub use glyph::{Glyph, GlyphMetrics, GLYPH_WIDTH, GLYPH_HEIGHT};
pub use raster::{TextRaster, RasterConfig};
pub use builtin::{builtin_glyph, builtin_glyph_count};
