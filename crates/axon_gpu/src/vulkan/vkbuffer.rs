// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
// vulkan/vkbuffer.rs -- VkBuffer + VkDeviceMemory with upload/download.

use ash::vk;
use crate::error::{GpuError, GpuResult};
use super::logical::VulkanLogicalDevice;
use super::memory::find_memory_type;

pub struct VkGpuBuffer {
    pub buffer: vk::Buffer,
    pub memory: vk::DeviceMemory,
    pub size:   vk::DeviceSize,
}

impl VkGpuBuffer {
    pub fn new_device(ld: &VulkanLogicalDevice, mem_props: &vk::PhysicalDeviceMemoryProperties, size: vk::DeviceSize) -> GpuResult<Self> {
        let usage = vk::BufferUsageFlags::STORAGE_BUFFER | vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::TRANSFER_SRC;
        Self::alloc(ld, mem_props, size, usage, vk::MemoryPropertyFlags::DEVICE_LOCAL)
    }

    pub fn new_staging(ld: &VulkanLogicalDevice, mem_props: &vk::PhysicalDeviceMemoryProperties, size: vk::DeviceSize) -> GpuResult<Self> {
        let usage = vk::BufferUsageFlags::TRANSFER_SRC | vk::BufferUsageFlags::TRANSFER_DST;
        let props = vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT;
        Self::alloc(ld, mem_props, size, usage, props)
    }

    fn alloc(ld: &VulkanLogicalDevice, mem_props: &vk::PhysicalDeviceMemoryProperties, size: vk::DeviceSize, usage: vk::BufferUsageFlags, properties: vk::MemoryPropertyFlags) -> GpuResult<Self> {
        let buffer_info = vk::BufferCreateInfo::default().size(size).usage(usage).sharing_mode(vk::SharingMode::EXCLUSIVE);
        let buffer = unsafe {
            ld.device.create_buffer(&buffer_info, None)
                .map_err(|e| GpuError::BackendError(format!("create_buffer: {:?}", e)))?
        };
        let mem_reqs = unsafe { ld.device.get_buffer_memory_requirements(buffer) };
        let mem_type = find_memory_type(mem_props, mem_reqs.memory_type_bits, properties)
            .or_else(|_| {
                // On unified memory (APU), DEVICE_LOCAL may need HOST_VISIBLE too
                if properties == vk::MemoryPropertyFlags::DEVICE_LOCAL {
                    find_memory_type(mem_props, mem_reqs.memory_type_bits,
                        vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT)
                } else { find_memory_type(mem_props, mem_reqs.memory_type_bits, properties) }
            })?;
        let alloc_info = vk::MemoryAllocateInfo::default()
            .allocation_size(mem_reqs.size).memory_type_index(mem_type);
        let memory = unsafe {
            ld.device.allocate_memory(&alloc_info, None)
                .map_err(|_| { unsafe { ld.device.destroy_buffer(buffer, None) }; GpuError::OutOfMemory(mem_reqs.size as usize) })?
        };
        unsafe {
            ld.device.bind_buffer_memory(buffer, memory, 0)
                .map_err(|e| GpuError::BackendError(format!("bind_buffer_memory: {:?}", e)))?;
        }
        Ok(VkGpuBuffer { buffer, memory, size })
    }

    pub fn upload(&self, ld: &VulkanLogicalDevice, mem_props: &vk::PhysicalDeviceMemoryProperties, data: &[f32]) -> GpuResult<()> {
        let byte_size = (data.len() * 4) as vk::DeviceSize;
        let staging = VkGpuBuffer::new_staging(ld, mem_props, byte_size)?;
        unsafe {
            let ptr = ld.device.map_memory(staging.memory, 0, byte_size, vk::MemoryMapFlags::empty())
                .map_err(|e| GpuError::BackendError(format!("map_memory: {:?}", e)))?;
            std::ptr::copy_nonoverlapping(data.as_ptr(), ptr as *mut f32, data.len());
            ld.device.unmap_memory(staging.memory);
        }
        ld.one_shot(|cmd| {
            let region = vk::BufferCopy { src_offset: 0, dst_offset: 0, size: byte_size };
            unsafe { ld.device.cmd_copy_buffer(cmd, staging.buffer, self.buffer, &[region]); }
            Ok(())
        })?;
        staging.destroy(ld);
        Ok(())
    }

    pub fn download(&self, ld: &VulkanLogicalDevice, mem_props: &vk::PhysicalDeviceMemoryProperties, len: usize) -> GpuResult<Vec<f32>> {
        let byte_size = (len * 4) as vk::DeviceSize;
        let staging = VkGpuBuffer::new_staging(ld, mem_props, byte_size)?;
        ld.one_shot(|cmd| {
            let region = vk::BufferCopy { src_offset: 0, dst_offset: 0, size: byte_size };
            unsafe { ld.device.cmd_copy_buffer(cmd, self.buffer, staging.buffer, &[region]); }
            Ok(())
        })?;
        let mut out = vec![0.0f32; len];
        unsafe {
            let ptr = ld.device.map_memory(staging.memory, 0, byte_size, vk::MemoryMapFlags::empty())
                .map_err(|e| GpuError::BackendError(format!("map_memory: {:?}", e)))?;
            std::ptr::copy_nonoverlapping(ptr as *const f32, out.as_mut_ptr(), len);
            ld.device.unmap_memory(staging.memory);
        }
        staging.destroy(ld);
        Ok(out)
    }

    pub fn destroy(&self, ld: &VulkanLogicalDevice) {
        unsafe { ld.device.destroy_buffer(self.buffer, None); ld.device.free_memory(self.memory, None); }
    }
}
