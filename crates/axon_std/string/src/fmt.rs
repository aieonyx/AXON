// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
//
// AxFormat — sovereign formatting trait and engine.
// Drives all display paths in the AXON compiler chain.

use crate::axstring::AxString;

/// Sovereign formatting trait. Implement this for any type
/// that needs to produce a string representation in the AXON chain.
pub trait AxFormat {
    fn ax_fmt(&self, buf: &mut AxString);
}

/// Format a value implementing AxFormat into a new AxString.
pub fn ax_format(f: &impl AxFormat) -> AxString {
    let mut buf = AxString::new();
    f.ax_fmt(&mut buf);
    buf
}

/// Format a value into an existing AxString buffer.
pub fn ax_format_into(f: &impl AxFormat, buf: &mut AxString) {
    f.ax_fmt(buf);
}

// ── Blanket implementations for primitive types ────────────────────────────────

impl AxFormat for &str {
    fn ax_fmt(&self, buf: &mut AxString) {
        buf.push_str(self);
    }
}

impl AxFormat for AxString {
    fn ax_fmt(&self, buf: &mut AxString) {
        buf.push_str(self.as_str());
    }
}

impl AxFormat for u8 {
    fn ax_fmt(&self, buf: &mut AxString) {
        buf.push_str(&self.to_string());
    }
}

impl AxFormat for u32 {
    fn ax_fmt(&self, buf: &mut AxString) {
        buf.push_str(&self.to_string());
    }
}

impl AxFormat for u64 {
    fn ax_fmt(&self, buf: &mut AxString) {
        buf.push_str(&self.to_string());
    }
}

impl AxFormat for i32 {
    fn ax_fmt(&self, buf: &mut AxString) {
        buf.push_str(&self.to_string());
    }
}

impl AxFormat for i64 {
    fn ax_fmt(&self, buf: &mut AxString) {
        buf.push_str(&self.to_string());
    }
}

impl AxFormat for bool {
    fn ax_fmt(&self, buf: &mut AxString) {
        buf.push_str(if *self { "true" } else { "false" });
    }
}

impl AxFormat for usize {
    fn ax_fmt(&self, buf: &mut AxString) {
        buf.push_str(&self.to_string());
    }
}
