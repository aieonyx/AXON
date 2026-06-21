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

// ── P58.1 M2: Vulkan buffer tests ────────────────────────────────────────────

#[cfg(feature = "vulkan")]
mod vulkan_buffer_tests {
    use axon_gpu::vulkan::{VulkanInstance, VulkanPhysicalDevice, VulkanLogicalDevice, VkGpuBuffer};

    fn vk_context() -> Option<(VulkanInstance, VulkanPhysicalDevice, VulkanLogicalDevice)> {
        let vi  = VulkanInstance::new().ok()?;
        let vpd = VulkanPhysicalDevice::select(&vi).ok()?;
        let ld  = VulkanLogicalDevice::new(&vi, &vpd).ok()?;
        Some((vi, vpd, ld))
    }

    #[test]
    fn test_vk_buffer_upload_download_roundtrip() {
        let Some((vi, vpd, ld)) = vk_context() else { return; };
        let data = vec![1.0f32, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0];
        let buf = VkGpuBuffer::new_device(&ld, &vpd.memory_props, (data.len()*4) as u64).unwrap();
        buf.upload(&ld, &vpd.memory_props, &data).unwrap();
        let out = buf.download(&ld, &vpd.memory_props, data.len()).unwrap();
        assert_eq!(data, out, "upload/download round-trip failed");
        buf.destroy(&ld);
    }

    #[test]
    fn test_vk_buffer_zeros_roundtrip() {
        let Some((vi, vpd, ld)) = vk_context() else { return; };
        let data = vec![0.0f32; 16];
        let buf = VkGpuBuffer::new_device(&ld, &vpd.memory_props, (data.len()*4) as u64).unwrap();
        buf.upload(&ld, &vpd.memory_props, &data).unwrap();
        let out = buf.download(&ld, &vpd.memory_props, data.len()).unwrap();
        assert_eq!(data, out);
        buf.destroy(&ld);
    }

    #[test]
    fn test_vk_staging_buffer_alloc() {
        let Some((vi, vpd, ld)) = vk_context() else { return; };
        let staging = VkGpuBuffer::new_staging(&ld, &vpd.memory_props, 1024).unwrap();
        assert!(staging.size >= 1024);
        staging.destroy(&ld);
    }

    #[test]
    fn test_vk_large_buffer_roundtrip() {
        let Some((vi, vpd, ld)) = vk_context() else { return; };
        let data: Vec<f32> = (0..4096).map(|i| i as f32 * 0.1).collect();
        let buf = VkGpuBuffer::new_device(&ld, &vpd.memory_props, (data.len()*4) as u64).unwrap();
        buf.upload(&ld, &vpd.memory_props, &data).unwrap();
        let out = buf.download(&ld, &vpd.memory_props, data.len()).unwrap();
        for (a, b) in data.iter().zip(out.iter()) {
            assert!((a - b).abs() < 1e-6, "mismatch: {} != {}", a, b);
        }
        buf.destroy(&ld);
    }
}

// ── P58.1 M3: Vulkan compute kernel tests ────────────────────────────────────

#[cfg(feature = "vulkan")]
mod vulkan_kernel_tests {
    use axon_gpu::{GpuDevice, GpuBuffer, BufferKind, GpuKernel, KernelOp};

    fn vk_device() -> Option<GpuDevice> {
        let dev = GpuDevice::discover().ok()?;
        if dev.is_vulkan() { Some(dev) } else { None }
    }

    #[test]
    fn test_vk_kernel_add() {
        let Some(dev) = vk_device() else { return; };
        let a   = GpuBuffer::from_slice(BufferKind::Input, &[1.0f32, 2.0, 3.0, 4.0]).unwrap();
        let b   = GpuBuffer::from_slice(BufferKind::Input, &[10.0f32, 20.0, 30.0, 40.0]).unwrap();
        let mut out = GpuBuffer::zeros(BufferKind::Output, 4).unwrap();
        GpuKernel::new(KernelOp::Add).dispatch(&dev, &[&a, &b], &mut out).unwrap();
        let r = out.to_vec();
        assert!((r[0]-11.0).abs()<1e-5 && (r[1]-22.0).abs()<1e-5 &&
                (r[2]-33.0).abs()<1e-5 && (r[3]-44.0).abs()<1e-5,
                "VK Add failed: {:?}", r);
    }

