// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
//
// AxString — sovereign heap string with SSO (Small String Optimization).
// Strings <= SSO_CAP bytes live entirely on the stack — zero heap cost.
// This covers the vast majority of AXON tokens (keywords, identifiers, operators).

use crate::axchar::AxChar;

const SSO_CAP: usize = 23;

/// Internal representation — 24 bytes total on the stack.
enum Inner {
    /// Stack-allocated path: len + inline buffer
    Sso { len: u8, buf: [u8; SSO_CAP] },
    /// Heap-allocated path: standard Vec<u8> backing
    Heap(Vec<u8>),
}

pub struct AxString {
    inner: Inner,
}

impl AxString {
    /// Create an empty AxString. No heap allocation.
    pub fn new() -> Self {
        AxString {
            inner: Inner::Sso { len: 0, buf: [0u8; SSO_CAP] },
        }
    }

    /// Create with pre-allocated heap capacity.
    pub fn with_capacity(cap: usize) -> Self {
        if cap <= SSO_CAP {
            Self::new()
        } else {
            AxString { inner: Inner::Heap(Vec::with_capacity(cap)) }
        }
    }

    /// Construct from a &str slice.
    pub fn ax_from_str(s: &str) -> Self {
        let bytes = s.as_bytes();
        if bytes.len() <= SSO_CAP {
            let mut buf = [0u8; SSO_CAP];
            buf[..bytes.len()].copy_from_slice(bytes);
            AxString { inner: Inner::Sso { len: bytes.len() as u8, buf } }
        } else {
            AxString { inner: Inner::Heap(bytes.to_vec()) }
        }
    }

    /// Construct from raw UTF-8 bytes. Returns None if not valid UTF-8.
    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        std::str::from_utf8(bytes).ok().map(Self::ax_from_str)
    }

    /// Return the byte length.
    pub fn len(&self) -> usize {
        match &self.inner {
            Inner::Sso { len, .. } => *len as usize,
            Inner::Heap(v) => v.len(),
        }
    }

    /// Return the Unicode scalar count (not byte length).
    pub fn char_count(&self) -> usize {
        self.as_str().chars().count()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Borrow as a &str.
    pub fn as_str(&self) -> &str {
        match &self.inner {
            Inner::Sso { len, buf } => {
                // Safety: bytes were validated UTF-8 on construction
                unsafe { std::str::from_utf8_unchecked(&buf[..*len as usize]) }
            }
            Inner::Heap(v) => {
                // Safety: bytes were validated UTF-8 on construction
                unsafe { std::str::from_utf8_unchecked(v) }
            }
        }
    }

    /// Return true if this string is stack-allocated (SSO path).
    pub fn is_sso(&self) -> bool {
        matches!(self.inner, Inner::Sso { .. })
    }

    /// Promote from SSO to heap if needed, then append bytes.
    fn push_bytes(&mut self, bytes: &[u8]) {
        let new_len = self.len() + bytes.len();
        match &mut self.inner {
            Inner::Sso { len, buf } => {
                if new_len <= SSO_CAP {
                    let old = *len as usize;
                    buf[old..old + bytes.len()].copy_from_slice(bytes);
                    *len = new_len as u8;
                } else {
                    // Promote to heap
                    let mut v = Vec::with_capacity(new_len);
                    v.extend_from_slice(&buf[..*len as usize]);
                    v.extend_from_slice(bytes);
                    self.inner = Inner::Heap(v);
                }
            }
            Inner::Heap(v) => v.extend_from_slice(bytes),
        }
    }

    /// Append a &str.
    pub fn push_str(&mut self, other: &str) {
        self.push_bytes(other.as_bytes());
    }

    /// Append a single AxChar.
    pub fn push_char(&mut self, ch: AxChar) {
        let mut buf = [0u8; 4];
        let n = ch.encode_utf8(&mut buf);
        self.push_bytes(&buf[..n]);
    }

    /// Clear all content. Stays on current allocation path.
    pub fn clear(&mut self) {
        match &mut self.inner {
            Inner::Sso { len, .. } => *len = 0,
            Inner::Heap(v) => v.clear(),
        }
    }

    /// Concatenate two AxStrings into a new one.
    pub fn concat(a: &AxString, b: &AxString) -> AxString {
        let mut result = AxString::with_capacity(a.len() + b.len());
        result.push_str(a.as_str());
        result.push_str(b.as_str());
        result
    }

    pub fn contains(&self, pat: &str) -> bool {
        self.as_str().contains(pat)
    }

    pub fn starts_with(&self, pat: &str) -> bool {
        self.as_str().starts_with(pat)
    }

    pub fn ends_with(&self, pat: &str) -> bool {
        self.as_str().ends_with(pat)
    }

    pub fn trim(&self) -> AxString {
        AxString::ax_from_str(self.as_str().trim())
    }

    pub fn to_uppercase(&self) -> AxString {
        AxString::ax_from_str(&self.as_str().to_uppercase())
    }

    pub fn to_lowercase(&self) -> AxString {
        AxString::ax_from_str(&self.as_str().to_lowercase())
    }

    pub fn split<'a>(&'a self, pat: &str) -> Vec<&'a str> {
        self.as_str().split(pat).collect()
    }

    pub fn replace(&self, from: &str, to: &str) -> AxString {
        AxString::ax_from_str(&self.as_str().replace(from, to))
    }
}

impl Default for AxString {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for AxString {
    fn clone(&self) -> Self {
        AxString::ax_from_str(self.as_str())
    }
}

impl PartialEq for AxString {
    fn eq(&self, other: &Self) -> bool {
        self.as_str() == other.as_str()
    }
}

impl Eq for AxString {}

impl core::fmt::Debug for AxString {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "AxString({:?})", self.as_str())
    }
}

impl core::fmt::Display for AxString {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
