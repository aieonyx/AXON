// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
// vulkan/mod.rs -- Vulkan backend modules.
// M1: instance + physical device. M2: logical device + buffer management.

pub mod instance;
pub mod logical;
pub mod memory;
pub mod physical;
pub mod vkbuffer;

pub use instance::VulkanInstance;
pub use logical::VulkanLogicalDevice;
pub use physical::VulkanPhysicalDevice;
pub use vkbuffer::VkGpuBuffer;
