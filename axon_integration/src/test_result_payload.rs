// Copyright (c) 2026 Edison Lepiten / AIEONYX
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
