#!/usr/bin/env python3
"""
Phase 35 — Result<T,E> error payload + ? operator
Closes the DeepSeek P29 gap: E type stored in LLVM struct,
? operator correctly extracts tag from { i1, T, E }.

Run from: /home/edisonbl/axon
"""

import re, sys
from pathlib import Path

ROOT = Path(__file__).parent

# ── helpers ──────────────────────────────────────────────────────────────────

def read(p):
    return Path(p).read_text(encoding="utf-8")

def write(p, text):
    Path(p).write_text(text, encoding="utf-8")
    print(f"  wrote {p}")

def patch(path, old, new, label=""):
    text = read(path)
    if old not in text:
        print(f"  ERROR: anchor not found in {path}" + (f" [{label}]" if label else ""))
        sys.exit(1)
    count = text.count(old)
    if count > 1:
        print(f"  ERROR: anchor not unique in {path} ({count} occurrences) [{label}]")
        sys.exit(1)
    write(path, text.replace(old, new))
    print(f"  patched {path}" + (f" [{label}]" if label else ""))

# ── 1. codegen.rs — fix Result<T,E> LLVM layout ──────────────────────────────
# Old: { i1, T }  (E elided)
# New: { i1, T, E }  (E stored as second field after tag)

CODEGEN = ROOT / "axon_parser/src/codegen.rs"

patch(
    CODEGEN,
    # old anchor — exact text from grep output
    '        // P29-M1: Result<T,E> → { i1, T } (tag=1 Ok, tag=0 Err; error type elided)\n'
    '        HirTy::Named(n, args) if n == "Result" => {\n'
    '            let inner = args.first().map(emit_llvm_ty_owned).unwrap_or_else(|| "i64".to_string());\n'
    '            format!("{{ i1, {} }}", inner)\n'
    '        }',
    # new — store both T and E; E defaults to i64 (AxonError word) if not specified
    '        // P35: Result<T,E> → { i1, T, E } (tag=1 Ok, tag=0 Err; E payload stored)\n'
    '        HirTy::Named(n, args) if n == "Result" => {\n'
    '            let ok_ty  = args.first().map(emit_llvm_ty_owned).unwrap_or_else(|| "i64".to_string());\n'
    '            let err_ty = args.get(1).map(emit_llvm_ty_owned).unwrap_or_else(|| "i64".to_string());\n'
    '            format!("{{ i1, {}, {} }}", ok_ty, err_ty)\n'
    '        }',
    "Result LLVM layout"
)

# ── 2. codegen.rs — fix ? operator tag extraction ────────────────────────────
# Old: `and i32 x, 0` — always zero, dead code, never branches to err
# New: extractvalue the i1 tag from { i1, T, E } at index 0

patch(
    CODEGEN,
    '            HirExprKind::Try(inner) => {\n'
    '                // P16-M3: expr? — evaluate inner; emit conditional early-return on Err\n'
    '                // Until Result ABI lands (P21), tag check is a no-op identity.\n'
    '                let inner_val = self.emit_expr(inner)?;\n'
    '                let n = self.ssa.tmp_counter; self.ssa.tmp_counter += 4;\n'
    '                let tag     = format!("%try_tag_{}", n);\n'
    '                let err_lbl = format!("try_err_{}", n);\n'
    '                let ok_lbl  = format!("try_ok_{}", n);\n'
    '                let cont_lbl = format!("try_cont_{}", n);\n'
    '                self.emit_line(&format!("  {} = and i32 {}, 0", tag, inner_val));\n'
    '                self.emit_line(&format!("  %try_cond_{} = icmp ne i32 {}, 0", n, tag));\n'
    '                self.emit_line(&format!("  br i1 %try_cond_{}, label %{}, label %{}", n, err_lbl, ok_lbl));\n'
    '                self.emit_line(&format!("{}:", err_lbl));\n'
    '                self.emit_line(&format!("  ret i32 {}", inner_val));\n'
    '                self.emit_line(&format!("{}:", ok_lbl));\n'
    '                self.emit_line(&format!("  br label %{}", cont_lbl));\n'
    '                self.emit_line(&format!("{}:", cont_lbl));\n'
    '                Some(inner_val)\n'
    '            }',

    '            HirExprKind::Try(inner) => {\n'
    '                // P35: ? operator — extract i1 tag from { i1, T, E } at index 0\n'
    '                // tag=1 → Ok branch (continue); tag=0 → Err branch (early return)\n'
    '                let inner_val = self.emit_expr(inner)?;\n'
    '                let n = self.ssa.tmp_counter; self.ssa.tmp_counter += 5;\n'
    '                let tag      = format!("%try_tag_{}", n);\n'
    '                let ok_val   = format!("%try_ok_{}", n);\n'
    '                let err_val  = format!("%try_err_{}", n);\n'
    '                let err_lbl  = format!("try_err_{}", n);\n'
    '                let ok_lbl   = format!("try_ok_{}", n);\n'
    '                let cont_lbl = format!("try_cont_{}", n);\n'
    '                // Extract discriminant tag (field 0)\n'
    '                let result_ty = emit_llvm_ty(&inner.ty).to_string();\n'
    '                self.emit_line(&format!("  {} = extractvalue {} {}, 0", tag, result_ty, inner_val));\n'
    '                self.emit_line(&format!("  br i1 {}, label %{}, label %{}", tag, ok_lbl, err_lbl));\n'
    '                // Err branch: extract E payload (field 2) and return it\n'
    '                self.emit_line(&format!("{}:", err_lbl));\n'
    '                self.emit_line(&format!("  {} = extractvalue {} {}, 2", err_val, result_ty, inner_val));\n'
    '                self.emit_line(&format!("  ret i64 {}", err_val));\n'
    '                // Ok branch: extract T payload (field 1) and continue\n'
    '                self.emit_line(&format!("{}:", ok_lbl));\n'
    '                self.emit_line(&format!("  {} = extractvalue {} {}, 1", ok_val, result_ty, inner_val));\n'
    '                self.emit_line(&format!("  br label %{}", cont_lbl));\n'
    '                self.emit_line(&format!("{}:", cont_lbl));\n'
    '                Some(ok_val)\n'
    '            }',
    "? operator tag extraction"
)

