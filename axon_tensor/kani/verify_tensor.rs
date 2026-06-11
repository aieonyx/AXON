// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
//
// axon_tensor — Kani Formal Verification Harnesses
// Phase 32 | Tensor<T,D> bounds + ops correctness

#[cfg(kani)]
mod verify {
    use axon_tensor::Tensor;
    use axon_tensor::ops::TensorOps;

    // ------------------------------------------------------------------
    // Tensor zeros: all elements are zero
    // ------------------------------------------------------------------
    #[kani::proof]
    fn verify_tensor_zeros_rank2() {
        let t = Tensor::<f64, 2>::zeros([3, 3]);
        let i: usize = kani::any();
        let j: usize = kani::any();
        kani::assume(i < 3 && j < 3);
        assert_eq!(t.get([i, j]), 0.0);
    }

    // ------------------------------------------------------------------
    // Tensor set/get roundtrip
    // ------------------------------------------------------------------
    #[kani::proof]
    fn verify_tensor_set_get() {
        let mut t = Tensor::<f64, 2>::zeros([4, 4]);
        let i: usize = kani::any();
        let j: usize = kani::any();
        let v: f64   = kani::any();
        kani::assume(i < 4 && j < 4 && v.is_finite());
        t.set([i, j], v);
        assert_eq!(t.get([i, j]), v);
    }

    // ------------------------------------------------------------------
    // Tensor add: result[i] == a[i] + b[i]
    // ------------------------------------------------------------------
    #[kani::proof]
    #[kani::unwind(4)]
    fn verify_tensor_add_correct() {
        let da: [f64; 4] = kani::any();
        let db: [f64; 4] = kani::any();
        kani::assume(da.iter().all(|x| x.is_finite()));
        kani::assume(db.iter().all(|x| x.is_finite()));
        let a = Tensor::<f64, 1>::from_vec([4], da.to_vec());
        let b = Tensor::<f64, 1>::from_vec([4], db.to_vec());
        let c = a.add(&b);
        let idx: usize = kani::any();
        kani::assume(idx < 4);
        assert!((c.get([idx]) - (da[idx] + db[idx])).abs() < 1e-10);
    }

    // ------------------------------------------------------------------
    // numel == product of shape
    // ------------------------------------------------------------------
    #[kani::proof]
    fn verify_tensor_numel_matches_shape() {
        let t = Tensor::<f32, 3>::zeros([2, 3, 4]);
        assert_eq!(t.numel(), 2 * 3 * 4);
    }

    // ------------------------------------------------------------------
    // strides row-major: strides[last] == 1
    // ------------------------------------------------------------------
    #[kani::proof]
    fn verify_tensor_last_stride_is_one() {
        let t = Tensor::<f32, 3>::zeros([2, 3, 4]);
        let strides = t.strides();
        assert_eq!(strides[2], 1);
    }

    // ------------------------------------------------------------------
    // fill: all elements equal fill value
    // ------------------------------------------------------------------
    #[kani::proof]
    #[kani::unwind(9)]
    fn verify_tensor_fill() {
        let v: f64 = kani::any();
        kani::assume(v.is_finite());
        let mut t = Tensor::<f64, 2>::zeros([3, 3]);
        t.fill(v);
        let i: usize = kani::any();
        let j: usize = kani::any();
        kani::assume(i < 3 && j < 3);
        assert_eq!(t.get([i, j]), v);
    }
}
