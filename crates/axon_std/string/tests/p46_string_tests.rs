// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
// P46 QA — axon_std::string test suite
// Pass bar: 12/12 before P47 begins.

use axon_std_string::{ax_format, AxChar, AxFormat, AxString};

// T1: empty string
#[test]
fn test_new_empty() {
    let s = AxString::new();
    assert_eq!(s.len(), 0);
    assert!(s.is_empty());
    assert_eq!(s.as_str(), "");
}

// T2: from_str round-trip
#[test]
fn test_from_str() {
    let s = AxString::ax_from_str("sovereign");
    assert_eq!(s.as_str(), "sovereign");
    assert_eq!(s.len(), 9);
}

// T3: SSO — strings <= 23 bytes stay on stack
#[test]
fn test_sso_stack() {
    let s = AxString::ax_from_str("axon"); // 4 bytes
    assert!(s.is_sso(), "short string must use SSO path");
    let s23 = AxString::ax_from_str("12345678901234567890123"); // exactly 23
    assert!(s23.is_sso(), "23-byte string must use SSO path");
}

// T4: heap promotion — strings > 23 bytes go to heap
#[test]
fn test_heap_promotion() {
    let s = AxString::ax_from_str("123456789012345678901234"); // 24 bytes
    assert!(!s.is_sso(), "24-byte string must use heap path");
    assert_eq!(s.len(), 24);
    assert_eq!(s.as_str(), "123456789012345678901234");
}

// T5: push_str grows correctly across SSO boundary
#[test]
fn test_push_str() {
    let mut s = AxString::ax_from_str("hello");
    s.push_str(" sovereign");
    assert_eq!(s.as_str(), "hello sovereign");
    // push past SSO boundary
    s.push_str(" axon compiler chain bootstrap");
    assert!(s.as_str().starts_with("hello sovereign axon"));
}

// T6: push_char appends Unicode correctly
#[test]
fn test_push_char() {
    let mut s = AxString::ax_from_str("axon");
    s.push_char(AxChar::from_char('!'));
    assert_eq!(s.as_str(), "axon!");
    // multi-byte Unicode
    s.push_char(AxChar::from_char('ñ'));
    assert_eq!(s.as_str(), "axon!ñ");
}

// T7: Unicode char_count vs byte len
#[test]
fn test_unicode_char_count() {
    let s = AxString::ax_from_str("héllo"); // é = 2 bytes
    assert_eq!(s.char_count(), 5);
    assert_eq!(s.len(), 6); // 5 ASCII + 1 extra byte for é
}

// T8: concat
#[test]
fn test_concat() {
    let a = AxString::ax_from_str("axon");
    let b = AxString::ax_from_str("_std");
    let c = AxString::concat(&a, &b);
    assert_eq!(c.as_str(), "axon_std");
}

// T9: trim
#[test]
fn test_trim() {
    let s = AxString::ax_from_str("  sovereign  ");
    assert_eq!(s.trim().as_str(), "sovereign");
}

// T10: contains, starts_with, ends_with
#[test]
fn test_contains_starts_ends() {
    let s = AxString::ax_from_str("axon_std_string");
    assert!(s.contains("std"));
    assert!(s.starts_with("axon"));
    assert!(s.ends_with("string"));
    assert!(!s.contains("missing"));
}

// T11: AxChar construction and classification
#[test]
fn test_axchar() {
    let c = AxChar::from_u32(b'a' as u32).unwrap();
    assert!(c.is_alphabetic());
    assert!(!c.is_numeric());
    assert_eq!(c.to_u32(), 97);

    let n = AxChar::from_u32(b'5' as u32).unwrap();
    assert!(n.is_numeric());

    let ws = AxChar::from_char(' ');
    assert!(ws.is_whitespace());

    // Invalid scalar
    assert!(AxChar::from_u32(0xD800).is_none()); // surrogate half
}

// T12: AxFormat trait produces correct output
#[test]
fn test_ax_format() {
    struct Version { major: u32, minor: u32, patch: u32 }
    impl AxFormat for Version {
        fn ax_fmt(&self, buf: &mut AxString) {
            self.major.ax_fmt(buf);
            buf.push_char(AxChar::from_char('.'));
            self.minor.ax_fmt(buf);
            buf.push_char(AxChar::from_char('.'));
            self.patch.ax_fmt(buf);
        }
    }

    let v = Version { major: 0, minor: 46, patch: 0 };
    let result = ax_format(&v);
    assert_eq!(result.as_str(), "0.46.0");
}
