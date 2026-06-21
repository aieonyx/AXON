// Copyright (c) 2026 Edison Lepitel / AIEONYX
// SPDX-License-Identifier: Apache-2.0
// vulkan/compute.rs -- Sovereign GPU kernel dispatch via Vulkan compute.
// P58.1 M3: SPIR-V shader dispatch for Add, Mul, Scale, ReLU, MatMul.

use ash::vk;
use crate::error::{GpuError, GpuResult};
use crate::kernel::KernelOp;
use super::logical::VulkanLogicalDevice;
use super::physical::VulkanPhysicalDevice;
use super::pipeline::ComputePipeline;
use super::vkbuffer::VkGpuBuffer;

// Pre-compiled SPIR-V shaders (compiled from GLSL by build.rs via naga)
static SPV_ADD:    &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/add.spv"));
static SPV_MUL:    &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/mul.spv"));
static SPV_SCALE:  &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/scale.spv"));
static SPV_RELU:   &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/relu.spv"));
static SPV_MATMUL: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/matmul.spv"));

/// Dispatch a kernel on Vulkan compute.
/// `inputs`: device-local buffers with data already uploaded.
/// `output`: device-local buffer to receive results.
pub fn dispatch_vulkan(
    op:     &KernelOp,
    ld:     &VulkanLogicalDevice,
    vpd:    &VulkanPhysicalDevice,
    inputs: &[&VkGpuBuffer],
    output: &VkGpuBuffer,
) -> GpuResult<()> {
    match op {
        KernelOp::Add => {
            if inputs.len() < 2 { return Err(GpuError::KernelFailed("Add needs 2 inputs".into())); }
            let n = (inputs[0].size / 4) as u32;
            let pipeline = ComputePipeline::new(ld, SPV_ADD, 3, 4)?;
            run_binary(ld, &pipeline, inputs[0], inputs[1], output, n, &[n])?;
            pipeline.destroy(ld);
        }
        KernelOp::Mul => {
            if inputs.len() < 2 { return Err(GpuError::KernelFailed("Mul needs 2 inputs".into())); }
            let n = (inputs[0].size / 4) as u32;
            let pipeline = ComputePipeline::new(ld, SPV_MUL, 3, 4)?;
            run_binary(ld, &pipeline, inputs[0], inputs[1], output, n, &[n])?;
            pipeline.destroy(ld);
        }
        KernelOp::Scale(s) => {
            if inputs.is_empty() { return Err(GpuError::KernelFailed("Scale needs 1 input".into())); }
            let n = (inputs[0].size / 4) as u32;
            let pipeline = ComputePipeline::new(ld, SPV_SCALE, 2, 8)?;
            run_unary_scale(ld, &pipeline, inputs[0], output, n, *s)?;
            pipeline.destroy(ld);
        }
        KernelOp::ReLU => {
            if inputs.is_empty() { return Err(GpuError::KernelFailed("ReLU needs 1 input".into())); }
            let n = (inputs[0].size / 4) as u32;
            let pipeline = ComputePipeline::new(ld, SPV_RELU, 2, 4)?;
            run_unary(ld, &pipeline, inputs[0], output, n, &[n])?;
            pipeline.destroy(ld);
        }
        KernelOp::MatMul { rows, cols, inner } => {
            if inputs.len() < 2 { return Err(GpuError::KernelFailed("MatMul needs 2 inputs".into())); }
            let pipeline = ComputePipeline::new(ld, SPV_MATMUL, 3, 12)?;
            run_matmul(ld, &pipeline, inputs[0], inputs[1], output, *rows, *cols, *inner)?;
            pipeline.destroy(ld);
        }
        _ => return Err(GpuError::UnsupportedOp(format!("{:?} not yet on Vulkan", op))),
    }
    Ok(())
}

// ── Dispatch helpers ──────────────────────────────────────────────────────────

fn run_binary(
    ld: &VulkanLogicalDevice,
    pipeline: &ComputePipeline,
    a: &VkGpuBuffer, b: &VkGpuBuffer, out: &VkGpuBuffer,
    n: u32,
    push: &[u32],
) -> GpuResult<()> {
    let bufs = [(a.buffer, a.size), (b.buffer, b.size), (out.buffer, out.size)];
    let desc = pipeline.bind_buffers(ld, &bufs)?;
    let push_bytes = push_to_bytes(push);
    ld.one_shot(|cmd| {
        unsafe {
            ld.device.cmd_bind_pipeline(cmd, vk::PipelineBindPoint::COMPUTE, pipeline.pipeline);
            ld.device.cmd_bind_descriptor_sets(cmd, vk::PipelineBindPoint::COMPUTE,
                pipeline.pipeline_layout, 0, &[desc], &[]);
            if !push_bytes.is_empty() {
                ld.device.cmd_push_constants(cmd, pipeline.pipeline_layout,
                    vk::ShaderStageFlags::COMPUTE, 0, &push_bytes);
            }
            ld.device.cmd_dispatch(cmd, (n + 63) / 64, 1, 1);
        }
        Ok(())
    })
}

