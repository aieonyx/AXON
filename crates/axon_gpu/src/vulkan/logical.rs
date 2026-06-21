// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
// vulkan/logical.rs -- VkDevice, compute VkQueue, VkCommandPool.

use ash::{vk, Device};
use crate::error::{GpuError, GpuResult};
use super::{VulkanInstance, VulkanPhysicalDevice};

pub struct VulkanLogicalDevice {
    pub device:        Device,
    pub compute_queue: vk::Queue,
    pub command_pool:  vk::CommandPool,
    pub compute_qfi:   u32,
}

impl VulkanLogicalDevice {
    pub fn new(vi: &VulkanInstance, vpd: &VulkanPhysicalDevice) -> GpuResult<Self> {
        let queue_priority = [1.0f32];
        let queue_info = vk::DeviceQueueCreateInfo::default()
            .queue_family_index(vpd.compute_qfi)
            .queue_priorities(&queue_priority);
        let queue_infos = [queue_info];
        let device_info = vk::DeviceCreateInfo::default()
            .queue_create_infos(&queue_infos);
        let device = unsafe {
            vi.instance.create_device(vpd.handle, &device_info, None)
                .map_err(|e| GpuError::BackendError(format!("VkDevice: {:?}", e)))?
        };
        let compute_queue = unsafe { device.get_device_queue(vpd.compute_qfi, 0) };
        let pool_info = vk::CommandPoolCreateInfo::default()
            .queue_family_index(vpd.compute_qfi)
            .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER);
        let command_pool = unsafe {
            device.create_command_pool(&pool_info, None)
                .map_err(|e| GpuError::BackendError(format!("VkCommandPool: {:?}", e)))?
        };
        Ok(VulkanLogicalDevice { device, compute_queue, command_pool, compute_qfi: vpd.compute_qfi })
    }

    pub fn one_shot<F>(&self, f: F) -> GpuResult<()>
    where F: FnOnce(vk::CommandBuffer) -> GpuResult<()> {
        let alloc_info = vk::CommandBufferAllocateInfo::default()
            .command_pool(self.command_pool)
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(1);
        let cmd = unsafe {
            self.device.allocate_command_buffers(&alloc_info)
                .map_err(|e| GpuError::BackendError(format!("alloc cmd: {:?}", e)))?[0]
        };
        let begin_info = vk::CommandBufferBeginInfo::default()
            .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);
        unsafe {
            self.device.begin_command_buffer(cmd, &begin_info)
                .map_err(|e| GpuError::BackendError(format!("begin cmd: {:?}", e)))?;
        }
        f(cmd)?;
        unsafe {
            self.device.end_command_buffer(cmd)
                .map_err(|e| GpuError::BackendError(format!("end cmd: {:?}", e)))?;
        }
        let cmds = [cmd];
        let submit_info = vk::SubmitInfo::default().command_buffers(&cmds);
        let submits = [submit_info];
        unsafe {
            self.device.queue_submit(self.compute_queue, &submits, vk::Fence::null())
                .map_err(|e| GpuError::BackendError(format!("queue_submit: {:?}", e)))?;
            self.device.queue_wait_idle(self.compute_queue)
                .map_err(|e| GpuError::BackendError(format!("queue_wait: {:?}", e)))?;
            self.device.free_command_buffers(self.command_pool, &cmds);
        }
        Ok(())
    }
}

impl Drop for VulkanLogicalDevice {
    fn drop(&mut self) {
        unsafe {
            self.device.destroy_command_pool(self.command_pool, None);
            self.device.destroy_device(None);
        }
    }
}
