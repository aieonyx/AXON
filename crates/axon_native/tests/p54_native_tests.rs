// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
// P54 QA -- axon_native test suite
// Pass bar: 10/10 before P55 begins.

use axon_native::{
    mov_rax_imm32, mov_rbp_rsp, push_rbp, ret_byte,
    native_codegen_source,
};

fn contains_seq(haystack: &[u8], needle: &[u8]) -> bool {
    haystack.windows(needle.len()).any(|w| w == needle)
}

// T1: push rbp = [0x55]
#[test]
fn test_bytes_push_rbp() {
    assert_eq!(push_rbp(), vec![0x55u8]);
}

// T2: mov rbp, rsp = [0x48, 0x89, 0xe5]
#[test]
fn test_bytes_mov_rbp_rsp() {
    assert_eq!(mov_rbp_rsp(), vec![0x48u8, 0x89, 0xe5]);
}

// T3: ret = [0xc3]
#[test]
fn test_bytes_ret() {
    assert_eq!(ret_byte(), vec![0xc3u8]);
}

// T4: mov rax, imm32(42) = [0x48, 0xc7, 0xc0, 0x2a, 0x00, 0x00, 0x00]
#[test]
fn test_bytes_mov_rax_imm32() {
    assert_eq!(mov_rax_imm32(42), vec![0x48u8, 0xc7, 0xc0, 0x2a, 0x00, 0x00, 0x00]);
}

// T5: function starts with prologue bytes 55 48 89 e5
#[test]
fn test_native_prologue() {
    let bytes = native_codegen_source("fn f() -> i32 { return 42; }").unwrap();
    assert!(bytes.starts_with(&[0x55, 0x48, 0x89, 0xe5]),
        "expected prologue 55 48 89 e5, got: {:02x?}", &bytes[..4.min(bytes.len())]);
}

// T6: return 42 — bytes contain imm32 0x2a and end with 0xc3
#[test]
fn test_native_int_return() {
    let bytes = native_codegen_source("fn f() -> i32 { return 42; }").unwrap();
    assert!(contains_seq(&bytes, &[0x2a, 0x00, 0x00, 0x00]),
        "expected imm32 0x2a in: {:02x?}", bytes);
    assert_eq!(*bytes.last().unwrap(), 0xc3,
        "expected final ret (0xc3), got: 0x{:02x}", bytes.last().unwrap());
}

// T7: return first param → mov rax, rdi = [0x48, 0x89, 0xf8]
#[test]
fn test_native_param_return() {
    let bytes = native_codegen_source("fn add(a: i32, b: i32) -> i32 { return a; }").unwrap();
    assert!(contains_seq(&bytes, &[0x48, 0x89, 0xf8]),
        "expected mov rax, rdi (48 89 f8) in: {:02x?}", bytes);
}

// T8: n * n → imul rax, rbx = [0x48, 0x0f, 0xaf, 0xc3]
#[test]
fn test_native_mul() {
    let bytes = native_codegen_source("fn square(n: i32) -> i32 { return n * n; }").unwrap();
    assert!(contains_seq(&bytes, &[0x48, 0x0f, 0xaf, 0xc3]),
        "expected imul rax, rbx (48 0f af c3) in: {:02x?}", bytes);
}

// T9: if expression → test rax, rax + je prefix
#[test]
fn test_native_if() {
    let src = "fn f(a: i32, b: i32) -> i32 { if a == 1 { return a; } return b; }";
    let bytes = native_codegen_source(src).unwrap();
    assert!(contains_seq(&bytes, &[0x48, 0x85, 0xc0]),
        "expected test rax,rax (48 85 c0) in: {:02x?}", bytes);
    assert!(contains_seq(&bytes, &[0x0f, 0x84]),
        "expected je prefix (0f 84) in: {:02x?}", bytes);
}

// T10: two-function program → two prologues, call instruction, ends with ret
#[test]
fn test_native_full_program() {
    let src = "fn square(n: i32) -> i32 { return n * n; } fn main() -> i32 { return square(5); }";
    let bytes = native_codegen_source(src).unwrap();

    // Two function prologues
    let prologue = &[0x55u8, 0x48, 0x89, 0xe5];
    let count = bytes.windows(4).filter(|w| *w == prologue).count();
    assert_eq!(count, 2, "expected 2 prologues, found {}", count);

    // call instruction: opcode 0xe8
    assert!(bytes.contains(&0xe8u8),
        "expected call opcode (0xe8) in: {:02x?}", &bytes[..20.min(bytes.len())]);

    // ends with ret
    assert_eq!(*bytes.last().unwrap(), 0xc3,
        "expected final ret (0xc3)");
}
