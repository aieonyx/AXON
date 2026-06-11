// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
//
// P31-M2 — Statistics Tests

use axon_math::stats::{mean, variance, std_dev, normalize, z_score, slice_min, slice_max};

// ------------------------------------------------------------------
// mean
// ------------------------------------------------------------------

#[test]
fn tc_p31_m2_mean_basic() {
    let data = [1.0f64, 2.0, 3.0, 4.0, 5.0];
    let m = mean(&data).unwrap();
    assert!((m - 3.0).abs() < 1e-10);
}

#[test]
fn tc_p31_m2_mean_single() {
    let data = [42.0f64];
    assert!((mean(&data).unwrap() - 42.0).abs() < 1e-10);
}

#[test]
fn tc_p31_m2_mean_empty() {
    let data: [f64; 0] = [];
    assert!(mean(&data).is_none());
}

#[test]
fn tc_p31_m2_mean_uniform() {
    let data = [7.0f32, 7.0, 7.0, 7.0];
    assert!((mean(&data).unwrap() - 7.0).abs() < 1e-6);
}

// ------------------------------------------------------------------
// variance
// ------------------------------------------------------------------

#[test]
fn tc_p31_m2_variance_basic() {
    // [2, 4, 4, 4, 5, 5, 7, 9] — population variance = 4.0
    let data = [2.0f64, 4.0, 4.0, 4.0, 5.0, 5.0, 7.0, 9.0];
    let v = variance(&data).unwrap();
    assert!((v - 4.0).abs() < 1e-10, "variance = {v}");
}

#[test]
fn tc_p31_m2_variance_single() {
    let data = [5.0f64];
    assert!((variance(&data).unwrap() - 0.0).abs() < 1e-10);
}

#[test]
fn tc_p31_m2_variance_empty() {
    let data: [f64; 0] = [];
    assert!(variance(&data).is_none());
}

// ------------------------------------------------------------------
// std_dev
// ------------------------------------------------------------------

#[test]
fn tc_p31_m2_std_dev_basic() {
    let data = [2.0f64, 4.0, 4.0, 4.0, 5.0, 5.0, 7.0, 9.0];
    let s = std_dev(&data).unwrap();
    assert!((s - 2.0).abs() < 1e-10, "std_dev = {s}");
}

#[test]
fn tc_p31_m2_std_dev_constant() {
    let data = [3.0f64, 3.0, 3.0, 3.0];
    let s = std_dev(&data).unwrap();
    assert!(s.abs() < 1e-10);
}

// ------------------------------------------------------------------
// normalize
// ------------------------------------------------------------------

#[test]
fn tc_p31_m2_normalize_range() {
    let data = [0.0f64, 5.0, 10.0];
    let n = normalize(&data);
    assert!((n[0] - 0.0).abs() < 1e-10);
    assert!((n[1] - 0.5).abs() < 1e-10);
    assert!((n[2] - 1.0).abs() < 1e-10);
}

#[test]
fn tc_p31_m2_normalize_constant() {
    let data = [4.0f64, 4.0, 4.0];
    let n = normalize(&data);
    // All zeros when min == max
    for &x in &n { assert_eq!(x, 0.0); }
}

#[test]
fn tc_p31_m2_normalize_bounds() {
    let data = [1.0f32, 2.0, 3.0, 4.0, 5.0];
    let n = normalize(&data);
    for &x in &n {
        assert!(x >= 0.0 && x <= 1.0, "out of [0,1]: {x}");
    }
}

// ------------------------------------------------------------------
// z_score
// ------------------------------------------------------------------

#[test]
fn tc_p31_m2_zscore_mean_zero() {
    let data = [2.0f64, 4.0, 4.0, 4.0, 5.0, 5.0, 7.0, 9.0];
    let z = z_score(&data);
    let zm = mean(&z).unwrap();
    assert!(zm.abs() < 1e-10, "z-score mean should be ~0, got {zm}");
}

#[test]
fn tc_p31_m2_zscore_std_one() {
    let data = [2.0f64, 4.0, 4.0, 4.0, 5.0, 5.0, 7.0, 9.0];
    let z = z_score(&data);
    let s = std_dev(&z).unwrap();
    assert!((s - 1.0).abs() < 1e-10, "z-score std should be ~1, got {s}");
}

// ------------------------------------------------------------------
// min / max
// ------------------------------------------------------------------

#[test]
fn tc_p31_m2_min_max() {
    let data = [3.0f64, 1.0, 4.0, 1.0, 5.0, 9.0, 2.0, 6.0];
    assert_eq!(slice_min(&data).unwrap(), 1.0);
    assert_eq!(slice_max(&data).unwrap(), 9.0);
}

#[test]
fn tc_p31_m2_min_max_empty() {
    let data: [f64; 0] = [];
    assert!(slice_min(&data).is_none());
    assert!(slice_max(&data).is_none());
}
