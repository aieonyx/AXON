// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
// P63 QA -- axon_ai_runtime HANIEL-ONYX sovereign AI inference tests
// Pass bar: 25/25
// P3 Doctrine: crowns axon_gpu P58, axon_layout P60, axon_font P62
// This is the +i in S4+i.
use axon_ai_runtime::{
    Tensor, AiError,
    matmul, relu, sigmoid, tanh, softmax, add_bias, layer_norm, dot, cross_entropy,
    DenseLayer, SovereignModel, Activation,
    ComputeGraph, GraphNode,
};
use std::collections::HashMap;

// ── Tensor tests ──────────────────────────────────────────────────────────────
#[test]
fn test_tensor_zeros() {
    let t = Tensor::zeros(vec![2, 3]).unwrap();
    assert_eq!(t.shape, vec![2, 3]);
    assert!(t.data.iter().all(|&x| x == 0.0));
}
#[test]
fn test_tensor_ones() {
    let t = Tensor::ones(vec![3]).unwrap();
    assert!(t.data.iter().all(|&x| x == 1.0));
}
#[test]
fn test_tensor_shape_mismatch_fails() {
    assert!(Tensor::new(vec![2, 3], vec![0.0; 5]).is_err());
}
#[test]
fn test_tensor_empty_fails() {
    assert!(Tensor::zeros(vec![0, 3]).is_err());
}
#[test]
fn test_tensor_get_set() {
    let mut t = Tensor::zeros(vec![2, 2]).unwrap();
    t.set(&[1, 0], 3.14).unwrap();
    assert!((t.get(&[1, 0]).unwrap() - 3.14).abs() < 1e-5);
}
#[test]
fn test_tensor_reshape() {
    let t = Tensor::zeros(vec![2, 3]).unwrap();
    let r = t.reshape(vec![6]).unwrap();
    assert_eq!(r.shape, vec![6]);
}
#[test]
fn test_tensor_transpose() {
    let t = Tensor::from_matrix(2, 3, vec![1.,2.,3.,4.,5.,6.]).unwrap();
    let tr = t.transpose().unwrap();
    assert_eq!(tr.shape, vec![3, 2]);
    assert_eq!(tr.data, vec![1.,4.,2.,5.,3.,6.]);
}
#[test]
fn test_tensor_stats() {
    let t = Tensor::from_vec(vec![1.0, 2.0, 3.0, 4.0]).unwrap();
    assert_eq!(t.sum(),  10.0);
    assert_eq!(t.mean(),  2.5);
    assert_eq!(t.max(),   4.0);
    assert_eq!(t.min(),   1.0);
}

// ── Ops tests ─────────────────────────────────────────────────────────────────
#[test]
fn test_matmul_2x2() {
    let a = Tensor::from_matrix(2,2,vec![1.,2.,3.,4.]).unwrap();
    let b = Tensor::from_matrix(2,2,vec![5.,6.,7.,8.]).unwrap();
    let c = matmul(&a, &b).unwrap();
    assert_eq!(c.shape, vec![2, 2]);
    assert!((c.data[0] - 19.0).abs() < 1e-4);
    assert!((c.data[1] - 22.0).abs() < 1e-4);
    assert!((c.data[2] - 43.0).abs() < 1e-4);
    assert!((c.data[3] - 50.0).abs() < 1e-4);
}
#[test]
fn test_relu() {
    let t = Tensor::from_vec(vec![-2.0, -1.0, 0.0, 1.0, 2.0]).unwrap();
    let r = relu(&t);
    assert_eq!(r.data, vec![0.0, 0.0, 0.0, 1.0, 2.0]);
}
#[test]
fn test_sigmoid_range() {
    let t = Tensor::from_vec(vec![-10.0, 0.0, 10.0]).unwrap();
    let s = sigmoid(&t);
    assert!(s.data[0] > 0.0 && s.data[0] < 0.01);
    assert!((s.data[1] - 0.5).abs() < 1e-5);
    assert!(s.data[2] > 0.99 && s.data[2] < 1.0);
}
#[test]
fn test_softmax_sums_to_one() {
    let t = Tensor::from_vec(vec![1.0, 2.0, 3.0]).unwrap();
    let s = softmax(&t).unwrap();
    assert!((s.sum() - 1.0).abs() < 1e-5);
    assert!(s.data.iter().all(|&x| x > 0.0));
}
#[test]
fn test_add_bias() {
    let m = Tensor::from_matrix(2, 3, vec![1.,1.,1.,2.,2.,2.]).unwrap();
    let b = Tensor::from_vec(vec![0.0, 1.0, 2.0]).unwrap();
    let r = add_bias(&m, &b).unwrap();
    assert_eq!(r.data, vec![1.,2.,3.,2.,3.,4.]);
}
#[test]
fn test_layer_norm() {
    let t = Tensor::from_matrix(1, 4, vec![1.,2.,3.,4.]).unwrap();
    let n = layer_norm(&t, 1e-5).unwrap();
    let mean: f32 = n.data.iter().sum::<f32>() / 4.0;
    assert!(mean.abs() < 1e-4);
}
#[test]
fn test_dot_product() {
    let a = Tensor::from_vec(vec![1.0, 2.0, 3.0]).unwrap();
    let b = Tensor::from_vec(vec![4.0, 5.0, 6.0]).unwrap();
    assert!((dot(&a, &b).unwrap() - 32.0).abs() < 1e-5);
}
#[test]
fn test_cross_entropy() {
    let pred = Tensor::from_vec(vec![0.1, 0.7, 0.2]).unwrap();
    let loss = cross_entropy(&pred, 1).unwrap();
    assert!(loss > 0.0);
    assert!((loss - (-0.7f32.ln())).abs() < 1e-4);
}

