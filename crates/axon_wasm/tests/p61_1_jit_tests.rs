// Copyright (c) 2026 Edison Lepiten / AIEONYX
// P61.1 — ECHO WASM JIT tests (20 tests)

use axon_wasm::jit::{
    jit_compile, CodeBuffer, leb128_u32, leb128_i32, leb128_i64,
};
use axon_wasm::module::WasmFunc;
use axon_wasm::types::ValType;

// Helper: build a minimal WasmFunc from raw opcode bytes
fn make_func(body: Vec<u8>) -> WasmFunc {
    WasmFunc { type_idx: 0, locals: vec![], body }
}

// ── T1: CodeBuffer allocates non-null ────────────────────────────────────────
#[test]
fn t1_code_buffer_alloc() {
    let buf = CodeBuffer::new(4096).expect("mmap should succeed");
    assert!(buf.written() == 0);
}

// ── T2: CodeBuffer emit_u8 ───────────────────────────────────────────────────
#[test]
fn t2_emit_u8() {
    let mut buf = CodeBuffer::new(4096).unwrap();
    buf.emit_u8(0x90).unwrap(); // nop
    assert_eq!(buf.written(), 1);
}

// ── T3: LEB128 u32 single byte ───────────────────────────────────────────────
#[test]
fn t3_leb128_u32_single() {
    let (v, n) = leb128_u32(&[42]).unwrap();
    assert_eq!(v, 42);
    assert_eq!(n, 1);
}

// ── T4: LEB128 u32 multi-byte ────────────────────────────────────────────────
#[test]
fn t4_leb128_u32_multi() {
    // 300 = 0xAC 0x02 in LEB128
    let (v, n) = leb128_u32(&[0xAC, 0x02]).unwrap();
    assert_eq!(v, 300);
    assert_eq!(n, 2);
}

// ── T5: LEB128 i32 positive ───────────────────────────────────────────────────
#[test]
fn t5_leb128_i32_positive() {
    let (v, n) = leb128_i32(&[0x10]).unwrap();
    assert_eq!(v, 16);
    assert_eq!(n, 1);
}

// ── T6: LEB128 i32 negative ───────────────────────────────────────────────────
#[test]
fn t6_leb128_i32_negative() {
    // -1 in LEB128 signed = 0x7F
    let (v, n) = leb128_i32(&[0x7F]).unwrap();
    assert_eq!(v, -1);
    assert_eq!(n, 1);
}

// ── T7: LEB128 i64 large positive ────────────────────────────────────────────
#[test]
fn t7_leb128_i64_large() {
    // 624485 = 0xE5 0x8E 0x26
    let (v, n) = leb128_i64(&[0xE5, 0x8E, 0x26]).unwrap();
    assert_eq!(v, 624485);
    assert_eq!(n, 3);
}

// ── T8: JIT compile empty body (just end) returns 0 ─────────────────────────
#[test]
fn t8_jit_empty_body() {
    let func = make_func(vec![0x0B]); // end
    let jf = jit_compile(&func, 0).unwrap();
    let mut locals: Vec<i64> = vec![];
    let result = jf.call(&mut locals);
    assert_eq!(result, 0);
}

// ── T9: JIT i32.const 42 ─────────────────────────────────────────────────────
#[test]
fn t9_jit_i32_const_42() {
    // i32.const 42, end
    let func = make_func(vec![0x41, 42, 0x0B]);
    let jf = jit_compile(&func, 0).unwrap();
    let mut locals: Vec<i64> = vec![];
    let result = jf.call(&mut locals);
    assert_eq!(result, 42);
}

// ── T10: JIT i32.const negative ──────────────────────────────────────────────
#[test]
fn t10_jit_i32_const_neg() {
    // i32.const -1 (LEB128: 0x7F), end
    let func = make_func(vec![0x41, 0x7F, 0x0B]);
    let jf = jit_compile(&func, 0).unwrap();
    let mut locals: Vec<i64> = vec![];
    let result = jf.call(&mut locals);
    assert_eq!(result, -1i64);
}

// ── T11: JIT i32.add ─────────────────────────────────────────────────────────
#[test]
fn t11_jit_i32_add() {
    // i32.const 10, i32.const 32, i32.add, end  → 42
    let func = make_func(vec![0x41, 10, 0x41, 32, 0x6A, 0x0B]);
    let jf = jit_compile(&func, 0).unwrap();
    let mut locals: Vec<i64> = vec![];
    let result = jf.call(&mut locals);
    assert_eq!(result, 42);
}

