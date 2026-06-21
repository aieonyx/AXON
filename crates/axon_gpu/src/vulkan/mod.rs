// Copyright (c) 2026 Edison Lepitel / AIEONYX
// SPDX-License-Identifier: Apache-2.0
// vulkan/mod.rs -- Vulkan backend modules.
// M1: instance + physical. M2: logical + buffers. M3: pipelines + compute.

pub mod compute;
pub mod instance;
pub mod logical;
pub mod memory;
pub mod physical;
pub mod pipeline;
pub mod vkbuffer;

pub use compute::dispatch_vulkan;
pub use instance::VulkanInstance;
pub use logical::VulkanLogicalDevice;
pub use physical::VulkanPhysicalDevice;
pub use vkbuffer::VkGpuBuffer;
