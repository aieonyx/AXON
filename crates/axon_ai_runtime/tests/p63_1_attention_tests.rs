// Copyright (c) 2026 Edison Lepiten / AIEONYX
// P63.1 — Transformer attention tests (20 tests)

use axon_ai_runtime::attention_ffi::*;

#[test]
fn t1_alloc_free() {
    let ptr = ai_alloc_f32(16);
    assert!(ptr != 0);
    assert_eq!(ai_free_f32(ptr), 0);
}

#[test]
fn t2_set_get_roundtrip() {
    let ptr = ai_alloc_f32(4);
    ai_set_f32(ptr, 0, 1.5); ai_set_f32(ptr, 1, 2.5);
    ai_set_f32(ptr, 2, -1.0); ai_set_f32(ptr, 3, 0.0);
    assert!((ai_get_f32(ptr, 0) - 1.5).abs() < 1e-6);
    assert!((ai_get_f32(ptr, 1) - 2.5).abs() < 1e-6);
    assert!((ai_get_f32(ptr, 2) + 1.0).abs() < 1e-6);
    assert!((ai_get_f32(ptr, 3)).abs() < 1e-6);
    ai_free_f32(ptr);
}

#[test]
fn t3_dot_product() {
    let a = ai_alloc_f32(3); let b = ai_alloc_f32(3);
    ai_set_f32(a, 0, 1.0); ai_set_f32(a, 1, 2.0); ai_set_f32(a, 2, 3.0);
    ai_set_f32(b, 0, 4.0); ai_set_f32(b, 1, 5.0); ai_set_f32(b, 2, 6.0);
    let dot = ai_dot(a, b, 3);
    assert!((dot - 32.0).abs() < 1e-4, "dot={}", dot);
    ai_free_f32(a); ai_free_f32(b);
}

#[test]
fn t4_dot_zero() {
    let a = ai_alloc_f32(4); let b = ai_alloc_f32(4);
    for i in 0..4 { ai_set_f32(a, i, 0.0); ai_set_f32(b, i, 1.0); }
    assert!(ai_dot(a, b, 4).abs() < 1e-6);
    ai_free_f32(a); ai_free_f32(b);
}

#[test]
fn t5_softmax_sums_to_one() {
    let ptr = ai_alloc_f32(4);
    for i in 0..4 { ai_set_f32(ptr, i, (i + 1) as f32); }
    assert_eq!(ai_softmax_inplace(ptr, 4), 0);
    let sum: f32 = (0..4).map(|i| ai_get_f32(ptr, i)).sum();
    assert!((sum - 1.0).abs() < 1e-5, "sum={}", sum);
    ai_free_f32(ptr);
}

#[test]
fn t6_softmax_uniform() {
    let ptr = ai_alloc_f32(4);
    for i in 0..4 { ai_set_f32(ptr, i, 2.0); }
    ai_softmax_inplace(ptr, 4);
    for i in 0..4 {
        let v = ai_get_f32(ptr, i);
        assert!((v - 0.25).abs() < 1e-5, "v[{}]={}", i, v);
    }
    ai_free_f32(ptr);
}

#[test]
fn t7_softmax_large_values() {
    let ptr = ai_alloc_f32(3);
    ai_set_f32(ptr, 0, 1000.0); ai_set_f32(ptr, 1, 1001.0); ai_set_f32(ptr, 2, 1002.0);
    ai_softmax_inplace(ptr, 3);
    let sum: f32 = (0..3).map(|i| ai_get_f32(ptr, i)).sum();
    assert!((sum - 1.0).abs() < 1e-5);
    assert!(ai_get_f32(ptr, 2) > ai_get_f32(ptr, 1));
    assert!(ai_get_f32(ptr, 1) > ai_get_f32(ptr, 0));
    ai_free_f32(ptr);
}

#[test]
fn t8_matmul_identity() {
    let a = ai_alloc_f32(4); let b = ai_alloc_f32(4); let c = ai_alloc_f32(4);
    ai_set_f32(a, 0, 1.0); ai_set_f32(a, 1, 2.0); ai_set_f32(a, 2, 3.0); ai_set_f32(a, 3, 4.0);
    ai_set_f32(b, 0, 1.0); ai_set_f32(b, 1, 0.0); ai_set_f32(b, 2, 0.0); ai_set_f32(b, 3, 1.0);
    ai_matmul_flat(a, b, c, 2, 2, 2);
    assert!((ai_get_f32(c, 0) - 1.0).abs() < 1e-5);
    assert!((ai_get_f32(c, 1) - 2.0).abs() < 1e-5);
    assert!((ai_get_f32(c, 2) - 3.0).abs() < 1e-5);
    assert!((ai_get_f32(c, 3) - 4.0).abs() < 1e-5);
    ai_free_f32(a); ai_free_f32(b); ai_free_f32(c);
}

