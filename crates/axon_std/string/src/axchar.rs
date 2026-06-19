// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
//
// AxChar — sovereign Unicode scalar value.
// Wraps u32; only valid Unicode scalar values are constructible.

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct AxChar(u32);

impl AxChar {
    /// Construct from a Unicode scalar value. Returns None if invalid.
    pub fn from_u32(v: u32) -> Option<Self> {
        char::from_u32(v).map(|_| AxChar(v))
    }

    /// Construct from a Rust char directly.
    pub fn from_char(c: char) -> Self {
        AxChar(c as u32)
    }

    /// Return the raw Unicode scalar value.
    pub fn to_u32(self) -> u32 {
        self.0
    }

    /// Convert to Rust char.
    pub fn to_char(self) -> char {
        // Safety: constructor guarantees valid scalar value
        char::from_u32(self.0).unwrap()
    }

    pub fn is_alphabetic(self) -> bool {
        self.to_char().is_alphabetic()
    }

    pub fn is_numeric(self) -> bool {
        self.to_char().is_numeric()
    }

    pub fn is_whitespace(self) -> bool {
        self.to_char().is_whitespace()
    }

    pub fn is_ascii(self) -> bool {
        self.0 < 128
    }

    pub fn len_utf8(self) -> usize {
        self.to_char().len_utf8()
    }

    /// Encode this char into a UTF-8 byte buffer. Returns bytes written.
    pub fn encode_utf8(self, buf: &mut [u8]) -> usize {
        self.to_char().encode_utf8(buf).len()
    }
}

impl From<char> for AxChar {
    fn from(c: char) -> Self {
        AxChar::from_char(c)
    }
}
