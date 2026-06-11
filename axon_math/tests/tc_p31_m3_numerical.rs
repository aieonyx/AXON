// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
//
// P31-M3 — Numerical Tests: FFT + Integration

use axon_math::numerical::{fft, ifft, fft_f64, integrate_simpson, integrate_simpson_f32};
use num_complex::Complex;

// ------------------------------------------------------------------
// FFT — basic correctness
// ------------------------------------------------------------------

#[test]
fn tc_p31_m3_fft_dc_only() {
    // Input: constant signal [1, 1, 1, 1, 1, 1, 1, 1]
    // FFT[0] should be N (8.0), all others ~0
    let mut data: Vec<Complex<f32>> = (0..8).map(|_| Complex::new(1.0, 0.0)).collect();
    fft(&mut data).unwrap();
    assert!((data[0].re - 8.0).abs() < 1e-5, "DC bin: {}", data[0].re);
    for i in 1..8 {
        assert!(data[i].l1_norm() < 1e-5, "bin {i} should be ~0, got {}", data[i].l1_norm());
    }
}

#[test]
fn tc_p31_m3_fft_impulse() {
    // Input: [1, 0, 0, 0, 0, 0, 0, 0] — FFT is all-ones spectrum
    let mut data: Vec<Complex<f32>> = vec![Complex::new(0.0, 0.0); 8];
    data[0] = Complex::new(1.0, 0.0);
    fft(&mut data).unwrap();
    for i in 0..8 {
        assert!((data[i].re - 1.0).abs() < 1e-5, "bin {i} re: {}", data[i].re);
        assert!(data[i].im.abs() < 1e-5, "bin {i} im: {}", data[i].im);
    }
}

#[test]
fn tc_p31_m3_fft_power_of_2_required() {
    let mut data: Vec<Complex<f32>> = vec![Complex::new(1.0, 0.0); 6];
    assert!(fft(&mut data).is_err());
}

#[test]
fn tc_p31_m3_fft_empty_error() {
    let mut data: Vec<Complex<f32>> = vec![];
    assert!(fft(&mut data).is_err());
}

// ------------------------------------------------------------------
// IFFT roundtrip
// ------------------------------------------------------------------

#[test]
fn tc_p31_m3_ifft_roundtrip() {
    let original: Vec<Complex<f32>> = (0..8)
        .map(|i| Complex::new(i as f32, 0.0))
        .collect();
    let mut data = original.clone();
    fft(&mut data).unwrap();
    ifft(&mut data).unwrap();
    for i in 0..8 {
        assert!(
            (data[i].re - original[i].re).abs() < 1e-4,
            "roundtrip mismatch at {i}: got {}, expected {}",
            data[i].re, original[i].re
        );
        assert!(data[i].im.abs() < 1e-4);
    }
}

// ------------------------------------------------------------------
// FFT f64
// ------------------------------------------------------------------

#[test]
fn tc_p31_m3_fft_f64_dc() {
    let mut data: Vec<Complex<f64>> = (0..8).map(|_| Complex::new(1.0, 0.0)).collect();
    fft_f64(&mut data).unwrap();
    assert!((data[0].re - 8.0).abs() < 1e-12);
    for i in 1..8 {
        assert!(data[i].l1_norm() < 1e-12);
    }
}

// ------------------------------------------------------------------
// Simpson's rule integration
// ------------------------------------------------------------------

#[test]
fn tc_p31_m3_integrate_constant() {
    // ∫₀¹ 5 dx = 5
    let result = integrate_simpson(|_| 5.0, 0.0, 1.0, 100).unwrap();
    assert!((result - 5.0).abs() < 1e-10, "result = {result}");
}

#[test]
fn tc_p31_m3_integrate_linear() {
    // ∫₀¹ x dx = 0.5
    let result = integrate_simpson(|x| x, 0.0, 1.0, 100).unwrap();
    assert!((result - 0.5).abs() < 1e-10, "result = {result}");
}

#[test]
fn tc_p31_m3_integrate_quadratic() {
    // ∫₀¹ x² dx = 1/3
    let result = integrate_simpson(|x| x * x, 0.0, 1.0, 100).unwrap();
    assert!((result - 1.0 / 3.0).abs() < 1e-10, "result = {result}");
}

#[test]
fn tc_p31_m3_integrate_sine() {
    // ∫₀^π sin(x) dx = 2.0
    let result = integrate_simpson(|x| x.sin(), 0.0, core::f64::consts::PI, 1000).unwrap();
    assert!((result - 2.0).abs() < 1e-8, "result = {result}");
}

#[test]
fn tc_p31_m3_integrate_odd_n_error() {
    assert!(integrate_simpson(|x| x, 0.0, 1.0, 3).is_err());
}

#[test]
fn tc_p31_m3_integrate_zero_n_error() {
    assert!(integrate_simpson(|x| x, 0.0, 1.0, 0).is_err());
}

// ------------------------------------------------------------------
// Simpson f32
// ------------------------------------------------------------------

#[test]
fn tc_p31_m3_integrate_f32_quadratic() {
    let result = integrate_simpson_f32(|x| x * x, 0.0_f32, 1.0_f32, 100).unwrap();
    assert!((result - 1.0_f32 / 3.0_f32).abs() < 1e-5, "result = {result}");
}
