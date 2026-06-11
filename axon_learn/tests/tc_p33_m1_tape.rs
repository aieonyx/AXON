// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
//
// P33-M1 — Tape Autodiff Tests

use axon_learn::tape::Tape;

// ------------------------------------------------------------------
// Basic forward values
// ------------------------------------------------------------------

#[test]
fn tc_p33_m1_tape_leaf_value() {
    let mut tape = Tape::new();
    let x = tape.leaf(3.0);
    assert_eq!(x.value, 3.0);
}

#[test]
fn tc_p33_m1_tape_add_forward() {
    let mut tape = Tape::new();
    let a = tape.leaf(2.0);
    let b = tape.leaf(3.0);
    let c = a.add(b, &mut tape);
    assert_eq!(c.value, 5.0);
}

#[test]
fn tc_p33_m1_tape_mul_forward() {
    let mut tape = Tape::new();
    let a = tape.leaf(3.0);
    let b = tape.leaf(4.0);
    let c = a.mul(b, &mut tape);
    assert_eq!(c.value, 12.0);
}

#[test]
fn tc_p33_m1_tape_square_forward() {
    let mut tape = Tape::new();
    let x = tape.leaf(5.0);
    let y = x.square(&mut tape);
    assert_eq!(y.value, 25.0);
}

#[test]
fn tc_p33_m1_tape_relu_positive() {
    let mut tape = Tape::new();
    let x = tape.leaf(2.0);
    let y = x.relu(&mut tape);
    assert_eq!(y.value, 2.0);
}

#[test]
fn tc_p33_m1_tape_relu_negative() {
    let mut tape = Tape::new();
    let x = tape.leaf(-3.0);
    let y = x.relu(&mut tape);
    assert_eq!(y.value, 0.0);
}

#[test]
fn tc_p33_m1_tape_sigmoid_zero() {
    let mut tape = Tape::new();
    let x = tape.leaf(0.0);
    let y = x.sigmoid(&mut tape);
    assert!((y.value - 0.5).abs() < 1e-6);
}

// ------------------------------------------------------------------
// Backward pass — gradient correctness
// ------------------------------------------------------------------

#[test]
fn tc_p33_m1_grad_add() {
    // z = a + b; dz/da = 1, dz/db = 1
    let mut tape = Tape::new();
    let a = tape.leaf(2.0);
    let b = tape.leaf(3.0);
    let z = a.add(b, &mut tape);
    let grads = tape.backward(z.id);
    assert!((grads[a.id.0] - 1.0).abs() < 1e-6);
    assert!((grads[b.id.0] - 1.0).abs() < 1e-6);
}

#[test]
fn tc_p33_m1_grad_mul() {
    // z = a * b; dz/da = b = 3, dz/db = a = 2
    let mut tape = Tape::new();
    let a = tape.leaf(2.0);
    let b = tape.leaf(3.0);
    let z = a.mul(b, &mut tape);
    let grads = tape.backward(z.id);
    assert!((grads[a.id.0] - 3.0).abs() < 1e-6, "dz/da = {}", grads[a.id.0]);
    assert!((grads[b.id.0] - 2.0).abs() < 1e-6, "dz/db = {}", grads[b.id.0]);
}

#[test]
fn tc_p33_m1_grad_square() {
    // z = x²; dz/dx = 2x = 6
    let mut tape = Tape::new();
    let x = tape.leaf(3.0);
    let z = x.square(&mut tape);
    let grads = tape.backward(z.id);
    assert!((grads[x.id.0] - 6.0).abs() < 1e-6);
}

#[test]
fn tc_p33_m1_grad_chain_rule() {
    // z = (a + b)²; dz/da = 2(a+b) = 10, dz/db = 2(a+b) = 10
    let mut tape = Tape::new();
    let a = tape.leaf(2.0);
    let b = tape.leaf(3.0);
    let s = a.add(b, &mut tape);     // s = 5
    let z = s.square(&mut tape);     // z = 25
    let grads = tape.backward(z.id);
    assert!((grads[a.id.0] - 10.0).abs() < 1e-5);
    assert!((grads[b.id.0] - 10.0).abs() < 1e-5);
}

#[test]
fn tc_p33_m1_grad_relu_positive() {
    let mut tape = Tape::new();
    let x = tape.leaf(2.0);
    let y = x.relu(&mut tape);
    let grads = tape.backward(y.id);
    assert!((grads[x.id.0] - 1.0).abs() < 1e-6);
}

#[test]
fn tc_p33_m1_grad_relu_negative() {
    let mut tape = Tape::new();
    let x = tape.leaf(-1.0);
    let y = x.relu(&mut tape);
    let grads = tape.backward(y.id);
    assert!((grads[x.id.0] - 0.0).abs() < 1e-6);
}

#[test]
fn tc_p33_m1_grad_scale() {
    // z = 3 * x; dz/dx = 3
    let mut tape = Tape::new();
    let x = tape.leaf(4.0);
    let z = x.scale(3.0, &mut tape);
    let grads = tape.backward(z.id);
    assert!((grads[x.id.0] - 3.0).abs() < 1e-6);
}

#[test]
fn tc_p33_m1_grad_neg() {
    let mut tape = Tape::new();
    let x = tape.leaf(5.0);
    let z = x.neg(&mut tape);
    let grads = tape.backward(z.id);
    assert!((grads[x.id.0] - (-1.0)).abs() < 1e-6);
}

#[test]
fn tc_p33_m1_grad_ln() {
    // z = ln(x); dz/dx = 1/x = 0.5 at x=2
    let mut tape = Tape::new();
    let x = tape.leaf(2.0);
    let z = x.ln(&mut tape);
    let grads = tape.backward(z.id);
    assert!((grads[x.id.0] - 0.5).abs() < 1e-5);
}

#[test]
fn tc_p33_m1_tape_len() {
    let mut tape = Tape::new();
    let a = tape.leaf(1.0);
    let b = tape.leaf(2.0);
    let _c = a.add(b, &mut tape);
    assert_eq!(tape.len(), 3);
}
