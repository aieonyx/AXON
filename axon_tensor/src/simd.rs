// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
//
// axon_tensor::simd — SIMD-accelerated dot product and element-wise ops
// x86_64: SSE2/AVX2 via core::arch (behind cfg guards)
// Scalar fallback for aarch64 / BASTION OS / no simd feature

// ------------------------------------------------------------------
// Scalar fallback — always available
// ------------------------------------------------------------------

/// Scalar dot product of two f32 slices. Reference path.
pub fn dot_f32_scalar(a: &[f32], b: &[f32]) -> f32 {
    assert_eq!(a.len(), b.len(), "dot_f32_scalar: length mismatch");
    let mut acc = 0.0f32;
    for i in 0..a.len() {
        acc += a[i] * b[i];
    }
    acc
}

/// Scalar dot product of two f64 slices. Reference path.
pub fn dot_f64_scalar(a: &[f64], b: &[f64]) -> f64 {
    assert_eq!(a.len(), b.len(), "dot_f64_scalar: length mismatch");
    let mut acc = 0.0f64;
    for i in 0..a.len() {
        acc += a[i] * b[i];
    }
    acc
}

/// Scalar element-wise add into output slice.
pub fn add_f32_scalar(a: &[f32], b: &[f32], out: &mut [f32]) {
    assert_eq!(a.len(), b.len());
    assert_eq!(a.len(), out.len());
    for i in 0..a.len() {
        out[i] = a[i] + b[i];
    }
}

/// Scalar element-wise multiply into output slice.
pub fn mul_f32_scalar(a: &[f32], b: &[f32], out: &mut [f32]) {
    assert_eq!(a.len(), b.len());
    assert_eq!(a.len(), out.len());
    for i in 0..a.len() {
        out[i] = a[i] * b[i];
    }
}

// ------------------------------------------------------------------
// x86_64 SSE2 SIMD paths (feature = "simd" + target_arch)
// ------------------------------------------------------------------

#[cfg(all(feature = "simd", target_arch = "x86_64"))]
pub mod x86 {
    use core::arch::x86_64::*;

    /// SSE2 dot product for f32 slices (processes 4 elements per iteration).
    /// Falls back to scalar for tail elements.
    ///
    /// # Safety
    /// Caller must ensure SSE2 is available (x86_64 baseline — always true).
    pub fn dot_f32_sse2(a: &[f32], b: &[f32]) -> f32 {
        assert_eq!(a.len(), b.len(), "dot_f32_sse2: length mismatch");
        let len = a.len();
        let chunks = len / 4;
        let tail   = len % 4;

        let mut acc = unsafe { _mm_setzero_ps() };

        unsafe {
            for i in 0..chunks {
                let va = _mm_loadu_ps(a.as_ptr().add(i * 4));
                let vb = _mm_loadu_ps(b.as_ptr().add(i * 4));
                acc = _mm_add_ps(acc, _mm_mul_ps(va, vb));
            }
        }

        // Horizontal sum of SSE register
        let mut result = unsafe {
            let shuf = _mm_shuffle_ps(acc, acc, 0b_10_11_00_01);
            let sums = _mm_add_ps(acc, shuf);
            let shuf2 = _mm_movehl_ps(shuf, sums);
            let sums2 = _mm_add_ss(sums, shuf2);
            _mm_cvtss_f32(sums2)
        };

        // Scalar tail
        let tail_start = chunks * 4;
        for i in 0..tail {
            result += a[tail_start + i] * b[tail_start + i];
        }
        result
    }

    /// SSE2 element-wise f32 add (4-wide).
    ///
    /// # Safety
    /// SSE2 required (x86_64 baseline).
    pub fn add_f32_sse2(a: &[f32], b: &[f32], out: &mut [f32]) {
        assert_eq!(a.len(), b.len());
        assert_eq!(a.len(), out.len());
        let len    = a.len();
        let chunks = len / 4;
        let tail   = len % 4;

        unsafe {
            for i in 0..chunks {
                let va = _mm_loadu_ps(a.as_ptr().add(i * 4));
                let vb = _mm_loadu_ps(b.as_ptr().add(i * 4));
                _mm_storeu_ps(out.as_mut_ptr().add(i * 4), _mm_add_ps(va, vb));
            }
        }

        let tail_start = chunks * 4;
        for i in 0..tail {
            out[tail_start + i] = a[tail_start + i] + b[tail_start + i];
        }
    }
}

// ------------------------------------------------------------------
// Public dispatch: use SIMD if available, else scalar
// ------------------------------------------------------------------

/// Dispatch dot product: SIMD on x86_64 with simd feature, scalar otherwise.
pub fn dot_f32(a: &[f32], b: &[f32]) -> f32 {
    #[cfg(all(feature = "simd", target_arch = "x86_64"))]
    return x86::dot_f32_sse2(a, b);

    #[cfg(not(all(feature = "simd", target_arch = "x86_64")))]
    dot_f32_scalar(a, b)
}

/// Dispatch element-wise add: SIMD on x86_64, scalar otherwise.
pub fn add_f32(a: &[f32], b: &[f32], out: &mut [f32]) {
    #[cfg(all(feature = "simd", target_arch = "x86_64"))]
    return x86::add_f32_sse2(a, b, out);

    #[cfg(not(all(feature = "simd", target_arch = "x86_64")))]
    add_f32_scalar(a, b, out)
}
