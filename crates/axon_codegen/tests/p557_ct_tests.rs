// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
// P55.7 QA -- @constant_time codegen enforcement tests.
// Pass bar: 8/8 before P55.7 closes.

use axon_codegen::codegen_source;

// T1: @constant_time fn emits #0 attribute reference on define line
#[test]
fn test_ct_define_has_attr_group() {
    let ir = codegen_source("@constant_time fn secure(x: i32) -> i32 { return x; }").unwrap();
    assert!(ir.contains("define i32 @secure(i32 %x) #0 {"),
        "expected '#0' on define line in:\n{}", ir);
}

// T2: @constant_time module emits attributes #0 block
#[test]
fn test_ct_emits_attribute_block() {
    let ir = codegen_source("@constant_time fn f() -> i32 { return 1; }").unwrap();
    assert!(ir.contains("attributes #0 = { noinline optnone"),
        "expected attributes #0 block in:\n{}", ir);
}

// T3: @constant_time emits noinline
#[test]
fn test_ct_noinline() {
    let ir = codegen_source("@constant_time fn f() -> i32 { return 1; }").unwrap();
    assert!(ir.contains("noinline"), "expected 'noinline' in:\n{}", ir);
}

// T4: @constant_time emits optnone
#[test]
fn test_ct_optnone() {
    let ir = codegen_source("@constant_time fn f() -> i32 { return 1; }").unwrap();
    assert!(ir.contains("optnone"), "expected 'optnone' in:\n{}", ir);
}

// T5: @constant_time emits no-speculation
#[test]
fn test_ct_no_speculation() {
    let ir = codegen_source("@constant_time fn f() -> i32 { return 1; }").unwrap();
    assert!(ir.contains("no-speculation"), "expected 'no-speculation' in:\n{}", ir);
}

// T6: @constant_time emits axon.constant_time metadata
#[test]
fn test_ct_metadata() {
    let ir = codegen_source("@constant_time fn f() -> i32 { return 1; }").unwrap();
    assert!(ir.contains("axon.constant_time"),
        "expected 'axon.constant_time' metadata in:\n{}", ir);
}

// T7: non-@constant_time fn does NOT get #0 attribute
#[test]
fn test_plain_fn_no_attr_group() {
    let ir = codegen_source("fn plain(x: i32) -> i32 { return x; }").unwrap();
    assert!(!ir.contains("#0"), "plain fn should not have #0 in:\n{}", ir);
    assert!(!ir.contains("attributes"), "plain fn should not emit attributes block in:\n{}", ir);
}

// T8: mixed module — CT fn gets #0, plain fn does not
#[test]
fn test_mixed_module_ct_and_plain() {
    let src = r#"
        @constant_time fn secure(x: i32) -> i32 { return x; }
        fn plain(y: i32) -> i32 { return y; }
    "#;
    let ir = codegen_source(src).unwrap();
    assert!(ir.contains("define i32 @secure(i32 %x) #0 {"),
        "secure fn should have #0 in:\n{}", ir);
    assert!(ir.contains("define i32 @plain(i32 %y) {"),
        "plain fn should not have #0 in:\n{}", ir);
    assert!(ir.contains("attributes #0"),
        "module should have attributes block in:\n{}", ir);
    // Audit comment lists secure
    assert!(ir.contains("@secure"), "audit comment should list @secure in:\n{}", ir);
}