#[test]
fn t9_matmul_rectangular() {
    let a = ai_alloc_f32(6); let b = ai_alloc_f32(6); let c = ai_alloc_f32(4);
    for (i, v) in [1.0f32,2.,3.,4.,5.,6.].iter().enumerate() { ai_set_f32(a, i as i64, *v); }
    for (i, v) in [1.0f32,2.,3.,4.,5.,6.].iter().enumerate() { ai_set_f32(b, i as i64, *v); }
    ai_matmul_flat(a, b, c, 2, 2, 3);
    assert!((ai_get_f32(c, 0) - 22.0).abs() < 1e-3, "c00={}", ai_get_f32(c,0));
    assert!((ai_get_f32(c, 1) - 28.0).abs() < 1e-3, "c01={}", ai_get_f32(c,1));
    assert!((ai_get_f32(c, 2) - 49.0).abs() < 1e-3, "c10={}", ai_get_f32(c,2));
    assert!((ai_get_f32(c, 3) - 64.0).abs() < 1e-3, "c11={}", ai_get_f32(c,3));
    ai_free_f32(a); ai_free_f32(b); ai_free_f32(c);
}

#[test]
fn t10_scale_inplace() {
    let ptr = ai_alloc_f32(4);
    for i in 0..4 { ai_set_f32(ptr, i, 2.0); }
    ai_scale_inplace(ptr, 4, 3.0);
    for i in 0..4 { assert!((ai_get_f32(ptr, i) - 6.0).abs() < 1e-5); }
    ai_free_f32(ptr);
}

#[test]
fn t11_add_inplace() {
    let dst = ai_alloc_f32(3); let src = ai_alloc_f32(3);
    for i in 0..3 { ai_set_f32(dst, i, 1.0); ai_set_f32(src, i, 2.0); }
    ai_add_inplace(dst, src, 3);
    for i in 0..3 { assert!((ai_get_f32(dst, i) - 3.0).abs() < 1e-5); }
    ai_free_f32(dst); ai_free_f32(src);
}

#[test]
fn t12_relu_inplace() {
    let ptr = ai_alloc_f32(4);
    ai_set_f32(ptr, 0, -2.0); ai_set_f32(ptr, 1, 0.0);
    ai_set_f32(ptr, 2, 1.5);  ai_set_f32(ptr, 3, -0.1);
    ai_relu_inplace(ptr, 4);
    assert!((ai_get_f32(ptr, 0)).abs() < 1e-6);
    assert!((ai_get_f32(ptr, 1)).abs() < 1e-6);
    assert!((ai_get_f32(ptr, 2) - 1.5).abs() < 1e-5);
    assert!((ai_get_f32(ptr, 3)).abs() < 1e-6);
    ai_free_f32(ptr);
}

#[test]
fn t13_scores_shape() {
    let seq_len: i64 = 4; let d_k: i64 = 4;
    let q = ai_alloc_f32(seq_len * d_k);
    let k = ai_alloc_f32(seq_len * d_k);
    for i in 0..(seq_len * d_k) {
        ai_set_f32(q, i, (i + 1) as f32 * 0.1);
        ai_set_f32(k, i, (i + 1) as f32 * 0.1);
    }
    let kt = ai_alloc_f32(seq_len * d_k);
    for i in 0..(seq_len as usize) {
        for j in 0..(d_k as usize) {
            let val = ai_get_f32(k, (i * d_k as usize + j) as i64);
            ai_set_f32(kt, (j * seq_len as usize + i) as i64, val);
        }
    }
    let scores = ai_alloc_f32(seq_len * seq_len);
    assert_eq!(ai_matmul_flat(q, kt, scores, seq_len, seq_len, d_k), 0);
    ai_scale_inplace(scores, seq_len * seq_len, 0.5);
    for i in 0..(seq_len * seq_len) {
        assert!(ai_get_f32(scores, i).is_finite(), "scores[{}] not finite", i);
    }
    ai_free_f32(q); ai_free_f32(k); ai_free_f32(kt); ai_free_f32(scores);
}

