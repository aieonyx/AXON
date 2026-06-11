// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
//
// axon_math::stats — Statistical Primitives
// no_std | libm for sqrt | works on fixed-length slices

use num_traits::{Float, Zero, NumCast};
use core::ops::{Add, Sub, Mul, Div, AddAssign};

// ------------------------------------------------------------------
// Internal helpers
// ------------------------------------------------------------------

/// Sum all elements of a slice. Returns Zero if empty.
#[inline]
fn slice_sum<T>(data: &[T]) -> T
where
    T: Copy + Zero + AddAssign,
{
    let mut acc = T::zero();
    for &x in data {
        acc += x;
    }
    acc
}

/// Convert usize to T via NumCast. Returns None if conversion fails.
#[inline]
fn usize_to<T: NumCast>(n: usize) -> Option<T> {
    T::from(n)
}

// ------------------------------------------------------------------
// mean — arithmetic mean of a slice
// ------------------------------------------------------------------
/// Compute the arithmetic mean of a non-empty slice.
/// Returns `None` if the slice is empty or length overflows T.
pub fn mean<T>(data: &[T]) -> Option<T>
where
    T: Copy + Zero + AddAssign + Div<Output = T> + NumCast,
{
    if data.is_empty() {
        return None;
    }
    let sum = slice_sum(data);
    let n: T = usize_to(data.len())?;
    Some(sum / n)
}

// ------------------------------------------------------------------
// variance — population variance
// ------------------------------------------------------------------
/// Compute the population variance of a non-empty slice.
/// Returns `None` if the slice is empty or length overflows T.
pub fn variance<T>(data: &[T]) -> Option<T>
where
    T: Copy
        + Zero
        + AddAssign
        + Add<Output = T>
        + Sub<Output = T>
        + Mul<Output = T>
        + Div<Output = T>
        + NumCast,
{
    let m = mean(data)?;
    let mut acc = T::zero();
    for &x in data {
        let diff = x - m;
        acc += diff * diff;
    }
    let n: T = usize_to(data.len())?;
    Some(acc / n)
}

// ------------------------------------------------------------------
// std_dev — population standard deviation
// ------------------------------------------------------------------
/// Compute the population standard deviation. Requires `Float` for sqrt.
// Finding 2: Float already implies AddAssign; From<u8> replaced by NumCast.
pub fn std_dev<T: Float + NumCast + AddAssign>(data: &[T]) -> Option<T> {
    let v = variance(data)?;
    Some(v.sqrt())
}

// ------------------------------------------------------------------
// normalize — rescale slice to [0, 1]
// ------------------------------------------------------------------
/// Normalize a fixed-length array to the [0, 1] range (min-max scaling).
/// Returns a zeroed array if min == max.
// Finding 2: From<u8> removed — never used inside this function.
pub fn normalize<T: Float, const N: usize>(data: &[T; N]) -> [T; N] {
    let mut min = data[0];
    let mut max = data[0];
    for &x in data.iter() {
        if x < min { min = x; }
        if x > max { max = x; }
    }
    let range = max - min;
    let mut out = [T::zero(); N];
    if range == T::zero() {
        return out;
    }
    for i in 0..N {
        out[i] = (data[i] - min) / range;
    }
    out
}

// ------------------------------------------------------------------
// z_score — standardize a fixed-length array (mean=0, std=1)
// ------------------------------------------------------------------
/// Z-score standardization. Returns zeroed array if std_dev is zero.
pub fn z_score<T: Float + NumCast + AddAssign, const N: usize>(data: &[T; N]) -> [T; N] {
    let m = match mean(data) {
        Some(v) => v,
        None => return [T::zero(); N],
    };
    let s = match std_dev(data) {
        Some(v) => v,
        None => return [T::zero(); N],
    };
    let mut out = [T::zero(); N];
    if s == T::zero() {
        return out;
    }
    for i in 0..N {
        out[i] = (data[i] - m) / s;
    }
    out
}

// ------------------------------------------------------------------
// min / max helpers
// ------------------------------------------------------------------
pub fn slice_min<T: Copy + PartialOrd>(data: &[T]) -> Option<T> {
    if data.is_empty() { return None; }
    let mut m = data[0];
    for &x in &data[1..] { if x < m { m = x; } }
    Some(m)
}

pub fn slice_max<T: Copy + PartialOrd>(data: &[T]) -> Option<T> {
    if data.is_empty() { return None; }
    let mut m = data[0];
    for &x in &data[1..] { if x > m { m = x; } }
    Some(m)
}
