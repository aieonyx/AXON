// Copyright (c) 2026 Edison Lepiten / AIEONYX
// axon_aarch64 P71 — AArch64 freestanding codegen tests (20 tests)

use axon_aarch64::ir::{
    compile_to_ir, AxIr, BinOpKind,
    Aarch64Target, CompileError,
};
use axon_aarch64::emit::emit_asm;
use axon_aarch64::linker::{
    generate_linker_script, AIXOS_LOAD_ADDR, DEFAULT_STACK_SIZE,
};
use axon_aarch64::conformance::{check_conformance, ir_contains};

fn freestanding() -> Aarch64Target { Aarch64Target::Freestanding }
fn linux() -> Aarch64Target { Aarch64Target::LinuxAarch64 }

// ── T1: Target triple correct ─────────────────────────────────────────────────
#[test]
fn t1_target_triples() {
    assert_eq!(freestanding().triple(), "aarch64-unknown-none-elf");
    assert_eq!(linux().triple(), "aarch64-unknown-linux-gnu");
    assert!(freestanding().is_freestanding());
    assert!(!linux().is_freestanding());
}

// ── T2: compile_to_ir: literal assignment ────────────────────────────────────
#[test]
fn t2_ir_literal() {
    let ir = compile_to_ir(b"let x = 42\n").unwrap();
    assert!(ir.iter().any(|n| matches!(n, AxIr::LoadImm { var, value }
        if var == "x" && *value == 42)));
}

// ── T3: compile_to_ir: print string ──────────────────────────────────────────
#[test]
fn t3_ir_print_str() {
    let ir = compile_to_ir(b"print \"sovereign\"\n").unwrap();
    assert!(ir.iter().any(|n| matches!(n, AxIr::PrintStr { text }
        if text == "sovereign")));
}

// ── T4: compile_to_ir: print variable ────────────────────────────────────────
#[test]
fn t4_ir_print_var() {
    let ir = compile_to_ir(b"let x = 7\nprint x\n").unwrap();
    assert!(ir.iter().any(|n| matches!(n, AxIr::PrintVar { var } if var == "x")));
}

// ── T5: compile_to_ir: addition ───────────────────────────────────────────────
#[test]
fn t5_ir_addition() {
    let ir = compile_to_ir(b"let z = 3 + 4\n").unwrap();
    assert!(ir.iter().any(|n| matches!(n,
        AxIr::BinOp { dst, op: BinOpKind::Add, .. } if dst == "z")));
}

// ── T6: compile_to_ir: subtraction ───────────────────────────────────────────
#[test]
fn t6_ir_subtraction() {
    let ir = compile_to_ir(b"let z = 10 - 3\n").unwrap();
    assert!(ir.iter().any(|n| matches!(n,
        AxIr::BinOp { op: BinOpKind::Sub, .. })));
}

// ── T7: compile_to_ir: multiplication ────────────────────────────────────────
#[test]
fn t7_ir_multiplication() {
    let ir = compile_to_ir(b"let z = 6 * 7\n").unwrap();
    assert!(ir.iter().any(|n| matches!(n,
        AxIr::BinOp { op: BinOpKind::Mul, .. })));
}

// ── T8: compile_to_ir: program start/end ─────────────────────────────────────
#[test]
fn t8_ir_program_bounds() {
    let ir = compile_to_ir(b"print \"ok\"\n").unwrap();
    assert!(matches!(ir.first(), Some(AxIr::ProgramStart)));
    assert!(matches!(ir.last(), Some(AxIr::ProgramEnd)));
}

// ── T9: compile_to_ir: comments skipped ──────────────────────────────────────
#[test]
fn t9_ir_comments_skipped() {
    let ir = compile_to_ir(b"// comment\nlet x = 1\n").unwrap();
    // No IR node for the comment
    assert!(!ir.iter().any(|n| matches!(n, AxIr::LoadImm { value, .. } if *value == 0)));
    assert!(ir.iter().any(|n| matches!(n, AxIr::LoadImm { var, value }
        if var == "x" && *value == 1)));
}

// ── T10: compile_to_ir: awp statement ────────────────────────────────────────
#[test]
fn t10_ir_awp() {
    let ir = compile_to_ir(b"awp ping\n").unwrap();
    assert!(ir.iter().any(|n| matches!(n, AxIr::AwpSend { payload }
        if payload == "ping")));
}

// ── T11: emit_asm: freestanding has _start ───────────────────────────────────
#[test]
fn t11_emit_freestanding_start() {
    let ir = compile_to_ir(b"let x = 1\n").unwrap();
    let asm = emit_asm(&ir, &freestanding());
    assert!(asm.contains("_start"), "freestanding must have _start entry");
    assert!(asm.contains("axon_main"), "must call axon_main");
    assert!(asm.contains("wfi"), "freestanding halt must use WFI");
}

