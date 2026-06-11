// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
//
// axon_math — Kani Formal Verification Harnesses
// Phase 31 | arithmetic safety + bounds correctness

#[cfg(kani)]
mod verify {
    use axon_math::linalg::{dot, Matrix, matmul, transpose};
    use axon_math::stats::{mean, variance, normalize};
    use axon_math::numerical::{integrate_simpson};

    // ------------------------------------------------------------------
    // dot product: result is not NaN for finite inputs
    // ------------------------------------------------------------------
    #[kani::proof]
    #[kani::unwind(3)]
    fn verify_dot_f32_finite() {
        let a: [f32; 3] = kani::any();
        let b: [f32; 3] = kani::any();
        kani::assume(a.iter().all(|x| x.is_finite()));
        kani::assume(b.iter().all(|x| x.is_finite()));
        let result = dot(&a, &b);
        assert!(!result.is_nan());
    }

    // ------------------------------------------------------------------
    // dot product: zero vector gives zero result
    // ------------------------------------------------------------------
    #[kani::proof]
    #[kani::unwind(3)]
    fn verify_dot_zero_vector() {
        let a: [f64; 3] = kani::any();
        kani::assume(a.iter().all(|x| x.is_finite()));
        let b = [0.0f64; 3];
        let result = dot(&a, &b);
        assert!(result == 0.0);
    }

    // ------------------------------------------------------------------
    // matmul with identity: result equals input
    // ------------------------------------------------------------------
    #[kani::proof]
    #[kani::unwind(2)]
    fn verify_matmul_identity() {
        let data: [[f64; 2]; 2] = kani::any();
        kani::assume(data.iter().flatten().all(|x: &f64| x.is_finite()));
        let a = Matrix::<f64, 2, 2>::from_array(data);
        let id = Matrix::<f64, 2, 2>::identity();
        let result = matmul(&id, &a);
        for i in 0..2 {
            for j in 0..2 {
                assert!((result.get(i, j) - a.get(i, j)).abs() < 1e-10);
            }
        }
    }

    // ------------------------------------------------------------------
    // transpose double: (A^T)^T == A
    // ------------------------------------------------------------------
    #[kani::proof]
    #[kani::unwind(2)]
    fn verify_transpose_involution() {
        let data: [[f64; 2]; 2] = kani::any();
        kani::assume(data.iter().flatten().all(|x: &f64| x.is_finite()));
        let a = Matrix::<f64, 2, 2>::from_array(data);
        let tt = transpose(&transpose(&a));
        for i in 0..2 {
            for j in 0..2 {
                assert_eq!(tt.get(i, j), a.get(i, j));
            }
        }
    }

    // ------------------------------------------------------------------
    // mean: result within [min, max] of input (NumCast path, no truncation)
    // ------------------------------------------------------------------
    #[kani::proof]
    #[kani::unwind(4)]
    fn verify_mean_bounded() {
        let data: [f64; 4] = kani::any();
        kani::assume(data.iter().all(|x| x.is_finite()));
        if let Some(m) = mean(&data) {
            let mn = data.iter().cloned().fold(f64::INFINITY, f64::min);
            let mx = data.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
            assert!(m >= mn - 1e-10 && m <= mx + 1e-10);
        }
    }

    // ------------------------------------------------------------------
    // variance: always non-negative for finite inputs
    // ------------------------------------------------------------------
    #[kani::proof]
    #[kani::unwind(4)]
    fn verify_variance_nonnegative() {
        let data: [f64; 4] = kani::any();
        kani::assume(data.iter().all(|x| x.is_finite()));
        if let Some(v) = variance(&data) {
            assert!(v >= -1e-10, "variance must be non-negative, got {v}");
        }
    }

    // ------------------------------------------------------------------
    // normalize: output always in [0, 1] for finite inputs
    // ------------------------------------------------------------------
    #[kani::proof]
    #[kani::unwind(4)]
    fn verify_normalize_bounds() {
        let data: [f64; 4] = kani::any();
        kani::assume(data.iter().all(|x| x.is_finite()));
        let out = normalize(&data);
        for &x in &out {
            assert!(x >= -1e-10 && x <= 1.0 + 1e-10);
        }
    }

    // ------------------------------------------------------------------
    // integrate_simpson: constant function integrates exactly
    // ------------------------------------------------------------------
    #[kani::proof]
    fn verify_simpson_constant() {
        // ∫₀¹ 5.0 dx = 5.0, with n=2 (minimum even)
        let result = integrate_simpson(|_| 5.0_f64, 0.0, 1.0, 2).unwrap();
        assert!((result - 5.0).abs() < 1e-10);
    }

    // ------------------------------------------------------------------
    // matrix zeros: all elements are zero
    // ------------------------------------------------------------------
    #[kani::proof]
    fn verify_matrix_zeros() {
        let m = Matrix::<f64, 3, 3>::zeros();
        let i: usize = kani::any();
        let j: usize = kani::any();
        kani::assume(i < 3 && j < 3);
        assert_eq!(m.get(i, j), 0.0);
    }

    // ------------------------------------------------------------------
    // matrix identity: diagonal = 1, off-diagonal = 0
    // ------------------------------------------------------------------
    #[kani::proof]
    fn verify_matrix_identity_diagonal() {
        let m = Matrix::<f64, 3, 3>::identity();
        let i: usize = kani::any();
        let j: usize = kani::any();
        kani::assume(i < 3 && j < 3);
        if i == j {
            assert_eq!(m.get(i, j), 1.0);
        } else {
            assert_eq!(m.get(i, j), 0.0);
        }
    }
}
