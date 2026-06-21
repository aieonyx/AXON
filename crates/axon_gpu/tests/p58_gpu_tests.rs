// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
// P58 QA -- axon_gpu sovereign GPU compute tests
// Pass bar: 18/18
use axon_gpu::{
    GpuDevice, GpuBackend, GpuBuffer, BufferKind,
    GpuKernel, KernelOp, GpuError,
};

// ── Device tests ──────────────────────────────────────────────────────────────

#[test]
fn test_device_discover() {
    let dev = GpuDevice::discover().unwrap();
    assert!(dev.max_threads() > 0);
}

#[test]
fn test_device_cpu_fallback() {
    let dev = GpuDevice::cpu_fallback();
    assert!(dev.is_cpu_fallback());
    assert_eq!(*dev.backend(), GpuBackend::CpuFallback);
}

#[test]
fn test_device_name_set() {
    let dev = GpuDevice::cpu_fallback();
    assert!(!dev.device_name().is_empty());
}

// ── Buffer tests ──────────────────────────────────────────────────────────────

#[test]
fn test_buffer_zeros() {
    let buf = GpuBuffer::zeros(BufferKind::Input, 4).unwrap();
    assert_eq!(buf.len, 4);
    assert_eq!(buf.to_vec(), vec![0.0f32; 4]);
}

#[test]
fn test_buffer_from_slice() {
    let data = vec![1.0f32, 2.0, 3.0];
    let buf  = GpuBuffer::from_slice(BufferKind::Input, &data).unwrap();
    assert_eq!(buf.to_vec(), data);
}

#[test]
fn test_buffer_get_set() {
    let mut buf = GpuBuffer::zeros(BufferKind::Output, 4).unwrap();
    buf.set(2, 3.14).unwrap();
    assert!((buf.get(2).unwrap() - 3.14).abs() < 1e-5);
}

#[test]
fn test_buffer_fill() {
    let mut buf = GpuBuffer::zeros(BufferKind::Input, 4).unwrap();
    buf.fill(5.0);
    assert!(buf.to_vec().iter().all(|&x| x == 5.0));
}

#[test]
fn test_buffer_empty_fails() {
    assert!(GpuBuffer::zeros(BufferKind::Input, 0).is_err());
    assert!(GpuBuffer::from_slice(BufferKind::Input, &[]).is_err());
}

#[test]
fn test_buffer_size_bytes() {
    let buf = GpuBuffer::zeros(BufferKind::Input, 8).unwrap();
    assert_eq!(buf.size_bytes(), 32);
}

// ── Kernel tests ──────────────────────────────────────────────────────────────

#[test]
fn test_kernel_add() {
    let dev = GpuDevice::cpu_fallback();
    let a   = GpuBuffer::from_slice(BufferKind::Input, &[1.0, 2.0, 3.0]).unwrap();
    let b   = GpuBuffer::from_slice(BufferKind::Input, &[4.0, 5.0, 6.0]).unwrap();
    let mut out = GpuBuffer::zeros(BufferKind::Output, 3).unwrap();
    GpuKernel::new(KernelOp::Add).dispatch(&dev, &[&a, &b], &mut out).unwrap();
    assert_eq!(out.to_vec(), vec![5.0, 7.0, 9.0]);
}

#[test]
fn test_kernel_mul() {
    let dev = GpuDevice::cpu_fallback();
    let a   = GpuBuffer::from_slice(BufferKind::Input, &[2.0, 3.0]).unwrap();
    let b   = GpuBuffer::from_slice(BufferKind::Input, &[4.0, 5.0]).unwrap();
    let mut out = GpuBuffer::zeros(BufferKind::Output, 2).unwrap();
    GpuKernel::new(KernelOp::Mul).dispatch(&dev, &[&a, &b], &mut out).unwrap();
    assert_eq!(out.to_vec(), vec![8.0, 15.0]);
}

#[test]
fn test_kernel_scale() {
    let dev = GpuDevice::cpu_fallback();
    let a   = GpuBuffer::from_slice(BufferKind::Input, &[1.0, 2.0, 3.0]).unwrap();
    let mut out = GpuBuffer::zeros(BufferKind::Output, 3).unwrap();
    GpuKernel::new(KernelOp::Scale(2.0)).dispatch(&dev, &[&a], &mut out).unwrap();
    assert_eq!(out.to_vec(), vec![2.0, 4.0, 6.0]);
}

