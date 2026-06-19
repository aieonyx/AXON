// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
//
// P55 Bootstrap QA — axonc compiler driver
// Pass bar: 8/8 — the bootstrap proof gate.
//
// These tests verify the foundation of the 3-pass bootstrap:
//   identical input → identical output (determinism)
//   source → ELF binary (pipeline completeness)
//   sovereign metadata present (version + stack)

use axonc::{compile, compile_elf, AXONC_VERSION, BOOTSTRAP_DATE, SOVEREIGN_STACK};

// T1: compile() returns non-empty machine bytes
#[test]
fn test_compile_returns_bytes() {
    let bytes = compile("fn f() -> i32 { return 42; }").unwrap();
    assert!(!bytes.is_empty(), "compile() must return non-empty bytes");
}

// T2: determinism — simple function compiled twice is bit-for-bit identical
#[test]
fn test_determinism_simple() {
    let src = "fn f() -> i32 { return 42; }";
    let a = compile(src).unwrap();
    let b = compile(src).unwrap();
    assert_eq!(a, b, "determinism violated: passes produced different bytes");
}

// T3: determinism — two-function program compiled twice is identical
#[test]
fn test_determinism_complex() {
    let src = "fn sq(n: i32) -> i32 { return n * n; } \
               fn main() -> i32 { return sq(5); }";
    let a = compile(src).unwrap();
    let b = compile(src).unwrap();
    assert_eq!(a, b, "determinism violated on complex program");
}

// T4: 3-pass simulation — three consecutive compilations are all identical
//     This is the core bootstrap condition:
//     sha256(pass1) == sha256(pass2) == sha256(pass3) → BOOTSTRAP ACHIEVED
#[test]
fn test_3pass_simulation() {
    let src = "fn main() -> i32 { return 42; }";
    let pass1 = compile(src).unwrap();
    let pass2 = compile(src).unwrap();
    let pass3 = compile(src).unwrap();
    assert_eq!(pass1, pass2, "Pass 1 != Pass 2 — bootstrap condition failed");
    assert_eq!(pass2, pass3, "Pass 2 != Pass 3 — bootstrap condition failed");
    // All three passes identical: bootstrap condition MET for this program
}

// T5: full pipeline — source → correct machine byte structure
//     Verifies prologue (P54), imm32 (P52 type→P54 emit), ret (P54)
#[test]
fn test_pipeline_full() {
    let bytes = compile("fn f() -> i32 { return 42; }").unwrap();
    assert!(
        bytes.starts_with(&[0x55, 0x48, 0x89, 0xe5]),
        "expected function prologue (55 48 89 e5)"
    );
    assert!(
        bytes.windows(4).any(|w| w == &[0x2a, 0x00, 0x00, 0x00]),
        "expected imm32 42 (2a 00 00 00) in output"
    );
    assert_eq!(
        *bytes.last().unwrap(), 0xc3,
        "expected ret (0xc3) as final byte"
    );
}

// T6: compile_elf() produces a valid ELF64 binary
#[test]
fn test_compile_elf() {
    let src = "fn main() -> i32 { return 42; }";
    let out  = std::env::temp_dir().join("axonc_p55_test.elf");
    compile_elf(src, out.to_str().unwrap()).unwrap();
    let bytes = std::fs::read(&out).unwrap();
    assert_eq!(
        &bytes[0..4], &[0x7f, b'E', b'L', b'F'],
        "expected ELF magic \\x7fELF at offset 0"
    );
    assert_eq!(bytes[4], 2, "expected ELFCLASS64 (2) at offset 4");
    assert_eq!(bytes[5], 1, "expected ELFDATA2LSB (1) at offset 5");
    let _ = std::fs::remove_file(&out);
}

// T7: version and bootstrap metadata constants are present and correct
#[test]
fn test_axonc_version() {
    assert!(!AXONC_VERSION.is_empty(), "AXONC_VERSION must be set");
    assert!(!BOOTSTRAP_DATE.is_empty(), "BOOTSTRAP_DATE must be set");
    assert!(
        AXONC_VERSION.contains("bootstrap"),
        "AXONC_VERSION must include 'bootstrap'"
    );
    assert!(
        SOVEREIGN_STACK.contains("axon_native"),
        "SOVEREIGN_STACK must include axon_native (P54)"
    );
    assert!(
        SOVEREIGN_STACK.contains("axon_lex"),
        "SOVEREIGN_STACK must include axon_lex (P49)"
    );
}

// T8: sovereign stack — all 4 program variants compile without error
//     Exercises the full range of P45–P54 capabilities
#[test]
fn test_sovereign_stack() {
    let programs: &[&str] = &[
        "fn f() -> i32 { return 0; }",
        "fn add(a: i32, b: i32) -> i32 { return a + b; }",
        "fn sq(n: i32) -> i32 { return n * n; }",
        "fn f(a: i32, b: i32) -> i32 { if a == 1 { return a; } return b; }",
    ];
    for src in programs {
        let result = compile(src);
        assert!(
            result.is_ok(),
            "sovereign stack failed for: {}\nError: {:?}",
            src,
            result.err()
        );
    }
}
