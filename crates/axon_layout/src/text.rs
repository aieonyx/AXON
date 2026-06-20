// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
// Text metrics and line breaking -- sovereign implementation.
// Clean-room: Unicode line breaking algorithm concepts from UAX #14.
// P60.0: monospace metrics approximation. P60.1: font-aware metrics.
use crate::rect::Size;
use crate::error::{LayoutError, LayoutResult};

pub const MAX_TEXT_LEN: usize = 65_536;

#[derive(Debug, Clone)]
pub struct TextStyle {
    pub font_size:   f32,
    pub line_height: f32,
    pub char_width:  f32,
}

impl TextStyle {
    pub fn new(font_size: f32) -> Self {
        TextStyle {
            font_size,
            line_height: font_size * 1.4,
            char_width:  font_size * 0.6,
        }
    }
    pub fn default() -> Self { Self::new(16.0) }
    pub fn small()   -> Self { Self::new(12.0) }
    pub fn large()   -> Self { Self::new(24.0) }
}

#[derive(Debug, Clone)]
pub struct TextMetrics {
    pub width:    f32,
    pub height:   f32,
    pub lines:    usize,
    pub chars:    usize,
}

pub fn measure_text(text: &str, style: &TextStyle, max_width: f32) -> LayoutResult<TextMetrics> {
    if text.len() > MAX_TEXT_LEN {
        return Err(LayoutError::TextTooLong(text.len()));
    }
    if text.is_empty() {
        return Ok(TextMetrics { width: 0.0, height: 0.0, lines: 0, chars: 0 });
    }
    let chars = text.chars().count();
    let lines = break_lines(text, style, max_width);
    let max_line_width = text.lines()
        .map(|l| l.chars().count() as f32 * style.char_width)
        .fold(0.0f32, f32::max)
        .min(max_width);
    Ok(TextMetrics {
        width:  max_line_width,
        height: lines as f32 * style.line_height,
        lines,
        chars,
    })
}

pub fn break_lines(text: &str, style: &TextStyle, max_width: f32) -> usize {
    if max_width <= 0.0 { return text.lines().count().max(1); }
    let chars_per_line = (max_width / style.char_width).floor() as usize;
    if chars_per_line == 0 { return text.len(); }
    let mut lines = 0usize;
    for paragraph in text.split('\n') {
        if paragraph.is_empty() { lines += 1; continue; }
        let words: Vec<&str> = paragraph.split_whitespace().collect();
        if words.is_empty() { lines += 1; continue; }
        let mut current_len = 0usize;
        lines += 1;
        for word in &words {
            let word_len = word.chars().count();
            if current_len == 0 {
                current_len = word_len;
            } else if current_len + 1 + word_len <= chars_per_line {
                current_len += 1 + word_len;
            } else {
                lines += 1;
                current_len = word_len;
            }
        }
    }
    lines.max(1)
}

pub fn text_fits_in(text: &str, style: &TextStyle, size: &Size) -> bool {
    let metrics = measure_text(text, style, size.width);
    match metrics {
        Ok(m) => m.height <= size.height,
        Err(_) => false,
    }
}