#[test]
fn test_kernel_relu() {
    let dev = GpuDevice::cpu_fallback();
    let a   = GpuBuffer::from_slice(BufferKind::Input, &[-1.0, 0.0, 2.0, -0.5]).unwrap();
    let mut out = GpuBuffer::zeros(BufferKind::Output, 4).unwrap();
    GpuKernel::new(KernelOp::ReLU).dispatch(&dev, &[&a], &mut out).unwrap();
    assert_eq!(out.to_vec(), vec![0.0, 0.0, 2.0, 0.0]);
}

#[test]
fn test_kernel_dot_product() {
    let dev = GpuDevice::cpu_fallback();
    let a   = GpuBuffer::from_slice(BufferKind::Input, &[1.0, 2.0, 3.0]).unwrap();
    let b   = GpuBuffer::from_slice(BufferKind::Input, &[4.0, 5.0, 6.0]).unwrap();
    let mut out = GpuBuffer::zeros(BufferKind::Output, 1).unwrap();
    GpuKernel::new(KernelOp::DotProduct).dispatch(&dev, &[&a, &b], &mut out).unwrap();
    assert!((out.get(0).unwrap() - 32.0).abs() < 1e-5);
}

#[test]
fn test_kernel_matmul() {
    let dev = GpuDevice::cpu_fallback();
    // 2x2 * 2x2 = 2x2
    // [1,2] * [5,6] = [1*5+2*7, 1*6+2*8] = [19, 22]
    // [3,4]   [7,8]   [3*5+4*7, 3*6+4*8]   [43, 50]
    let a = GpuBuffer::from_slice(BufferKind::Input, &[1.0,2.0,3.0,4.0]).unwrap();
    let b = GpuBuffer::from_slice(BufferKind::Input, &[5.0,6.0,7.0,8.0]).unwrap();
    let mut out = GpuBuffer::zeros(BufferKind::Output, 4).unwrap();
    GpuKernel::new(KernelOp::MatMul { rows:2, cols:2, inner:2 })
        .dispatch(&dev, &[&a, &b], &mut out).unwrap();
    let r = out.to_vec();
    assert!((r[0]-19.0).abs()<1e-4);
    assert!((r[1]-22.0).abs()<1e-4);
    assert!((r[2]-43.0).abs()<1e-4);
    assert!((r[3]-50.0).abs()<1e-4);
}

#[test]
fn test_kernel_shape_mismatch_fails() {
    let dev = GpuDevice::cpu_fallback();
    let a   = GpuBuffer::from_slice(BufferKind::Input, &[1.0, 2.0]).unwrap();
    let b   = GpuBuffer::from_slice(BufferKind::Input, &[1.0, 2.0, 3.0]).unwrap();
    let mut out = GpuBuffer::zeros(BufferKind::Output, 2).unwrap();
    let result = GpuKernel::new(KernelOp::Add).dispatch(&dev, &[&a, &b], &mut out);
    assert!(result.is_err());
}

// ── P58.1 M1: Vulkan probe tests ─────────────────────────────────────────────

#[test]
fn test_vulkan_discover_or_fallback() {
    // discover() must always succeed — Vulkan or CPU fallback
    let dev = GpuDevice::discover().unwrap();
    assert!(dev.max_threads() > 0);
    assert!(!dev.device_name().is_empty());
    println!("P58.1 backend: {} — {}", dev.backend(), dev.device_name());
}

#[test]
fn test_vulkan_backend_detected() {
    let dev = GpuDevice::discover().unwrap();
    // On this machine (AMD RADV) we expect Vulkan
    // Test is informational — both backends are valid
    println!("Backend: {:?}", dev.backend());
    println!("Device:  {}", dev.device_name());
    println!("VRAM:    {} MB", dev.vram_bytes() / (1024*1024));
    println!("Threads: {}", dev.max_threads());
    // Assert it's one of the valid backends
    let valid = dev.is_vulkan() || dev.is_cpu_fallback();
    assert!(valid, "backend must be Vulkan or CpuFallback");
}

#[test]
fn test_vulkan_vram_nonzero_if_vulkan() {
    let dev = GpuDevice::discover().unwrap();
    if dev.is_vulkan() {
        assert!(dev.vram_bytes() > 0, "Vulkan device should report VRAM > 0");
    }
}

#[test]
fn test_cpu_fallback_explicit() {
    let dev = GpuDevice::cpu_fallback();
    assert!(dev.is_cpu_fallback());
    assert_eq!(dev.vram_bytes(), 0);
}

#[test]
fn test_backend_display() {
    assert_eq!(format!("{}", GpuBackend::Vulkan),      "Vulkan");
    assert_eq!(format!("{}", GpuBackend::CpuFallback), "CPU Fallback");
}
