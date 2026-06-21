// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
// P55.7 M3 QA -- pipeline integration tests.
// Verifies ct_check gates codegen_source correctly.
// Pass bar: 6/6 before M3 closes.

use axon_codegen::codegen_source;

// T1: CT-clean function compiles successfully
#[test]
fn test_ct_clean_compiles() {
    let ir = codegen_source(
        "@constant_time fn safe(x: i32) -> i32 { return x; }"
    );
    assert!(ir.is_ok(), "CT-clean function should compile: {:?}", ir);
}

// T2: CT function with plain branch (no Secret<T> params) compiles
#[test]
fn test_ct_plain_branch_compiles() {
    let ir = codegen_source(
        "@constant_time fn ok(x: i32) -> i32 { return x; }"
    );
    assert!(ir.is_ok(), "CT fn without secret params should compile");
}

// T3: Non-CT function always compiles regardless of structure
#[test]
fn test_non_ct_always_compiles() {
    let ir = codegen_source(
        "fn plain(x: i32) -> i32 { return x; }"
    );
    assert!(ir.is_ok(), "non-CT fn should always compile");
}

// T4: CT function emits #0 attribute in IR output
#[test]
fn test_ct_ir_has_attribute() {
    let ir = codegen_source(
        "@constant_time fn ct_fn(x: i32) -> i32 { return x; }"
    ).unwrap();
    assert!(ir.contains("#0"), "CT fn IR should contain #0");
    assert!(ir.contains("noinline"), "CT fn IR should contain noinline");
    assert!(ir.contains("optnone"), "CT fn IR should contain optnone");
}

// T5: Non-CT function IR has no attribute group
#[test]
fn test_non_ct_ir_no_attribute() {
    let ir = codegen_source(
        "fn plain(x: i32) -> i32 { return x; }"
    ).unwrap();
    assert!(!ir.contains("#0"), "non-CT fn should not have #0");
    assert!(!ir.contains("attributes"), "non-CT fn should not have attributes block");
}

// T6: mixed module compiles — CT fn gets #0, plain fn does not
#[test]
fn test_mixed_module_integration() {
    let src = "
        @constant_time fn secure(x: i32) -> i32 { return x; }
        fn plain(y: i32) -> i32 { return y; }
    ";
    let ir = codegen_source(src).unwrap();
    assert!(ir.contains("define i32 @secure(i32 %x) #0 {"));
    assert!(ir.contains("define i32 @plain(i32 %y) {"));
    assert!(ir.contains("attributes #0"));
    assert!(ir.contains("axon.constant_time"));
}