// ── T12: JIT i32.sub ─────────────────────────────────────────────────────────
#[test]
fn t12_jit_i32_sub() {
    // i32.const 100, i32.const 58, i32.sub, end  → 42
    let func = make_func(vec![0x41, 0xE4, 0x00, 0x41, 58, 0x6B, 0x0B]); // 100=2-byte LEB128
    let jf = jit_compile(&func, 0).unwrap();
    let mut locals: Vec<i64> = vec![];
    let result = jf.call(&mut locals);
    assert_eq!(result, 42);
}

// ── T13: JIT i32.mul ─────────────────────────────────────────────────────────
#[test]
fn t13_jit_i32_mul() {
    // i32.const 6, i32.const 7, i32.mul, end  → 42
    let func = make_func(vec![0x41, 6, 0x41, 7, 0x6C, 0x0B]);
    let jf = jit_compile(&func, 0).unwrap();
    let mut locals: Vec<i64> = vec![];
    let result = jf.call(&mut locals);
    assert_eq!(result, 42);
}

// ── T14: JIT local.get / local.set ───────────────────────────────────────────
#[test]
fn t14_jit_local_get_set() {
    // locals[0] = 99 (pre-set), local.get 0, end  → 99
    let func = make_func(vec![0x20, 0x00, 0x0B]); // local.get 0, end
    let jf = jit_compile(&func, 1).unwrap();
    let mut locals = vec![99i64];
    let result = jf.call(&mut locals);
    assert_eq!(result, 99);
}

// ── T15: JIT local.set then local.get ────────────────────────────────────────
#[test]
fn t15_jit_local_set_get() {
    // i32.const 77, local.set 0, local.get 0, end  → 77
    let func = make_func(vec![0x41, 0xCD, 0x00, 0x21, 0x00, 0x20, 0x00, 0x0B]); // 77=2-byte LEB128
    let jf = jit_compile(&func, 1).unwrap();
    let mut locals = vec![0i64];
    let result = jf.call(&mut locals);
    assert_eq!(result, 77);
}

// ── T16: JIT local.tee ───────────────────────────────────────────────────────
#[test]
fn t16_jit_local_tee() {
    // i32.const 55, local.tee 0, end  → 55, and locals[0]==55
    let func = make_func(vec![0x41, 55, 0x22, 0x00, 0x0B]);
    let jf = jit_compile(&func, 1).unwrap();
    let mut locals = vec![0i64];
    let result = jf.call(&mut locals);
    assert_eq!(result, 55);
    assert_eq!(locals[0], 55);
}

// ── T17: JIT nop passthrough ─────────────────────────────────────────────────
#[test]
fn t17_jit_nop() {
    // nop, i32.const 42, nop, end
    let func = make_func(vec![0x01, 0x41, 42, 0x01, 0x0B]);
    let jf = jit_compile(&func, 0).unwrap();
    let mut locals: Vec<i64> = vec![];
    let result = jf.call(&mut locals);
    assert_eq!(result, 42);
}

// ── T18: JIT drop ────────────────────────────────────────────────────────────
#[test]
fn t18_jit_drop() {
    // i32.const 99, i32.const 42, drop, end  → 99 (99 stays on stack)
    // Wait: drop pops the top (42), leaving 99.
    let func = make_func(vec![0x41, 0xE3, 0x00, 0x41, 42, 0x1A, 0x0B]); // 99=2-byte LEB128
    let jf = jit_compile(&func, 0).unwrap();
    let mut locals: Vec<i64> = vec![];
    let result = jf.call(&mut locals);
    assert_eq!(result, 99);
}

// ── T19: JIT compound expression (a+b)*c ─────────────────────────────────────
#[test]
fn t19_jit_compound() {
    // (3 + 4) * 6 = 42
    // i32.const 3, i32.const 4, i32.add, i32.const 6, i32.mul, end
    let func = make_func(vec![0x41, 3, 0x41, 4, 0x6A, 0x41, 6, 0x6C, 0x0B]);
    let jf = jit_compile(&func, 0).unwrap();
    let mut locals: Vec<i64> = vec![];
    let result = jf.call(&mut locals);
    assert_eq!(result, 42);
}

// ── T20: JIT unsupported opcode returns error ─────────────────────────────────
#[test]
fn t20_jit_unsupported_opcode() {
    // 0xFF is not a valid WASM opcode in our JIT
    let func = make_func(vec![0xFF, 0x0B]);
    let result = jit_compile(&func, 0);
    assert!(result.is_err(), "unsupported opcode must return Err");
}
