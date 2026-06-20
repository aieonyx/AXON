// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
// Font collection -- sovereign glyph lookup.
use crate::glyph::Glyph;
use crate::builtin::builtin_glyph;
use crate::error::{FontError, FontResult};

pub const MAX_FONT_GLYPHS: usize = 65_536;

pub struct Font {
    pub name:   String,
    glyphs:     std::collections::HashMap<u32, Glyph>,
    fallback:   Option<Glyph>,
}

impl Font {
    pub fn new(name: &str) -> Self {
        Font { name: name.to_string(), glyphs: Default::default(), fallback: None }
    }

    pub fn builtin() -> Self {
        let mut font = Self::new("AIEONYX Sovereign 8x8");
        for cp in 0x20u32..=0x7eu32 {
            if let Some(g) = builtin_glyph(cp) {
                font.glyphs.insert(cp, g);
            }
        }
        font.fallback = builtin_glyph(0x3f); // '?' as fallback
        font
    }

    pub fn add_glyph(&mut self, glyph: Glyph) -> FontResult<()> {
        if self.glyphs.len() >= MAX_FONT_GLYPHS {
            return Err(FontError::FontTooLarge(self.glyphs.len()));
        }
        self.glyphs.insert(glyph.codepoint, glyph);
        Ok(())
    }

    pub fn glyph(&self, codepoint: u32) -> FontResult<&Glyph> {
        self.glyphs.get(&codepoint)
            .or(self.fallback.as_ref())
            .ok_or(FontError::GlyphNotFound(codepoint))
    }

    pub fn has_glyph(&self, codepoint: u32) -> bool {
        self.glyphs.contains_key(&codepoint)
    }

    pub fn glyph_count(&self) -> usize { self.glyphs.len() }

    pub fn coverage(&self, text: &str) -> f32 {
        let chars: Vec<char> = text.chars().collect();
        if chars.is_empty() { return 1.0; }
        let covered = chars.iter()
            .filter(|&&c| self.has_glyph(c as u32))
            .count();
        covered as f32 / chars.len() as f32
    }
}
