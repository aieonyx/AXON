// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
// P62 QA -- axon_font sovereign font rendering tests
// Pass bar: 20/20
// P3 Doctrine: complements axon_layout P60 (text metrics), axon_gpu P58 (rendering)
use axon_font::{
    Font, Glyph, GlyphMetrics, TextRaster, RasterConfig,
    builtin_glyph, builtin_glyph_count, FontError,
    GLYPH_WIDTH, GLYPH_HEIGHT,
};

// ── Builtin glyph tests ───────────────────────────────────────────────────────
#[test]
fn test_builtin_glyph_count() {
    assert_eq!(builtin_glyph_count(), 95);
}
#[test]
fn test_builtin_space_glyph() {
    let g = builtin_glyph(0x20).unwrap();
    assert_eq!(g.codepoint, 0x20);
    assert_eq!(g.pixel_count(), 0);
}
#[test]
fn test_builtin_ascii_coverage() {
    for cp in 0x20u32..=0x7eu32 {
        assert!(builtin_glyph(cp).is_some(), "missing glyph U+{:04X}", cp);
    }
}
#[test]
fn test_builtin_unknown_returns_none() {
    assert!(builtin_glyph(0x1000).is_none());
}
#[test]
fn test_builtin_a_has_pixels() {
    let g = builtin_glyph(0x41).unwrap(); // 'A'
    assert!(g.pixel_count() > 0);
}

// ── Glyph tests ───────────────────────────────────────────────────────────────
#[test]
fn test_glyph_pixel_access() {
    let g = builtin_glyph(0x41).unwrap(); // 'A'
    // pixel() should not error for valid coords
    let result = g.pixel(0, 0);
    assert!(result.is_ok());
}
#[test]
fn test_glyph_pixel_out_of_bounds() {
    let g = builtin_glyph(0x41).unwrap();
    assert!(g.pixel(GLYPH_WIDTH, 0).is_err());
    assert!(g.pixel(0, GLYPH_HEIGHT).is_err());
}
#[test]
fn test_glyph_render_rgba_size() {
    let g = builtin_glyph(0x41).unwrap();
    let buf = g.render_rgba([255,255,255,255], [0,0,0,255]);
    assert_eq!(buf.len(), GLYPH_WIDTH * GLYPH_HEIGHT * 4);
}
#[test]
fn test_glyph_metrics_monospace() {
    let m = GlyphMetrics::monospace_8x8();
    assert_eq!(m.width,  GLYPH_WIDTH);
    assert_eq!(m.height, GLYPH_HEIGHT);
    assert_eq!(m.advance_x, GLYPH_WIDTH as i32);
}

// ── Font tests ────────────────────────────────────────────────────────────────
#[test]
fn test_font_builtin_loaded() {
    let f = Font::builtin();
    assert!(f.glyph_count() >= 95);
}
#[test]
fn test_font_has_ascii() {
    let f = Font::builtin();
    assert!(f.has_glyph(b'A' as u32));
    assert!(f.has_glyph(b'z' as u32));
    assert!(f.has_glyph(b'0' as u32));
}
#[test]
fn test_font_missing_glyph_fallback() {
    let f = Font::builtin();
    // Non-ASCII falls back to '?'
    let g = f.glyph(0x1F600); // emoji — not in builtin
    assert!(g.is_ok()); // fallback '?' should be returned
}
#[test]
fn test_font_coverage_ascii() {
    let f = Font::builtin();
    let cov = f.coverage("Hello World");
    assert!((cov - 1.0).abs() < 1e-5);
}
#[test]
fn test_font_coverage_mixed() {
    let f = Font::builtin();
    let cov = f.coverage("Hello 🌍"); // emoji not covered
    assert!(cov < 1.0);
}

// ── Raster tests ──────────────────────────────────────────────────────────────
#[test]
fn test_raster_line_size() {
    let r = TextRaster::with_builtin();
    let (buf, w, h) = r.raster_line("Hi").unwrap();
    assert_eq!(w, 2 * GLYPH_WIDTH);
    assert_eq!(h, GLYPH_HEIGHT);
    assert_eq!(buf.len(), w * h * 4);
}
#[test]
fn test_raster_measure_width() {
    let r = TextRaster::with_builtin();
    assert_eq!(r.measure_width("Hello"), 5 * GLYPH_WIDTH);
}
#[test]
fn test_raster_measure_height() {
    let r = TextRaster::with_builtin();
    assert_eq!(r.measure_height(), GLYPH_HEIGHT);
}
#[test]
fn test_raster_scale_2x() {
    let cfg = RasterConfig::default().with_scale(2);
    let r = TextRaster::new(Font::builtin(), cfg);
    let (_, w, h) = r.raster_line("A").unwrap();
    assert_eq!(w, GLYPH_WIDTH * 2);
    assert_eq!(h, GLYPH_HEIGHT * 2);
}
#[test]
fn test_raster_coverage() {
    let r = TextRaster::with_builtin();
    assert!((r.coverage("Hello") - 1.0).abs() < 1e-5);
}