// ── Model tests ───────────────────────────────────────────────────────────────
#[test]
fn test_dense_layer_forward() {
    // (2,3) weights: input 2-wide -> output 3-wide
    let w = Tensor::from_matrix(2, 3, vec![1.,0.,0., 0.,1.,0.]).unwrap();
    let layer = DenseLayer::new("fc1", w, None, Activation::None).unwrap();
    let input = Tensor::from_vec(vec![1.0, 2.0]).unwrap();
    let out   = layer.forward(&input).unwrap();
    assert_eq!(out.shape, vec![1, 3]);
}
#[test]
fn test_model_empty_fails() {
    let m = SovereignModel::new("empty");
    let input = Tensor::from_vec(vec![1.0]).unwrap();
    assert!(m.infer(&input).is_err());
}
#[test]
fn test_model_single_layer_relu() {
    let w = Tensor::from_matrix(2, 2, vec![1.,0.,0.,1.]).unwrap();
    let layer = DenseLayer::new("l1", w, None, Activation::ReLU).unwrap();
    let mut m = SovereignModel::new("identity_relu");
    m.add_layer(layer);
    let input = Tensor::from_vec(vec![-1.0, 2.0]).unwrap();
    let out   = m.infer(&input).unwrap();
    assert_eq!(out.data[0], 0.0);
    assert_eq!(out.data[1], 2.0);
}
#[test]
fn test_model_predict_class() {
    let w = Tensor::from_matrix(2, 3, vec![1.,0.,0., 0.,0.,1.]).unwrap();
    let layer = DenseLayer::new("out", w, None, Activation::None).unwrap();
    let mut m = SovereignModel::new("classifier");
    m.add_layer(layer);
    let input = Tensor::from_vec(vec![0.0, 5.0]).unwrap();
    let cls   = m.predict_class(&input).unwrap();
    assert_eq!(cls, 2);
}

// ── Graph tests ───────────────────────────────────────────────────────────────
#[test]
fn test_graph_forward_relu() {
    let mut g = ComputeGraph::new();
    g.add_node(GraphNode::input("x", "x"));
    g.add_node(GraphNode::relu("out", "x"));
    let mut inputs = HashMap::new();
    inputs.insert("x".to_string(),
        Tensor::from_vec(vec![-1.0, 0.0, 2.0]).unwrap());
    let vals = g.forward(&inputs).unwrap();
    let out  = &vals["out"];
    assert_eq!(out.data, vec![0.0, 0.0, 2.0]);
}
#[test]
fn test_graph_forward_matmul() {
    let mut g = ComputeGraph::new();
    let w = Tensor::from_matrix(2,2,vec![1.,0.,0.,1.]).unwrap();
    g.add_node(GraphNode::input("x", "x"));
    g.add_node(GraphNode::constant("w", w));
    g.add_node(GraphNode::matmul("out", "x", "w"));
    let mut inputs = HashMap::new();
    inputs.insert("x".to_string(),
        Tensor::from_matrix(1, 2, vec![3.0, 4.0]).unwrap());
    let vals = g.forward(&inputs).unwrap();
    let out  = &vals["out"];
    assert!((out.data[0] - 3.0).abs() < 1e-5);
    assert!((out.data[1] - 4.0).abs() < 1e-5);
}
