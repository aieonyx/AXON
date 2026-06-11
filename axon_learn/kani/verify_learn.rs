// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
//
// axon_learn — Kani Formal Verification Harnesses
// Phase 33 | tape correctness + loss properties

#[cfg(kani)]
mod verify {
    use axon_learn::tape::Tape;
    use axon_learn::loss::mse;
    use axon_tensor::DynTensor;

    // ------------------------------------------------------------------
    // add gradient: dz/da == 1 and dz/db == 1
    // ------------------------------------------------------------------
    #[kani::proof]
    fn verify_add_grad() {
        let mut tape = Tape::new();
        let a = tape.leaf(kani::any());
        let b = tape.leaf(kani::any());
        let z = a.add(b, &mut tape);
        let grads = tape.backward(z.id);
        assert!((grads[a.id.0] - 1.0).abs() < 1e-6);
        assert!((grads[b.id.0] - 1.0).abs() < 1e-6);
    }

    // ------------------------------------------------------------------
    // mul gradient: dz/da == b.value, dz/db == a.value
    // ------------------------------------------------------------------
    #[kani::proof]
    fn verify_mul_grad() {
        let av: f32 = kani::any();
        let bv: f32 = kani::any();
        kani::assume(av.is_finite() && bv.is_finite());
        let mut tape = Tape::new();
        let a = tape.leaf(av);
        let b = tape.leaf(bv);
        let z = a.mul(b, &mut tape);
        let grads = tape.backward(z.id);
        assert!((grads[a.id.0] - bv).abs() < 1e-5);
        assert!((grads[b.id.0] - av).abs() < 1e-5);
    }

    // ------------------------------------------------------------------
    // square gradient: dz/dx == 2*x
    // ------------------------------------------------------------------
    #[kani::proof]
    fn verify_square_grad() {
        let xv: f32 = kani::any();
        kani::assume(xv.is_finite() && xv.abs() < 1e6);
        let mut tape = Tape::new();
        let x = tape.leaf(xv);
        let z = x.square(&mut tape);
        let grads = tape.backward(z.id);
        assert!((grads[x.id.0] - 2.0 * xv).abs() < 1e-3);
    }

    // ------------------------------------------------------------------
    // relu gradient: 0 or 1
    // ------------------------------------------------------------------
    #[kani::proof]
    fn verify_relu_grad_binary() {
        let xv: f32 = kani::any();
        kani::assume(xv.is_finite() && xv != 0.0);
        let mut tape = Tape::new();
        let x = tape.leaf(xv);
        let z = x.relu(&mut tape);
        let grads = tape.backward(z.id);
        let g = grads[x.id.0];
        assert!(g == 0.0 || g == 1.0);
    }

    // ------------------------------------------------------------------
    // MSE: non-negative for any finite inputs
    // ------------------------------------------------------------------
    #[kani::proof]
    #[kani::unwind(4)]
    fn verify_mse_nonnegative() {
        let p: [f32; 4] = kani::any();
        let t: [f32; 4] = kani::any();
        kani::assume(p.iter().all(|x| x.is_finite()));
        kani::assume(t.iter().all(|x| x.is_finite()));
        let pred   = DynTensor::from_vec(alloc::vec![4], p.to_vec());
        let target = DynTensor::from_vec(alloc::vec![4], t.to_vec());
        let loss = mse(&pred, &target);
        assert!(loss >= 0.0);
    }

    // ------------------------------------------------------------------
    // MSE: zero when pred == target
    // ------------------------------------------------------------------
    #[kani::proof]
    #[kani::unwind(4)]
    fn verify_mse_zero_when_equal() {
        let v: [f32; 4] = kani::any();
        kani::assume(v.iter().all(|x| x.is_finite()));
        let pred   = DynTensor::from_vec(alloc::vec![4], v.to_vec());
        let target = DynTensor::from_vec(alloc::vec![4], v.to_vec());
        let loss = mse(&pred, &target);
        assert!(loss.abs() < 1e-6);
    }
}