#[test]
fn t14_softmax_score_rows() {
    let seq_len: i64 = 4;
    let scores = ai_alloc_f32(seq_len * seq_len);
    for i in 0..(seq_len * seq_len) { ai_set_f32(scores, i, (i % 4) as f32); }
    for i in 0..seq_len {
        ai_softmax_inplace(scores + i * seq_len * 4, seq_len);
    }
    for i in 0..seq_len {
        let sum: f32 = (0..seq_len).map(|j| ai_get_f32(scores, i * seq_len + j)).sum();
        assert!((sum - 1.0).abs() < 1e-5, "row {} sum={}", i, sum);
    }
    ai_free_f32(scores);
}

#[test]
fn t15_attention_output_shape() {
    let seq_len: i64 = 4; let d_k: i64 = 4;
    let weights = ai_alloc_f32(seq_len * seq_len);
    let v = ai_alloc_f32(seq_len * d_k);
    let out = ai_alloc_f32(seq_len * d_k);
    for i in 0..(seq_len * seq_len) { ai_set_f32(weights, i, 0.25); }
    for i in 0..(seq_len * d_k) { ai_set_f32(v, i, (i + 1) as f32); }
    assert_eq!(ai_matmul_flat(weights, v, out, seq_len, d_k, seq_len), 0);
    // uniform weights: each output row = mean of V rows; col0 mean = (1+5+9+13)/4 = 7
    for i in 0..seq_len {
        assert!((ai_get_f32(out, i * d_k) - 7.0).abs() < 1e-3,
            "out[{},0]={}", i, ai_get_f32(out, i * d_k));
    }
    ai_free_f32(weights); ai_free_f32(v); ai_free_f32(out);
}

#[test]
fn t16_e2e_attention_finite() {
    let seq_len: i64 = 2; let d_model: i64 = 4; let d_k: i64 = 2;
    let x = ai_alloc_f32(seq_len * d_model);
    for i in 0..(seq_len * d_model) { ai_set_f32(x, i, (i + 1) as f32 * 0.1); }
    let wq = ai_alloc_f32(d_model * d_k);
    let wk = ai_alloc_f32(d_model * d_k);
    let wv = ai_alloc_f32(d_model * d_k);
    for i in 0..(d_model * d_k) {
        ai_set_f32(wq, i, 0.0); ai_set_f32(wk, i, 0.0); ai_set_f32(wv, i, 0.0);
    }
    for i in 0..d_k {
        ai_set_f32(wq, i * d_k + i, 1.0);
        ai_set_f32(wk, i * d_k + i, 1.0);
        ai_set_f32(wv, i * d_k + i, 0.5);
    }
    let q = ai_alloc_f32(seq_len * d_k);
    let k = ai_alloc_f32(seq_len * d_k);
    let v = ai_alloc_f32(seq_len * d_k);
    ai_matmul_flat(x, wq, q, seq_len, d_k, d_model);
    ai_matmul_flat(x, wk, k, seq_len, d_k, d_model);
    ai_matmul_flat(x, wv, v, seq_len, d_k, d_model);
    let kt = ai_alloc_f32(d_k * seq_len);
    for i in 0..(seq_len as usize) {
        for j in 0..(d_k as usize) {
            let val = ai_get_f32(k, (i * d_k as usize + j) as i64);
            ai_set_f32(kt, (j * seq_len as usize + i) as i64, val);
        }
    }
    let scores = ai_alloc_f32(seq_len * seq_len);
    ai_matmul_flat(q, kt, scores, seq_len, seq_len, d_k);
    ai_scale_inplace(scores, seq_len * seq_len, 0.7071068);
    for i in 0..seq_len { ai_softmax_inplace(scores + i * seq_len * 4, seq_len); }
    let out = ai_alloc_f32(seq_len * d_k);
    ai_matmul_flat(scores, v, out, seq_len, d_k, seq_len);
    for i in 0..(seq_len * d_k) {
        let val = ai_get_f32(out, i);
        assert!(val.is_finite(), "out[{}]={} not finite", i, val);
        assert!(val > 0.0, "out[{}]={} must be positive", i, val);
    }
    ai_free_f32(x); ai_free_f32(wq); ai_free_f32(wk); ai_free_f32(wv);
    ai_free_f32(q); ai_free_f32(k); ai_free_f32(v); ai_free_f32(kt);
    ai_free_f32(scores); ai_free_f32(out);
}

