// Copyright (c) 2026 Edison Lepitel / AIEONYX
// SPDX-License-Identifier: Apache-2.0
// GpuDevice -- sovereign GPU device abstraction.
// P58.0: CPU fallback backend.
// P58.1: Vulkan backend via ash.
use crate::error::{GpuError, GpuResult};

#[derive(Debug, Clone, PartialEq)]
pub enum GpuBackend {
    CpuFallback,
    Vulkan,
    Metal,
    DirectX12,
}

impl std::fmt::Display for GpuBackend {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            GpuBackend::CpuFallback => write!(f, "CPU Fallback"),
            GpuBackend::Vulkan      => write!(f, "Vulkan"),
            GpuBackend::Metal       => write!(f, "Metal"),
            GpuBackend::DirectX12   => write!(f, "DirectX12"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct GpuCapabilities {
    pub backend:       GpuBackend,
    pub device_name:   String,
    pub vram_bytes:    usize,
    pub max_threads:   usize,
    pub supports_f16:  bool,
    pub supports_f64:  bool,
}

#[derive(Debug)]
pub struct GpuDevice {
    pub caps: GpuCapabilities,
}

impl GpuDevice {
    /// Discover and initialise the best available GPU device.
    /// P58.1: probes Vulkan first, falls back to CPU if unavailable.
    pub fn discover() -> GpuResult<Self> {
        // Try Vulkan first
        #[cfg(feature = "vulkan")]
        {
            if let Ok(dev) = Self::try_vulkan() {
                return Ok(dev);
            }
        }
        // CPU fallback
        Ok(Self::cpu_fallback())
    }

    /// Attempt to initialise a Vulkan device. Returns Err if unavailable.
    #[cfg(feature = "vulkan")]
    fn try_vulkan() -> GpuResult<Self> {
        use crate::vulkan::{VulkanInstance, VulkanPhysicalDevice};
        let vi  = VulkanInstance::new()?;
        let vpd = VulkanPhysicalDevice::select(&vi)?;
        let name = vpd.name();
        let vram = vpd.vram_bytes();
        // max_threads: use subgroup size * max compute invocations as proxy
        let max_invocations = vpd.properties.limits.max_compute_work_group_invocations;
        Ok(GpuDevice {
            caps: GpuCapabilities {
                backend:      GpuBackend::Vulkan,
                device_name:  name,
                vram_bytes:   vram,
                max_threads:  max_invocations as usize,
                supports_f16: false, // detect via features in M2
                supports_f64: true,
            },
        })
    }

    /// Create a CPU fallback device explicitly.
    pub fn cpu_fallback() -> Self {
        GpuDevice {
            caps: GpuCapabilities {
                backend:      GpuBackend::CpuFallback,
                device_name:  "AIEONYX Sovereign CPU Compute".to_string(),
                vram_bytes:   0,
                max_threads:  std::thread::available_parallelism()
                                  .map(|n| n.get())
                                  .unwrap_or(1),
                supports_f16: false,
                supports_f64: true,
            },
        }
    }

    pub fn backend(&self)     -> &GpuBackend { &self.caps.backend }
    pub fn device_name(&self) -> &str        { &self.caps.device_name }
    pub fn max_threads(&self) -> usize       { self.caps.max_threads }
    pub fn vram_bytes(&self)  -> usize       { self.caps.vram_bytes }
    pub fn is_cpu_fallback(&self) -> bool {
        self.caps.backend == GpuBackend::CpuFallback
    }
    pub fn is_vulkan(&self) -> bool {
        self.caps.backend == GpuBackend::Vulkan
    }
}
