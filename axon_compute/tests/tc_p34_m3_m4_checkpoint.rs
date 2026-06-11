// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
//
// P34-M3/M4 — Model Checkpoint Tests

use axon_compute::checkpoint::{
    ModelCheckpoint, save_checkpoint, load_checkpoint,
};
use axon_tensor::DynTensor;
use axon_tensor::ops::TensorOps;

// ------------------------------------------------------------------
// Checkpoint construction
// ------------------------------------------------------------------

#[test]
fn tc_p34_m4_checkpoint_new() {
    let ckpt = ModelCheckpoint::new(100, 0.25);
    assert_eq!(ckpt.step, 100);
    assert!((ckpt.loss - 0.25).abs() < 1e-6);
    assert_eq!(ckpt.weight_count(), 0);
    assert_eq!(ckpt.version, 1);
}

#[test]
fn tc_p34_m4_checkpoint_add_weight() {
    let mut ckpt = ModelCheckpoint::new(1, 0.5);
    let w = DynTensor::from_vec(vec![2,3], vec![1.0,2.0,3.0,4.0,5.0,6.0]);
    ckpt.add_weight("layer0.weight", &w);
    assert_eq!(ckpt.weight_count(), 1);
    assert_eq!(ckpt.total_params(), 6);
}

#[test]
fn tc_p34_m4_checkpoint_get_weight() {
    let mut ckpt = ModelCheckpoint::new(1, 0.1);
    let w = DynTensor::from_vec(vec![2], vec![3.0, 7.0]);
    ckpt.add_weight("bias", &w);
    let entry = ckpt.get_weight("bias").unwrap();
    assert_eq!(entry.data, vec![3.0, 7.0]);
}

#[test]
fn tc_p34_m4_checkpoint_get_missing() {
    let ckpt = ModelCheckpoint::new(1, 0.0);
    assert!(ckpt.get_weight("nonexistent").is_none());
}

#[test]
fn tc_p34_m4_checkpoint_restore_tensor() {
    let mut ckpt = ModelCheckpoint::new(5, 0.3);
    let w = DynTensor::from_vec(vec![2,2], vec![1.0,2.0,3.0,4.0]);
    ckpt.add_weight("W", &w);
    let restored = ckpt.restore_tensor("W").unwrap();
    assert_eq!(restored.shape(), &[2usize, 2]);
    assert_eq!(restored.get(&[0,0]), 1.0);
    assert_eq!(restored.get(&[1,1]), 4.0);
}

#[test]
fn tc_p34_m4_checkpoint_total_params() {
    let mut ckpt = ModelCheckpoint::new(1, 0.0);
    ckpt.add_weight("W1", &DynTensor::zeros(vec![4,4]));  // 16
    ckpt.add_weight("b1", &DynTensor::zeros(vec![4]));    // 4
    ckpt.add_weight("W2", &DynTensor::zeros(vec![4,2]));  // 8
    ckpt.add_weight("b2", &DynTensor::zeros(vec![2]));    // 2
    assert_eq!(ckpt.total_params(), 30);
}

// ------------------------------------------------------------------
// Serialization round-trip
// ------------------------------------------------------------------

#[test]
fn tc_p34_m4_roundtrip_empty() {
    let ckpt = ModelCheckpoint::new(0, 0.0);
    let bytes = save_checkpoint(&ckpt);
    let loaded = load_checkpoint(&bytes).unwrap();
    assert_eq!(loaded.step, 0);
    assert_eq!(loaded.weight_count(), 0);
}

#[test]
fn tc_p34_m4_roundtrip_single_weight() {
    let mut ckpt = ModelCheckpoint::new(42, 0.123);
    let w = DynTensor::from_vec(vec![2,3], vec![1.0,2.0,3.0,4.0,5.0,6.0]);
    ckpt.add_weight("layer.weight", &w);

    let bytes = save_checkpoint(&ckpt);
    let loaded = load_checkpoint(&bytes).unwrap();

    assert_eq!(loaded.step, 42);
    assert!((loaded.loss - 0.123).abs() < 1e-5);
    assert_eq!(loaded.weight_count(), 1);

    let entry = loaded.get_weight("layer.weight").unwrap();
    assert_eq!(entry.shape, vec![2, 3]);
    assert_eq!(entry.data, vec![1.0,2.0,3.0,4.0,5.0,6.0]);
}

#[test]
fn tc_p34_m4_roundtrip_multi_weight() {
    let mut ckpt = ModelCheckpoint::new(100, 0.05);
    ckpt.add_weight("W0", &DynTensor::from_vec(vec![2,2], vec![1.0,2.0,3.0,4.0]));
    ckpt.add_weight("b0", &DynTensor::from_vec(vec![2],   vec![0.1, 0.2]));
    ckpt.add_weight("W1", &DynTensor::from_vec(vec![2,1], vec![5.0, 6.0]));

    let bytes = save_checkpoint(&ckpt);
    let loaded = load_checkpoint(&bytes).unwrap();

    assert_eq!(loaded.weight_count(), 3);
    assert_eq!(loaded.total_params(), 8);

    let b0 = loaded.restore_tensor("b0").unwrap();
    assert!((b0.get(&[0]) - 0.1).abs() < 1e-6);
    assert!((b0.get(&[1]) - 0.2).abs() < 1e-6);
}

#[test]
fn tc_p34_m4_roundtrip_preserves_shape() {
    let mut ckpt = ModelCheckpoint::new(1, 0.0);
    let w = DynTensor::zeros(vec![3, 4, 5]);
    ckpt.add_weight("3d_weight", &w);
    let bytes = save_checkpoint(&ckpt);
    let loaded = load_checkpoint(&bytes).unwrap();
    let entry = loaded.get_weight("3d_weight").unwrap();
    assert_eq!(entry.shape, vec![3, 4, 5]);
    assert_eq!(entry.data.len(), 60);
}

#[test]
fn tc_p34_m4_load_bad_magic() {
    let mut bytes = vec![0u8; 32];
    bytes[0..4].copy_from_slice(b"BADD");
    assert!(load_checkpoint(&bytes).is_err());
}

#[test]
fn tc_p34_m4_load_truncated() {
    let bytes = vec![b'O', b'N', b'Y']; // truncated
    assert!(load_checkpoint(&bytes).is_err());
}

#[test]
fn tc_p34_m4_save_produces_magic() {
    let ckpt = ModelCheckpoint::new(0, 0.0);
    let bytes = save_checkpoint(&ckpt);
    assert_eq!(&bytes[0..4], b"ONYX");
}