#[test]
fn t17_full_softmax_row_sum() {
    let seq_len: i64 = 4;
    let scores = ai_alloc_f32(seq_len * seq_len);
    for i in 0..seq_len {
        for j in 0..seq_len { ai_set_f32(scores, i * seq_len + j, (j + 1) as f32 * 0.5); }
    }
    for i in 0..seq_len { ai_softmax_inplace(scores + i * seq_len * 4, seq_len); }
    for i in 0..seq_len {
        let sum: f32 = (0..seq_len).map(|j| ai_get_f32(scores, i * seq_len + j)).sum();
        assert!((sum - 1.0).abs() < 1e-5, "row {} sum={}", i, sum);
        let last = ai_get_f32(scores, i * seq_len + (seq_len - 1));
        let first = ai_get_f32(scores, i * seq_len);
        assert!(last > first, "highest score must get highest weight");
    }
    ai_free_f32(scores);
}

#[test]
fn t18_causal_mask() {
    let seq_len: i64 = 4;
    let scores = ai_alloc_f32(seq_len * seq_len);
    for i in 0..(seq_len * seq_len) { ai_set_f32(scores, i, 1.0); }
    for i in 0..seq_len {
        for j in (i + 1)..seq_len {
            ai_set_f32(scores, i * seq_len + j, -1e9);
        }
    }
    for i in 0..seq_len { ai_softmax_inplace(scores + i * seq_len * 4, seq_len); }
    for i in 0..seq_len {
        for j in (i + 1)..seq_len {
            let v = ai_get_f32(scores, i * seq_len + j);
            assert!(v < 1e-6, "masked[{},{}]={} must be ~0", i, j, v);
        }
        let sum: f32 = (0..seq_len).map(|j| ai_get_f32(scores, i * seq_len + j)).sum();
        assert!((sum - 1.0).abs() < 1e-4, "causal row {} sum={}", i, sum);
    }
    ai_free_f32(scores);
}

#[test]
fn t19_attention_manual_verify() {
    // Q=K=I[2x2], V=[[1,2],[3,4]]
    // expected: row0≈[1.65,2.65], row1≈[2.35,3.35]
    let q = ai_alloc_f32(4); let k = ai_alloc_f32(4); let v_mat = ai_alloc_f32(4);
    ai_set_f32(q, 0, 1.0); ai_set_f32(q, 1, 0.0); ai_set_f32(q, 2, 0.0); ai_set_f32(q, 3, 1.0);
    ai_set_f32(k, 0, 1.0); ai_set_f32(k, 1, 0.0); ai_set_f32(k, 2, 0.0); ai_set_f32(k, 3, 1.0);
    ai_set_f32(v_mat, 0, 1.0); ai_set_f32(v_mat, 1, 2.0);
    ai_set_f32(v_mat, 2, 3.0); ai_set_f32(v_mat, 3, 4.0);
    // K^T = K (symmetric)
    let kt = ai_alloc_f32(4);
    ai_set_f32(kt, 0, 1.0); ai_set_f32(kt, 1, 0.0); ai_set_f32(kt, 2, 0.0); ai_set_f32(kt, 3, 1.0);
    let scores = ai_alloc_f32(4);
    ai_matmul_flat(q, kt, scores, 2, 2, 2);
    ai_scale_inplace(scores, 4, 0.7071068f32);
    ai_softmax_inplace(scores, 2);
    ai_softmax_inplace(scores + 2 * 4, 2);
    let out = ai_alloc_f32(4);
    ai_matmul_flat(scores, v_mat, out, 2, 2, 2);
    assert!((ai_get_f32(out, 0) - 1.65).abs() < 0.05, "out[0,0]={}", ai_get_f32(out,0));
    assert!((ai_get_f32(out, 1) - 2.65).abs() < 0.05, "out[0,1]={}", ai_get_f32(out,1));
    assert!((ai_get_f32(out, 2) - 2.35).abs() < 0.05, "out[1,0]={}", ai_get_f32(out,2));
    assert!((ai_get_f32(out, 3) - 3.35).abs() < 0.05, "out[1,1]={}", ai_get_f32(out,3));
    ai_free_f32(q); ai_free_f32(k); ai_free_f32(v_mat);
    ai_free_f32(kt); ai_free_f32(scores); ai_free_f32(out);
}

#[test]
fn t20_null_guard() {
    assert_eq!(ai_dot(0, 0, 4), 0.0);
    assert_eq!(ai_softmax_inplace(0, 4), -1);
    assert_eq!(ai_matmul_flat(0, 0, 0, 2, 2, 2), -1);
    assert_eq!(ai_scale_inplace(0, 4, 1.0), -1);
    assert_eq!(ai_add_inplace(0, 0, 4), -1);
    assert_eq!(ai_relu_inplace(0, 4), -1);
}
