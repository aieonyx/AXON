// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
//
// AxStr — sovereign borrowed string slice.
// Thin wrapper over &str — zero cost, zero copy.

use crate::axstring::AxString;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AxStr<'a>(&'a str);

impl<'a> AxStr<'a> {
    pub fn new(s: &'a str) -> Self {
        AxStr(s)
    }

    pub fn as_str(self) -> &'a str {
        self.0
    }

    pub fn len(self) -> usize {
        self.0.len()
    }

    pub fn is_empty(self) -> bool {
        self.0.is_empty()
    }

    pub fn to_owned(self) -> AxString {
        AxString::ax_from_str(self.0)
    }

    pub fn contains(self, pat: &str) -> bool {
        self.0.contains(pat)
    }

    pub fn starts_with(self, pat: &str) -> bool {
        self.0.starts_with(pat)
    }

    pub fn ends_with(self, pat: &str) -> bool {
        self.0.ends_with(pat)
    }
}

impl<'a> From<&'a str> for AxStr<'a> {
    fn from(s: &'a str) -> Self {
        AxStr(s)
    }
}

impl<'a> From<&'a AxString> for AxStr<'a> {
    fn from(s: &'a AxString) -> Self {
        AxStr(s.as_str())
    }
}