// ── T12: emit_asm: BSS zero loop ─────────────────────────────────────────────
#[test]
fn t12_emit_bss_zero() {
    let ir = compile_to_ir(b"let x = 1\n").unwrap();
    let asm = emit_asm(&ir, &freestanding());
    assert!(asm.contains("__bss_start"), "must zero BSS");
    assert!(asm.contains("__bss_end"));
}

// ── T13: emit_asm: Linux uses syscall ────────────────────────────────────────
#[test]
fn t13_emit_linux_syscall() {
    let ir = compile_to_ir(b"print \"test\"\n").unwrap();
    let asm = emit_asm(&ir, &linux());
    assert!(asm.contains("svc #0"), "Linux must use SVC syscall");
    assert!(!asm.contains("_start"), "Linux mode has no bare-metal _start");
}

// ── T14: emit_asm: freestanding uses UART ────────────────────────────────────
#[test]
fn t14_emit_freestanding_uart() {
    let ir = compile_to_ir(b"print \"sovereign\"\n").unwrap();
    let asm = emit_asm(&ir, &freestanding());
    // Freestanding print goes through UART, not Linux syscall write
    assert!(asm.contains("axon_uart_write") || asm.contains("0x09000000"),
        "freestanding must use UART");
}

// ── T15: emit_asm: AArch64 arch directive ────────────────────────────────────
#[test]
fn t15_emit_arch_directive() {
    let ir = compile_to_ir(b"let x = 1\n").unwrap();
    let asm = emit_asm(&ir, &freestanding());
    assert!(asm.contains(".arch armv8-a"), "must declare armv8-a arch");
}

// ── T16: emit_asm: axon_print_i64 helper present ─────────────────────────────
#[test]
fn t16_emit_print_i64_helper() {
    let ir = compile_to_ir(b"let x = 42\nprint x\n").unwrap();
    let asm = emit_asm(&ir, &linux());
    assert!(asm.contains("axon_print_i64"), "must emit print_i64 helper");
}

// ── T17: linker script: correct load address ─────────────────────────────────
#[test]
fn t17_linker_script_load_addr() {
    let ld = generate_linker_script(AIXOS_LOAD_ADDR, DEFAULT_STACK_SIZE);
    assert!(ld.contains("0x40080000"), "must use aiXos load address");
    assert!(ld.contains("_start"), "must declare ENTRY(_start)");
    assert!(ld.contains("__bss_start"), "must declare BSS symbols");
    assert!(ld.contains("__stack_top"), "must declare stack top");
}

// ── T18: conformance: interp and IR agree on print ───────────────────────────
#[test]
fn t18_conformance_print() {
    let script = b"print \"sovereign\"\n";
    let r = check_conformance(script);
    assert!(r.is_conformant(), "print script must be conformant");
    assert_eq!(&r.interp_output, b"sovereign\n");
}

// ── T19: conformance: interp and IR agree on arithmetic ──────────────────────
#[test]
fn t19_conformance_arithmetic() {
    let script = b"let x = 6 * 7\nprint x\n";
    let r = check_conformance(script);
    assert!(r.is_conformant());
    assert_eq!(&r.interp_output, b"42\n");
}

// ── T20: full P71 codegen pipeline ───────────────────────────────────────────
#[test]
fn t20_full_p71_pipeline() {
    let script = b"\
// sovereign .ax for aiXos Phoenix
let version = 1
let answer = 6 * 7
print \"axon_aarch64 P71\"
print answer
awp sovereign-ping
";

    // 1. Conformance check (interp == IR)
    let conf = check_conformance(script);
    assert!(conf.is_conformant(), "script must be conformant");
    assert!(conf.interp_output.windows(2).any(|w| w == b"42"),
        "interp must output 42");

    // 2. Compile to IR
    let ir = compile_to_ir(script).unwrap();
    assert!(ir.iter().any(|n| matches!(n, AxIr::BinOp { op: BinOpKind::Mul, .. })));
    assert!(ir.iter().any(|n| matches!(n, AxIr::AwpSend { .. })));

    // 3. Emit freestanding AArch64 assembly
    let asm_free = emit_asm(&ir, &freestanding());
    assert!(asm_free.contains("_start"));
    assert!(asm_free.contains("axon_main"));
    assert!(asm_free.contains("wfi"));
    assert!(asm_free.contains(".arch armv8-a"));

    // 4. Emit Linux AArch64 assembly (for conformance testing)
    let asm_linux = emit_asm(&ir, &linux());
    assert!(asm_linux.contains("svc #0"));
    assert!(asm_linux.contains("axon_print_i64"));

    // 5. Linker script for aiXos
    let ld = generate_linker_script(AIXOS_LOAD_ADDR, DEFAULT_STACK_SIZE);
    assert!(ld.contains("0x40080000"));
    assert!(ld.contains("__stack_top"));

    // 6. IR contains_check helper
    assert!(ir_contains(script, |n| matches!(n, AxIr::PrintStr { .. })));
    assert!(ir_contains(script, |n| matches!(n, AxIr::AwpSend { .. })));
}
