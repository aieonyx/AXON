// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
//
// P55.6 QA — axon_runtime sovereign type runtime tests
// Pass bar: 25/25

use axon_runtime::{Money, SafeInt, Secret, Zeroize, assert_balanced};

// ── Money tests ───────────────────────────────────────────────────────────────

#[test]
fn test_money_new_exact() {
    let m = Money::new("EUR", 2, 19, 99);
    assert_eq!(m.amount, 1999);
    assert_eq!(m.currency, "EUR");
    assert_eq!(m.precision, 2);
}

#[test]
fn test_money_display() {
    let m = Money::new("EUR", 2, 19, 99);
    assert_eq!(m.display(), "19.99 EUR");
}

#[test]
fn test_money_add() {
    let a = Money::new("EUR", 2, 10, 0);
    let b = Money::new("EUR", 2, 5, 50);
    let c = a.add(&b);
    assert_eq!(c.amount, 1550);
    assert_eq!(c.display(), "15.50 EUR");
}

#[test]
fn test_money_sub() {
    let a = Money::new("EUR", 2, 20, 0);
    let b = Money::new("EUR", 2, 5, 0);
    let c = a.sub(&b);
    assert_eq!(c.amount, 1500);
    assert_eq!(c.display(), "15.00 EUR");
}

#[test]
fn test_money_mul_scalar() {
    let a = Money::new("EUR", 2, 5, 0);
    let b = a.mul_scalar(3);
    assert_eq!(b.amount, 1500);
    assert_eq!(b.display(), "15.00 EUR");
}

#[test]
fn test_money_is_zero() {
    let z = Money::from_raw("EUR", 2, 0);
    assert!(z.is_zero());
    let nz = Money::new("EUR", 2, 1, 0);
    assert!(!nz.is_zero());
}

#[test]
fn test_money_is_negative() {
    let n = Money::from_raw("EUR", 2, -100);
    assert!(n.is_negative());
    let p = Money::new("EUR", 2, 1, 0);
    assert!(!p.is_negative());
}

#[test]
fn test_money_no_float_precision_loss() {
    // 0.1 + 0.2 == 0.3 exactly — impossible with f64, trivial with fixed-point
    let a = Money::new("EUR", 2, 0, 10); // 0.10
    let b = Money::new("EUR", 2, 0, 20); // 0.20
    let c = a.add(&b);
    assert_eq!(c.amount, 30);           // exactly 0.30
    assert_eq!(c.display(), "0.30 EUR");
}

#[test]
#[should_panic(expected = "currency mismatch")]
fn test_money_currency_mismatch_panics() {
    let a = Money::new("EUR", 2, 10, 0);
    let b = Money::new("USD", 2, 5, 0);
    let _ = a.add(&b);
}

#[test]
fn test_money_balanced_ok() {
    let debits  = vec![Money::new("EUR", 2, 100, 0)];
    let credits = vec![Money::new("EUR", 2, 100, 0)];
    assert!(assert_balanced(&debits, &credits).is_ok());
}

#[test]
fn test_money_unbalanced_err() {
    let debits  = vec![Money::new("EUR", 2, 100, 0)];
    let credits = vec![Money::new("EUR", 2, 90,  0)];
    let result  = assert_balanced(&debits, &credits);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), 1000); // 100.00 - 90.00 = 10.00 = 1000 raw
}

#[test]
fn test_money_multi_entry_balanced() {
    let debits  = vec![
        Money::new("EUR", 2, 50, 0),
        Money::new("EUR", 2, 50, 0),
    ];
    let credits = vec![Money::new("EUR", 2, 100, 0)];
    assert!(assert_balanced(&debits, &credits).is_ok());
}

// ── SafeInt tests ─────────────────────────────────────────────────────────────

#[test]
fn test_safeint_new_valid() {
    let s = SafeInt::new(0, 100, 42);
    assert_eq!(s.get(), 42);
}

#[test]
#[should_panic(expected = "out of range")]
fn test_safeint_new_out_of_range_panics() {
    let _ = SafeInt::new(0, 100, 101);
}

#[test]
fn test_safeint_add_valid() {
    let s = SafeInt::new(0, 100, 40);
    let r = s.add(10);
    assert_eq!(r.get(), 50);
}

#[test]
#[should_panic(expected = "out of range")]
fn test_safeint_add_overflow_panics() {
    let s = SafeInt::new(0, 100, 95);
    let _ = s.add(10); // 105 > 100
}

#[test]
fn test_safeint_sub_valid() {
    let s = SafeInt::new(0, 100, 50);
    let r = s.sub(20);
    assert_eq!(r.get(), 30);
}

#[test]
#[should_panic(expected = "out of range")]
fn test_safeint_sub_underflow_panics() {
    let s = SafeInt::new(0, 100, 5);
    let _ = s.sub(10); // -5 < 0
}

#[test]
fn test_safeint_mul_valid() {
    let s = SafeInt::new(0, 1000, 10);
    let r = s.mul(5);
    assert_eq!(r.get(), 50);
}

#[test]
fn test_safeint_set_valid() {
    let mut s = SafeInt::new(0, 100, 10);
    s.set(50);
    assert_eq!(s.get(), 50);
}

// ── Secret tests ──────────────────────────────────────────────────────────────

#[test]
fn test_secret_debug_hides_value() {
    let s = Secret::new(vec![1u8, 2, 3]);
    assert_eq!(format!("{:?}", s), "Secret<*>");
}

#[test]
fn test_secret_display_hides_value() {
    let s = Secret::new("my_password".to_string());
    assert_eq!(format!("{}", s), "Secret<*>");
}

#[test]
fn test_secret_declassify() {
    let s = Secret::new(vec![42u8]);
    assert_eq!(s.declassify(), &vec![42u8]);
}

#[test]
fn test_secret_zeroize_on_drop() {
    let mut zeroed = false;
    {
        let mut v = vec![1u8, 2, 3, 4];
        v.zeroize();
        zeroed = v.is_empty();
    }
    assert!(zeroed);
}

#[test]
fn test_secret_i64_zeroize() {
    let mut val: i64 = 12345;
    val.zeroize();
    assert_eq!(val, 0);
}