    #[test]
    fn test_vk_kernel_mul() {
        let Some(dev) = vk_device() else { return; };
        let a   = GpuBuffer::from_slice(BufferKind::Input, &[2.0f32, 3.0, 4.0]).unwrap();
        let b   = GpuBuffer::from_slice(BufferKind::Input, &[5.0f32, 6.0, 7.0]).unwrap();
        let mut out = GpuBuffer::zeros(BufferKind::Output, 3).unwrap();
        GpuKernel::new(KernelOp::Mul).dispatch(&dev, &[&a, &b], &mut out).unwrap();
        let r = out.to_vec();
        assert!((r[0]-10.0).abs()<1e-5 && (r[1]-18.0).abs()<1e-5 && (r[2]-28.0).abs()<1e-5,
                "VK Mul failed: {:?}", r);
    }

    #[test]
    fn test_vk_kernel_scale() {
        let Some(dev) = vk_device() else { return; };
        let a   = GpuBuffer::from_slice(BufferKind::Input, &[1.0f32, 2.0, 3.0, 4.0]).unwrap();
        let mut out = GpuBuffer::zeros(BufferKind::Output, 4).unwrap();
        GpuKernel::new(KernelOp::Scale(3.0)).dispatch(&dev, &[&a], &mut out).unwrap();
        let r = out.to_vec();
        assert!((r[0]-3.0).abs()<1e-5 && (r[1]-6.0).abs()<1e-5 &&
                (r[2]-9.0).abs()<1e-5 && (r[3]-12.0).abs()<1e-5,
                "VK Scale failed: {:?}", r);
    }

    #[test]
    fn test_vk_kernel_relu() {
        let Some(dev) = vk_device() else { return; };
        let a   = GpuBuffer::from_slice(BufferKind::Input, &[-2.0f32, -1.0, 0.0, 1.0, 2.0]).unwrap();
        let mut out = GpuBuffer::zeros(BufferKind::Output, 5).unwrap();
        GpuKernel::new(KernelOp::ReLU).dispatch(&dev, &[&a], &mut out).unwrap();
        let r = out.to_vec();
        assert!((r[0]-0.0).abs()<1e-5 && (r[1]-0.0).abs()<1e-5 &&
                (r[2]-0.0).abs()<1e-5 && (r[3]-1.0).abs()<1e-5 && (r[4]-2.0).abs()<1e-5,
                "VK ReLU failed: {:?}", r);
    }

    #[test]
    fn test_vk_kernel_matmul_2x2() {
        let Some(dev) = vk_device() else { return; };
        // [[1,2],[3,4]] * [[5,6],[7,8]] = [[19,22],[43,50]]
        let a   = GpuBuffer::from_slice(BufferKind::Input, &[1.0f32,2.0,3.0,4.0]).unwrap();
        let b   = GpuBuffer::from_slice(BufferKind::Input, &[5.0f32,6.0,7.0,8.0]).unwrap();
        let mut out = GpuBuffer::zeros(BufferKind::Output, 4).unwrap();
        GpuKernel::new(KernelOp::MatMul{rows:2,cols:2,inner:2})
            .dispatch(&dev, &[&a,&b], &mut out).unwrap();
        let r = out.to_vec();
        assert!((r[0]-19.0).abs()<1e-4 && (r[1]-22.0).abs()<1e-4 &&
                (r[2]-43.0).abs()<1e-4 && (r[3]-50.0).abs()<1e-4,
                "VK MatMul 2x2 failed: {:?}", r);
    }
}
