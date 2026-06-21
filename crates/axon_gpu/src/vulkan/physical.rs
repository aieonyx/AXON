// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
// vulkan/physical.rs -- Physical device enumeration and selection.

use ash::vk;
use crate::error::{GpuError, GpuResult};
use super::VulkanInstance;

pub struct VulkanPhysicalDevice {
    pub handle:      vk::PhysicalDevice,
    pub properties:  vk::PhysicalDeviceProperties,
    pub compute_qfi: u32,   // compute queue family index
    pub memory_props: vk::PhysicalDeviceMemoryProperties,
}

impl VulkanPhysicalDevice {
    /// Enumerate and select the best physical device for compute.
    /// Preference: DiscreteGpu > IntegratedGpu > VirtualGpu > Cpu > Other.
    pub fn select(vi: &VulkanInstance) -> GpuResult<Self> {
        let devices = unsafe {
            vi.instance.enumerate_physical_devices().map_err(|e| {
                GpuError::BackendError(format!("enumerate_physical_devices: {:?}", e))
            })?
        };

        if devices.is_empty() {
            return Err(GpuError::NoDevice);
        }

        // Score devices by type
        let score_type = |t: vk::PhysicalDeviceType| -> u32 {
            match t {
                vk::PhysicalDeviceType::DISCRETE_GPU   => 4,
                vk::PhysicalDeviceType::INTEGRATED_GPU => 3,
                vk::PhysicalDeviceType::VIRTUAL_GPU    => 2,
                vk::PhysicalDeviceType::CPU            => 1,
                _                                       => 0,
            }
        };

        let mut best: Option<(u32, vk::PhysicalDevice, vk::PhysicalDeviceProperties, u32)> = None;

        for &dev in &devices {
            let props = unsafe { vi.instance.get_physical_device_properties(dev) };
            let score = score_type(props.device_type);

            // Find a compute queue family
            let qf_props = unsafe {
                vi.instance.get_physical_device_queue_family_properties(dev)
            };
            let compute_qfi = qf_props.iter().enumerate().find_map(|(i, qf)| {
                if qf.queue_flags.contains(vk::QueueFlags::COMPUTE) {
                    Some(i as u32)
                } else {
                    None
                }
            });

            if let Some(qfi) = compute_qfi {
                if best.is_none() || score > best.as_ref().unwrap().0 {
                    best = Some((score, dev, props, qfi));
                }
            }
        }

        let (_, handle, properties, compute_qfi) = best.ok_or(GpuError::NoDevice)?;

        let memory_props = unsafe {
            vi.instance.get_physical_device_memory_properties(handle)
        };

        Ok(VulkanPhysicalDevice { handle, properties, compute_qfi, memory_props })
    }

    /// Device name as a Rust String.
    pub fn name(&self) -> String {
        let bytes = &self.properties.device_name;
        let cstr = unsafe { std::ffi::CStr::from_ptr(bytes.as_ptr()) };
        cstr.to_string_lossy().into_owned()
    }

    /// VRAM size in bytes (heap with DEVICE_LOCAL flag).
    pub fn vram_bytes(&self) -> usize {
        let props = &self.memory_props;
        (0..props.memory_heap_count as usize)
            .filter(|&i| props.memory_heaps[i].flags
                .contains(vk::MemoryHeapFlags::DEVICE_LOCAL))
            .map(|i| props.memory_heaps[i].size as usize)
            .max()
            .unwrap_or(0)
    }
}
