// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
//
// P34-M1 — GPU Dispatch Layer Tests

use axon_compute::dispatch::{
    ComputeBackend, BufferDescriptor, LaunchConfig, KernelLaunch,
};

// ------------------------------------------------------------------
// ComputeBackend
// ------------------------------------------------------------------

#[test]
fn tc_p34_m1_cpu_always_available() {
    assert!(ComputeBackend::Cpu.is_available());
}

#[test]
fn tc_p34_m1_best_available_is_cpu_in_test() {
    // Without cuda/rocm/bastion features, best = CPU
    let best = ComputeBackend::best_available();
    assert_eq!(best, ComputeBackend::Cpu);
}

#[test]
fn tc_p34_m1_cuda_unavailable_without_feature() {
    assert!(!ComputeBackend::Cuda.is_available());
}

// ------------------------------------------------------------------
// BufferDescriptor
// ------------------------------------------------------------------

#[test]
fn tc_p34_m1_buffer_f32_rw() {
    let buf = BufferDescriptor::f32_rw(256, "weights");
    assert_eq!(buf.numel, 256);
    assert_eq!(buf.elem_bytes, 4);
    assert_eq!(buf.byte_size(), 1024);
    assert!(!buf.read_only);
}

#[test]
fn tc_p34_m1_buffer_f32_ro() {
    let buf = BufferDescriptor::f32_ro(64, "input");
    assert!(buf.read_only);
    assert_eq!(buf.byte_size(), 256);
}

#[test]
fn tc_p34_m1_buffer_byte_size() {
    let buf = BufferDescriptor::f32_rw(100, "x");
    assert_eq!(buf.byte_size(), 400); // 100 * 4
}

// ------------------------------------------------------------------
// LaunchConfig
// ------------------------------------------------------------------

#[test]
fn tc_p34_m1_launch_config_linear() {
    let cfg = LaunchConfig::linear(1024, 256);
    assert_eq!(cfg.grid_x, 4);
    assert_eq!(cfg.block_x, 256);
}

#[test]
fn tc_p34_m1_launch_config_linear_non_multiple() {
    // 1000 / 256 = 3 full blocks + 1 partial → 4 blocks
    let cfg = LaunchConfig::linear(1000, 256);
    assert_eq!(cfg.grid_x, 4);
}

#[test]
fn tc_p34_m1_launch_config_matrix() {
    let cfg = LaunchConfig::matrix(64, 64, 16);
    assert_eq!(cfg.grid_x, 4);
    assert_eq!(cfg.grid_y, 4);
    assert_eq!(cfg.block_x, 16);
    assert_eq!(cfg.block_y, 16);
}

#[test]
fn tc_p34_m1_launch_config_total_threads() {
    let cfg = LaunchConfig::linear(256, 256);
    // 1 block * 256 threads
    assert_eq!(cfg.total_threads(), 256);
}

// ------------------------------------------------------------------
// KernelLaunch
// ------------------------------------------------------------------

#[test]
fn tc_p34_m1_kernel_launch_build() {
    let launch = KernelLaunch::new(
        "matmul_f32",
        ComputeBackend::Cpu,
        LaunchConfig::linear(64, 64),
    )
    .with_input(BufferDescriptor::f32_ro(64, "A"))
    .with_output(BufferDescriptor::f32_rw(64, "C"))
    .with_param(8u32);

    assert_eq!(launch.kernel_id, "matmul_f32");
    assert_eq!(launch.inputs.len(), 1);
    assert_eq!(launch.outputs.len(), 1);
    assert_eq!(launch.params[0], 8u32);
}

#[test]
fn tc_p34_m1_kernel_launch_validate_ok() {
    let launch = KernelLaunch::new(
        "relu_f32",
        ComputeBackend::Cpu,
        LaunchConfig::linear(256, 256),
    );
    assert!(launch.validate().is_ok());
}

#[test]
fn tc_p34_m1_kernel_launch_validate_empty_id() {
    let launch = KernelLaunch::new(
        "",
        ComputeBackend::Cpu,
        LaunchConfig::linear(256, 256),
    );
    assert!(launch.validate().is_err());
}

#[test]
fn tc_p34_m1_kernel_launch_validate_unavailable_backend() {
    let launch = KernelLaunch::new(
        "matmul_f32",
        ComputeBackend::Cuda,
        LaunchConfig::linear(256, 256),
    );
    // Cuda feature not enabled → unavailable
    assert!(launch.validate().is_err());
}