fn run_unary(
    ld: &VulkanLogicalDevice,
    pipeline: &ComputePipeline,
    a: &VkGpuBuffer, out: &VkGpuBuffer,
    n: u32,
    push: &[u32],
) -> GpuResult<()> {
    let bufs = [(a.buffer, a.size), (out.buffer, out.size)];
    let desc = pipeline.bind_buffers(ld, &bufs)?;
    let push_bytes = push_to_bytes(push);
    ld.one_shot(|cmd| {
        unsafe {
            ld.device.cmd_bind_pipeline(cmd, vk::PipelineBindPoint::COMPUTE, pipeline.pipeline);
            ld.device.cmd_bind_descriptor_sets(cmd, vk::PipelineBindPoint::COMPUTE,
                pipeline.pipeline_layout, 0, &[desc], &[]);
            if !push_bytes.is_empty() {
                ld.device.cmd_push_constants(cmd, pipeline.pipeline_layout,
                    vk::ShaderStageFlags::COMPUTE, 0, &push_bytes);
            }
            ld.device.cmd_dispatch(cmd, (n + 63) / 64, 1, 1);
        }
        Ok(())
    })
}

fn run_unary_scale(
    ld: &VulkanLogicalDevice,
    pipeline: &ComputePipeline,
    a: &VkGpuBuffer, out: &VkGpuBuffer,
    n: u32, scale: f32,
) -> GpuResult<()> {
    let bufs = [(a.buffer, a.size), (out.buffer, out.size)];
    let desc = pipeline.bind_buffers(ld, &bufs)?;
    // Push: [n: u32, scale: f32] = 8 bytes
    let mut push_bytes = [0u8; 8];
    push_bytes[..4].copy_from_slice(&n.to_ne_bytes());
    push_bytes[4..].copy_from_slice(&scale.to_ne_bytes());
    ld.one_shot(|cmd| {
        unsafe {
            ld.device.cmd_bind_pipeline(cmd, vk::PipelineBindPoint::COMPUTE, pipeline.pipeline);
            ld.device.cmd_bind_descriptor_sets(cmd, vk::PipelineBindPoint::COMPUTE,
                pipeline.pipeline_layout, 0, &[desc], &[]);
            ld.device.cmd_push_constants(cmd, pipeline.pipeline_layout,
                vk::ShaderStageFlags::COMPUTE, 0, &push_bytes);
            ld.device.cmd_dispatch(cmd, (n + 63) / 64, 1, 1);
        }
        Ok(())
    })
}

fn run_matmul(
    ld: &VulkanLogicalDevice,
    pipeline: &ComputePipeline,
    a: &VkGpuBuffer, b: &VkGpuBuffer, out: &VkGpuBuffer,
    rows: usize, cols: usize, inner: usize,
) -> GpuResult<()> {
    let bufs = [(a.buffer, a.size), (b.buffer, b.size), (out.buffer, out.size)];
    let desc = pipeline.bind_buffers(ld, &bufs)?;
    let push = [rows as u32, cols as u32, inner as u32];
    let push_bytes = push_to_bytes(&push);
    ld.one_shot(|cmd| {
        unsafe {
            ld.device.cmd_bind_pipeline(cmd, vk::PipelineBindPoint::COMPUTE, pipeline.pipeline);
            ld.device.cmd_bind_descriptor_sets(cmd, vk::PipelineBindPoint::COMPUTE,
                pipeline.pipeline_layout, 0, &[desc], &[]);
            ld.device.cmd_push_constants(cmd, pipeline.pipeline_layout,
                vk::ShaderStageFlags::COMPUTE, 0, &push_bytes);
            // MatMul uses 8x8 local size
            ld.device.cmd_dispatch(cmd,
                (rows as u32 + 7) / 8,
                (cols as u32 + 7) / 8,
                1);
        }
        Ok(())
    })
}

fn push_to_bytes(vals: &[u32]) -> Vec<u8> {
    vals.iter().flat_map(|v| v.to_ne_bytes()).collect()
}
