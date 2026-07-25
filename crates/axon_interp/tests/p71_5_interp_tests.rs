// Copyright (c) 2026 Edison Lepiten / AIEONYX
// axon_interp P71.5 — Sovereign .ax interpreter tests (20 tests)
// Upstreamed from aiXos PL-59.1 + expanded for AXON workspace

use axon_interp::{exec, exec_with_state, AxInterp, eval_expr, AxVar, MAX_VARS};

fn run(script: &[u8]) -> axon_interp::AxResult {
    exec(script, 0, None)
}

fn out(script: &[u8]) -> Vec<u8> {
    let r = run(script);
    r.output[..r.output_len].to_vec()
}

// ── T1: print string literal (from PL-59.1) ──────────────────────────────────
#[test]
fn t1_print_string() {
    assert_eq!(out(b"print \"hello sovereign\"\n"), b"hello sovereign\n");
}

// ── T2: let + print variable (from PL-59.1) ──────────────────────────────────
#[test]
fn t2_let_print_var() {
    assert_eq!(out(b"let x = 42\nprint x\n"), b"42\n");
}

// ── T3: comments ignored (from PL-59.1) ──────────────────────────────────────
#[test]
fn t3_comments_ignored() {
    assert_eq!(out(b"// comment\nprint \"ok\"\n"), b"ok\n");
}

// ── T4: let from variable (from PL-59.1) ─────────────────────────────────────
#[test]
fn t4_let_from_var() {
    assert_eq!(out(b"let a = 10\nlet b = a\nprint b\n"), b"10\n");
}

// ── T5: multi-line script (from PL-59.1) ─────────────────────────────────────
#[test]
fn t5_multi_line() {
    let r = run(b"let n = 7\nprint \"aiXos\"\nprint n\n");
    assert!(!r.error);
    assert_eq!(r.lines_executed, 3);
}

// ── T6: awp no transport (from PL-59.1) ──────────────────────────────────────
#[test]
fn t6_awp_no_transport() {
    let r = run(b"awp hello\n");
    assert!(!r.error);
    assert!(r.output_len > 0);
}

// ── T7: integer addition ──────────────────────────────────────────────────────
#[test]
fn t7_addition() {
    assert_eq!(out(b"let x = 3 + 4\nprint x\n"), b"7\n");
}

// ── T8: integer subtraction ───────────────────────────────────────────────────
#[test]
fn t8_subtraction() {
    assert_eq!(out(b"let x = 10 - 3\nprint x\n"), b"7\n");
}

// ── T9: integer multiplication ────────────────────────────────────────────────
#[test]
fn t9_multiplication() {
    assert_eq!(out(b"let x = 6 * 7\nprint x\n"), b"42\n");
}

// ── T10: chained assignments ──────────────────────────────────────────────────
#[test]
fn t10_chained() {
    assert_eq!(out(b"let a = 5\nlet b = a + 3\nlet c = b + 2\nprint c\n"), b"10\n");
}

// ── T11: negative integer literal ────────────────────────────────────────────
#[test]
fn t11_negative() {
    assert_eq!(out(b"let x = -5\nprint x\n"), b"-5\n");
}

// ── T12: zero value ───────────────────────────────────────────────────────────
#[test]
fn t12_zero() {
    assert_eq!(out(b"let x = 0\nprint x\n"), b"0\n");
}

// ── T13: multiple variables ───────────────────────────────────────────────────
#[test]
fn t13_multiple_vars() {
    let script = b"let a = 1\nlet b = 2\nlet c = 3\nprint a\nprint b\nprint c\n";
    assert_eq!(out(script), b"1\n2\n3\n");
}

// ── T14: variable overwrite ───────────────────────────────────────────────────
#[test]
fn t14_var_overwrite() {
    assert_eq!(out(b"let x = 1\nlet x = 99\nprint x\n"), b"99\n");
}

// ── T15: awp with transport callback ─────────────────────────────────────────
#[test]
fn t15_awp_transport() {
    fn mock_send(_node: u64, _payload: &[u8]) -> bool { true }
    let r = exec(b"awp sovereign-ping\n", 42, Some(mock_send));
    assert!(!r.error);
    assert!(r.as_str().starts_with(b"awp: sent"));
}

// ── T16: error on undefined variable in let ───────────────────────────────────
#[test]
fn t16_undefined_var_error() {
    let r = run(b"let x = undefined_var\n");
    assert!(r.error, "undefined var should cause error");
}

// ── T17: eval_expr direct ────────────────────────────────────────────────────
#[test]
fn t17_eval_expr() {
    let vars = [AxVar::empty(); MAX_VARS];
    assert_eq!(eval_expr(b"42", &vars), Some(42));
    assert_eq!(eval_expr(b"3 + 4", &vars), Some(7));
    assert_eq!(eval_expr(b"10 - 3", &vars), Some(7));
    assert_eq!(eval_expr(b"6 * 7", &vars), Some(42));
}

// ── T18: persistent state across exec calls (REPL mode) ──────────────────────
#[test]
fn t18_persistent_state() {
    let mut interp = AxInterp::new();
    // First call sets variable
    exec_with_state(b"let x = 100\n", &mut interp, 0, None);
    // Second call reads it
    let r = exec_with_state(b"print x\n", &mut interp, 0, None);
    assert!(!r.error);
    assert_eq!(r.as_str(), b"100\n");
}

// ── T19: REPL state reset ────────────────────────────────────────────────────
#[test]
fn t19_repl_reset() {
    let mut interp = AxInterp::new();
    exec_with_state(b"let x = 42\n", &mut interp, 0, None);
    interp.reset();
    // After reset, x should be undefined
    let r = exec_with_state(b"let y = x\n", &mut interp, 0, None);
    assert!(r.error, "x should be undefined after reset");
}

// ── T20: full sovereign .ax script ───────────────────────────────────────────
#[test]
fn t20_full_sovereign_script() {
    let script = b"\
// AIEONYX sovereign script v0.1
// Tests all supported statement types
let version = 1
let major = 0
let minor = 68
let patch = major + minor
print \"AXONYX sovereign interpreter\"
print version
let answer = 6 * 7
print answer
awp ping
";
    let r = exec(script, 1, None);
    assert!(!r.error, "script error at line {}: {:?}", r.error_line,
        core::str::from_utf8(r.error_str()).unwrap_or("?"));
    let output = r.as_str();
    assert!(output.windows(6).any(|w| w == b"AXONYX"), "must print AXONYX");
    assert!(output.windows(2).any(|w| w == b"42"), "must print 42");
    assert!(r.lines_executed >= 7);
}
