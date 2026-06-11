// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
//
// P34-M2 — Kernel Dispatch + AWP Mesh Tests

use axon_compute::dispatch::ComputeBackend;
use axon_compute::kernel::{
    matmul_dispatch, elementwise_add_dispatch,
    elementwise_mul_dispatch, relu_dispatch, inference_pass,
};
use axon_compute::mesh::{
    MeshNode, NodeCapability, MeshDispatcher, TaskPriority,
};
use axon_learn::layers::Linear;
use axon_tensor::DynTensor;
use axon_tensor::ops::TensorOps;

// ------------------------------------------------------------------
// Kernel dispatch — CPU path
// ------------------------------------------------------------------

#[test]
fn tc_p34_m2_matmul_cpu() {
    let a = DynTensor::from_vec(vec![2,2], vec![1.0,2.0,3.0,4.0]);
    let b = DynTensor::from_vec(vec![2,2], vec![5.0,6.0,7.0,8.0]);
    let c = matmul_dispatch(&a, &b, ComputeBackend::Cpu).unwrap();
    assert!((c.get(&[0,0]) - 19.0).abs() < 1e-5);
    assert!((c.get(&[1,1]) - 50.0).abs() < 1e-5);
}

#[test]
fn tc_p34_m2_matmul_rank_error() {
    let a = DynTensor::zeros(vec![2,2,2]);
    let b = DynTensor::zeros(vec![2,2]);
    assert!(matmul_dispatch(&a, &b, ComputeBackend::Cpu).is_err());
}

#[test]
fn tc_p34_m2_matmul_dim_mismatch() {
    let a = DynTensor::zeros(vec![2,3]);
    let b = DynTensor::zeros(vec![2,2]);
    assert!(matmul_dispatch(&a, &b, ComputeBackend::Cpu).is_err());
}

#[test]
fn tc_p34_m2_elementwise_add_cpu() {
    let a = DynTensor::from_vec(vec![4], vec![1.0,2.0,3.0,4.0]);
    let b = DynTensor::from_vec(vec![4], vec![10.0,20.0,30.0,40.0]);
    let c = elementwise_add_dispatch(&a, &b, ComputeBackend::Cpu).unwrap();
    assert_eq!(c.get(&[0]), 11.0);
    assert_eq!(c.get(&[3]), 44.0);
}

#[test]
fn tc_p34_m2_elementwise_add_shape_mismatch() {
    let a = DynTensor::zeros(vec![4]);
    let b = DynTensor::zeros(vec![3]);
    assert!(elementwise_add_dispatch(&a, &b, ComputeBackend::Cpu).is_err());
}

#[test]
fn tc_p34_m2_elementwise_mul_cpu() {
    let a = DynTensor::from_vec(vec![3], vec![2.0,3.0,4.0]);
    let b = DynTensor::from_vec(vec![3], vec![5.0,6.0,7.0]);
    let c = elementwise_mul_dispatch(&a, &b, ComputeBackend::Cpu).unwrap();
    assert_eq!(c.get(&[0]), 10.0);
    assert_eq!(c.get(&[2]), 28.0);
}

#[test]
fn tc_p34_m2_relu_cpu() {
    let x = DynTensor::from_vec(vec![4], vec![1.0,-1.0,2.0,-2.0]);
    let y = relu_dispatch(&x, ComputeBackend::Cpu).unwrap();
    assert_eq!(y.get(&[0]), 1.0);
    assert_eq!(y.get(&[1]), 0.0);
    assert_eq!(y.get(&[2]), 2.0);
    assert_eq!(y.get(&[3]), 0.0);
}

#[test]
fn tc_p34_m2_inference_pass_shape() {
    let layer = Linear::new(4, 3);
    let input = DynTensor::zeros(vec![2, 4]);
    let out = inference_pass(&input, &layer, ComputeBackend::Cpu).unwrap();
    assert_eq!(out.shape(), &[2usize, 3]);
}

#[test]
fn tc_p34_m2_inference_pass_softmax_sums() {
    let layer = Linear::new(4, 3);
    let input = DynTensor::from_vec(vec![1,4], vec![1.0,2.0,3.0,4.0]);
    let out = inference_pass(&input, &layer, ComputeBackend::Cpu).unwrap();
    let s: f32 = (0..3).map(|j| out.get(&[0,j])).sum();
    assert!((s - 1.0).abs() < 1e-5);
}

// ------------------------------------------------------------------
// AWP Mesh dispatcher
// ------------------------------------------------------------------

#[test]
fn tc_p34_m2_mesh_register_nodes() {
    let mut dispatcher = MeshDispatcher::new();
    dispatcher.register_node(MeshNode::new(1, NodeCapability::Cpu, "node-1"));
    dispatcher.register_node(MeshNode::new(2, NodeCapability::Gpu, "node-2"));
    assert_eq!(dispatcher.node_count(), 2);
    assert_eq!(dispatcher.available_count(), 2);
}

#[test]
fn tc_p34_m2_mesh_submit_task() {
    let mut dispatcher = MeshDispatcher::new();
    let id = dispatcher.submit("matmul_f32");
    assert_eq!(id, 1);
    assert_eq!(dispatcher.pending_count(), 1);
}

#[test]
fn tc_p34_m2_mesh_task_lifecycle() {
    let mut dispatcher = MeshDispatcher::new();
    let id = dispatcher.submit("relu_f32");
    assert!(dispatcher.mark_running(id));
    assert_eq!(dispatcher.pending_count(), 0);
    assert!(dispatcher.mark_complete(id));
    let done = dispatcher.drain_complete();
    assert_eq!(done.len(), 1);
    assert_eq!(done[0].task_id, id);
}

#[test]
fn tc_p34_m2_mesh_select_gpu_node() {
    let mut dispatcher = MeshDispatcher::new();
    dispatcher.register_node(MeshNode::new(1, NodeCapability::Cpu, "cpu-1"));
    dispatcher.register_node(MeshNode::new(2, NodeCapability::Gpu, "gpu-1"));
    let node = dispatcher.select_node(true).unwrap();
    assert_eq!(node.0, 2); // GPU preferred
}

#[test]
fn tc_p34_m2_mesh_select_cpu_fallback() {
    let mut dispatcher = MeshDispatcher::new();
    dispatcher.register_node(MeshNode::new(1, NodeCapability::Cpu, "cpu-1"));
    let node = dispatcher.select_node(true).unwrap(); // no GPU available
    assert_eq!(node.0, 1);
}

#[test]
fn tc_p34_m2_mesh_unavailable_node_excluded() {
    let mut dispatcher = MeshDispatcher::new();
    let mut node = MeshNode::new(1, NodeCapability::Cpu, "cpu-1");
    node.mark_unavailable();
    dispatcher.register_node(node);
    assert_eq!(dispatcher.available_count(), 0);
    assert!(dispatcher.select_node(false).is_none());
}

#[test]
fn tc_p34_m2_mesh_priority_task() {
    let mut dispatcher = MeshDispatcher::new();
    let id = dispatcher.submit_full("matmul_f32", TaskPriority::High, 4096, None);
    assert_eq!(id, 1);
    assert_eq!(dispatcher.pending_count(), 1);
}

#[test]
fn tc_p34_m2_mesh_task_id_increments() {
    let mut dispatcher = MeshDispatcher::new();
    let id1 = dispatcher.submit("k1");
    let id2 = dispatcher.submit("k2");
    let id3 = dispatcher.submit("k3");
    assert_eq!(id1, 1);
    assert_eq!(id2, 2);
    assert_eq!(id3, 3);
}