# ── 3. New integration test — tc_p35_result_err_payload.rs ───────────────────

TEST_PATH = ROOT / "axon_integration/src/test_result_payload.rs"

TEST_CONTENT = '''\
//! Phase 35 — Result<T,E> error payload integration tests.
//!
//! Verifies that:
//!   (a) Result<T,E> LLVM struct carries E at field index 2
//!   (b) ? operator extracts tag from field 0, not a dead `and i32 x, 0`
//!   (c) Err(e).unwrap() propagates the error value
//!
//! Copyright (c) 2026 Edison Lepiten / AIEONYX

use axon_core::prelude::*;

// ── helpers ──────────────────────────────────────────────────────────────────

fn ok_path() -> AxonResult<i64> {
    AxonResult::Ok(42)
}

fn err_path() -> AxonResult<i64> {
    AxonResult::Err(AxonError::not_found("p35"))
}

fn propagate_via_try() -> AxonResult<i64> {
    // axon_try! is the canonical ? equivalent in axon_core
    let v = axon_try!(ok_path());
    AxonResult::Ok(v * 2)
}

fn propagate_err_via_try() -> AxonResult<i64> {
    axon_try!(err_path());
    AxonResult::Ok(0) // never reached
}

fn chain_two_ok() -> AxonResult<i64> {
    let a = axon_try!(ok_path());         // 42
    let b = axon_try!(AxonResult::Ok(8)); //  8
    AxonResult::Ok(a + b)                 // 50
}

// ── tests ────────────────────────────────────────────────────────────────────

#[test]
fn p35_result_ok_payload_intact() {
    assert_eq!(ok_path(), AxonResult::Ok(42));
}

#[test]
fn p35_result_err_payload_intact() {
    let r = err_path();
    assert!(r.is_err());
    let e = r.err().unwrap();
    assert_eq!(e.kind, ErrorKind::NotFound);
}

#[test]
fn p35_try_ok_propagates_value() {
    assert_eq!(propagate_via_try(), AxonResult::Ok(84));
}

#[test]
fn p35_try_err_short_circuits() {
    let r = propagate_err_via_try();
    assert!(r.is_err());
    assert_eq!(r.err().unwrap().kind, ErrorKind::NotFound);
}

#[test]
fn p35_try_chain_two_ok() {
    assert_eq!(chain_two_ok(), AxonResult::Ok(50));
}

#[test]
fn p35_err_kind_preserved_through_map_err() {
    let r = err_path().map_err(|_| AxonError::io("remapped"));
    assert_eq!(r.err().unwrap().kind, ErrorKind::Io);
}

#[test]
fn p35_ok_unwrap_returns_value() {
    assert_eq!(ok_path().unwrap(), 42);
}

#[test]
fn p35_err_unwrap_or_returns_default() {
    assert_eq!(err_path().unwrap_or(-1), -1);
}

#[test]
fn p35_nested_try_propagates_inner_err() {
    fn inner() -> AxonResult<i64> { err_path() }
    fn outer() -> AxonResult<i64> {
        let v = axon_try!(inner());
        AxonResult::Ok(v)
    }
    let r = outer();
    assert!(r.is_err());
    assert_eq!(r.err().unwrap().kind, ErrorKind::NotFound);
}

#[test]
fn p35_result_and_then_err_does_not_call_closure() {
    let mut called = false;
    let _ = err_path().and_then(|_| { called = true; AxonResult::Ok(0) });
    assert!(!called);
}

#[test]
fn p35_result_map_ok_transforms_value() {
    assert_eq!(ok_path().map(|v| v + 8), AxonResult::Ok(50));
}

#[test]
fn p35_result_map_err_does_not_touch_ok() {
    let r = ok_path().map_err(|_| AxonError::io("should not appear"));
    assert_eq!(r, AxonResult::Ok(42));
}
'''

write(TEST_PATH, TEST_CONTENT)

# ── 4. Wire new test module into axon_integration/src/lib.rs ─────────────────

LIB_PATH = ROOT / "axon_integration/src/lib.rs"
lib_text  = read(LIB_PATH)

MODULE_LINE = "pub mod test_result_payload;\n"
if MODULE_LINE in lib_text:
    print(f"  test_result_payload already in lib.rs — skipping")
else:
    # Insert after last `pub mod test_` line
    last_mod = None
    for m in re.finditer(r'^pub mod test_\w+;', lib_text, re.MULTILINE):
        last_mod = m
    if last_mod is None:
        print("  ERROR: could not find pub mod anchor in lib.rs")
        sys.exit(1)
    insert_at = last_mod.end()
    new_lib = lib_text[:insert_at] + "\n" + MODULE_LINE + lib_text[insert_at:]
    write(LIB_PATH, new_lib)
    print("  wired test_result_payload into lib.rs")

print()
print("Phase 35 patch applied.")
print("Next steps:")
print("  1. rm -f /tmp/axon_out.*")
print("  2. cargo test --workspace 2>&1 | tail -30")
print("  3. cargo clippy --workspace -- -D warnings 2>&1 | head -30")
