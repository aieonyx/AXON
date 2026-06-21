// Copyright (c) 2026 Edison Lepitel / AIEONYX
// SPDX-License-Identifier: Apache-2.0
// vulkan/pipeline.rs -- VkComputePipeline + VkDescriptorSetLayout per kernel.
// P58.1 M3: SPIR-V shader modules and pipeline objects.

use ash::vk;
use crate::error::{GpuError, GpuResult};
use super::logical::VulkanLogicalDevice;

pub struct ComputePipeline {
    pub pipeline:        vk::Pipeline,
    pub pipeline_layout: vk::PipelineLayout,
    pub desc_set_layout: vk::DescriptorSetLayout,
    pub desc_pool:       vk::DescriptorPool,
    pub push_constant_size: u32,
}

impl ComputePipeline {
    /// Create a compute pipeline from SPIR-V bytes.
    /// `binding_count`: number of storage buffer bindings.
    /// `push_constant_size`: bytes of push constants (0 if none).
    pub fn new(
        ld:               &VulkanLogicalDevice,
        spv_bytes:        &[u8],
        binding_count:    u32,
        push_constant_size: u32,
    ) -> GpuResult<Self> {
        // Ensure SPIR-V is u32-aligned
        assert!(spv_bytes.len() % 4 == 0, "SPIR-V must be 4-byte aligned");
        let spv_u32: Vec<u32> = spv_bytes.chunks_exact(4)
            .map(|c| u32::from_ne_bytes([c[0],c[1],c[2],c[3]]))
            .collect();

        // Shader module
        let shader_info = vk::ShaderModuleCreateInfo::default().code(&spv_u32);
        let shader_module = unsafe {
            ld.device.create_shader_module(&shader_info, None)
                .map_err(|e| GpuError::BackendError(format!("shader module: {:?}", e)))?
        };

        // Descriptor set layout — all bindings are storage buffers
        let bindings: Vec<vk::DescriptorSetLayoutBinding> = (0..binding_count)
            .map(|i| vk::DescriptorSetLayoutBinding::default()
                .binding(i)
                .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::COMPUTE))
            .collect();

        let dsl_info = vk::DescriptorSetLayoutCreateInfo::default().bindings(&bindings);
        let desc_set_layout = unsafe {
            ld.device.create_descriptor_set_layout(&dsl_info, None)
                .map_err(|e| GpuError::BackendError(format!("desc set layout: {:?}", e)))?
        };

        // Descriptor pool — one set per pipeline
        let pool_sizes = [vk::DescriptorPoolSize {
            ty: vk::DescriptorType::STORAGE_BUFFER,
            descriptor_count: binding_count,
        }];
        let pool_info = vk::DescriptorPoolCreateInfo::default()
            .pool_sizes(&pool_sizes)
            .max_sets(1);
        let desc_pool = unsafe {
            ld.device.create_descriptor_pool(&pool_info, None)
                .map_err(|e| GpuError::BackendError(format!("desc pool: {:?}", e)))?
        };

        // Pipeline layout
        let set_layouts = [desc_set_layout];
        let push_ranges = if push_constant_size > 0 {
            vec![vk::PushConstantRange {
                stage_flags: vk::ShaderStageFlags::COMPUTE,
                offset: 0,
                size: push_constant_size,
            }]
        } else { vec![] };

        let layout_info = vk::PipelineLayoutCreateInfo::default()
            .set_layouts(&set_layouts)
            .push_constant_ranges(&push_ranges);
        let pipeline_layout = unsafe {
            ld.device.create_pipeline_layout(&layout_info, None)
                .map_err(|e| GpuError::BackendError(format!("pipeline layout: {:?}", e)))?
        };

        // Compute pipeline
        let entry = c"main";
        let stage = vk::PipelineShaderStageCreateInfo::default()
            .stage(vk::ShaderStageFlags::COMPUTE)
            .module(shader_module)
            .name(entry);
        let pipeline_info = vk::ComputePipelineCreateInfo::default()
            .stage(stage)
            .layout(pipeline_layout);

        let pipelines = unsafe {
            ld.device.create_compute_pipelines(
                vk::PipelineCache::null(),
                &[pipeline_info],
                None,
            ).map_err(|(_, e)| GpuError::BackendError(format!("compute pipeline: {:?}", e)))?
        };

        // Shader module no longer needed after pipeline creation
        unsafe { ld.device.destroy_shader_module(shader_module, None); }

        Ok(ComputePipeline {
            pipeline: pipelines[0],
            pipeline_layout,
            desc_set_layout,
            desc_pool,
            push_constant_size,
        })
    }

    /// Allocate a descriptor set and bind storage buffers to it.
    pub fn bind_buffers(
        &self,
        ld:      &VulkanLogicalDevice,
        buffers: &[(vk::Buffer, vk::DeviceSize)], // (buffer, size) pairs
    ) -> GpuResult<vk::DescriptorSet> {
        let layouts = [self.desc_set_layout];
        let alloc_info = vk::DescriptorSetAllocateInfo::default()
            .descriptor_pool(self.desc_pool)
            .set_layouts(&layouts);
        let desc_set = unsafe {
            ld.device.allocate_descriptor_sets(&alloc_info)
                .map_err(|e| GpuError::BackendError(format!("alloc desc set: {:?}", e)))?[0]
        };

        let buffer_infos: Vec<vk::DescriptorBufferInfo> = buffers.iter()
            .map(|&(buf, size)| vk::DescriptorBufferInfo {
                buffer: buf, offset: 0, range: size,
            })
            .collect();

        let writes: Vec<vk::WriteDescriptorSet> = buffer_infos.iter().enumerate()
            .map(|(i, info)| vk::WriteDescriptorSet::default()
                .dst_set(desc_set)
                .dst_binding(i as u32)
                .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
                .buffer_info(std::slice::from_ref(info)))
            .collect();

        unsafe { ld.device.update_descriptor_sets(&writes, &[]); }
        Ok(desc_set)
    }

    pub fn destroy(&self, ld: &VulkanLogicalDevice) {
        unsafe {
            ld.device.destroy_pipeline(self.pipeline, None);
            ld.device.destroy_pipeline_layout(self.pipeline_layout, None);
            ld.device.destroy_descriptor_pool(self.desc_pool, None);
            ld.device.destroy_descriptor_set_layout(self.desc_set_layout, None);
        }
    }
}
