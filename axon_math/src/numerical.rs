// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
//
// axon_math::numerical — FFT + Numerical Integration
// Cooley-Tukey radix-2 DIT FFT (power-of-2 lengths)
// Simpson's rule integration
// no_std | num-complex | libm for trig

use num_complex::Complex;
use libm::{cosf, sinf, cos, sin};

// ------------------------------------------------------------------
// Internal: trig via libm (no_std safe)
// ------------------------------------------------------------------

#[inline]
fn cos_f32(x: f32) -> f32 { cosf(x) }
#[inline]
fn sin_f32(x: f32) -> f32 { sinf(x) }
#[inline]
fn cos_f64(x: f64) -> f64 { cos(x) }
#[inline]
fn sin_f64(x: f64) -> f64 { sin(x) }

// ------------------------------------------------------------------
// Bit-reversal permutation (in-place)
// ------------------------------------------------------------------
fn bit_reverse_f32(data: &mut [Complex<f32>]) {
    let n = data.len();
    let mut j = 0usize;
    for i in 1..n {
        let mut bit = n >> 1;
        while j & bit != 0 {
            j ^= bit;
            bit >>= 1;
        }
        j ^= bit;
        if i < j {
            data.swap(i, j);
        }
    }
}

fn bit_reverse_f64(data: &mut [Complex<f64>]) {
    let n = data.len();
    let mut j = 0usize;
    for i in 1..n {
        let mut bit = n >> 1;
        while j & bit != 0 {
            j ^= bit;
            bit >>= 1;
        }
        j ^= bit;
        if i < j {
            data.swap(i, j);
        }
    }
}

// ------------------------------------------------------------------
// fft — in-place Cooley-Tukey radix-2 DIT (f32)
// ------------------------------------------------------------------
/// Compute the FFT of `data` in-place. Length must be a power of 2.
/// Returns `Err` if length is zero or not a power of 2.
pub fn fft(data: &mut [Complex<f32>]) -> Result<(), &'static str> {
    let n = data.len();
    if n == 0 || n & (n - 1) != 0 {
        return Err("FFT length must be a non-zero power of 2");
    }
    bit_reverse_f32(data);
    let mut len = 2usize;
    while len <= n {
        let half = len / 2;
        let ang = -2.0f32 * core::f32::consts::PI / len as f32;
        let wlen = Complex::new(cos_f32(ang), sin_f32(ang));
        let mut i = 0;
        while i < n {
            let mut w = Complex::new(1.0f32, 0.0f32);
            for j in 0..half {
                let u = data[i + j];
                let v = data[i + j + half] * w;
                data[i + j]        = u + v;
                data[i + j + half] = u - v;
                w *= wlen;
            }
            i += len;
        }
        len <<= 1;
    }
    Ok(())
}

// ------------------------------------------------------------------
// ifft — inverse FFT (f32)
// ------------------------------------------------------------------
/// Compute the inverse FFT of `data` in-place. Length must be power of 2.
pub fn ifft(data: &mut [Complex<f32>]) -> Result<(), &'static str> {
    // Conjugate → forward FFT → conjugate → scale by 1/N
    let n = data.len();
    for x in data.iter_mut() {
        *x = x.conj();
    }
    fft(data)?;
    let scale = 1.0f32 / n as f32;
    for x in data.iter_mut() {
        *x = x.conj() * scale;
    }
    Ok(())
}

// ------------------------------------------------------------------
// fft_f64 — same algorithm, f64 precision
// ------------------------------------------------------------------
pub fn fft_f64(data: &mut [Complex<f64>]) -> Result<(), &'static str> {
    let n = data.len();
    if n == 0 || n & (n - 1) != 0 {
        return Err("FFT length must be a non-zero power of 2");
    }
    bit_reverse_f64(data);
    let mut len = 2usize;
    while len <= n {
        let half = len / 2;
        let ang = -2.0f64 * core::f64::consts::PI / len as f64;
        let wlen = Complex::new(cos_f64(ang), sin_f64(ang));
        let mut i = 0;
        while i < n {
            let mut w = Complex::new(1.0f64, 0.0f64);
            for j in 0..half {
                let u = data[i + j];
                let v = data[i + j + half] * w;
                data[i + j]        = u + v;
                data[i + j + half] = u - v;
                w *= wlen;
            }
            i += len;
        }
        len <<= 1;
    }
    Ok(())
}

// ------------------------------------------------------------------
// integrate_simpson — composite Simpson's rule
// ------------------------------------------------------------------
/// Numerically integrate `f` over `[a, b]` using composite Simpson's rule.
/// `n` must be even and > 0. Returns `Err` otherwise.
///
/// Error is O(h^4) where h = (b-a)/n.
pub fn integrate_simpson<F>(f: F, a: f64, b: f64, n: usize) -> Result<f64, &'static str>
where
    F: Fn(f64) -> f64,
{
    if n == 0 || !n.is_multiple_of(2) {
        return Err("integrate_simpson: n must be even and non-zero");
    }
    let h = (b - a) / n as f64;
    let mut sum = f(a) + f(b);
    for i in 1..n {
        let x = a + i as f64 * h;
        sum += if i % 2 == 0 { 2.0 * f(x) } else { 4.0 * f(x) };
    }
    Ok(sum * h / 3.0)
}

// ------------------------------------------------------------------
// integrate_simpson_f32 — f32 variant for embedded paths
// ------------------------------------------------------------------
pub fn integrate_simpson_f32<F>(f: F, a: f32, b: f32, n: usize) -> Result<f32, &'static str>
where
    F: Fn(f32) -> f32,
{
    if n == 0 || !n.is_multiple_of(2) {
        return Err("integrate_simpson_f32: n must be even and non-zero");
    }
    let h = (b - a) / n as f32;
    let mut sum = f(a) + f(b);
    for i in 1..n {
        let x = a + i as f32 * h;
        sum += if i % 2 == 0 { 2.0_f32 * f(x) } else { 4.0_f32 * f(x) };
    }
    Ok(sum * h / 3.0_f32)
}
