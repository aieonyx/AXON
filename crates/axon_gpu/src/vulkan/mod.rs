// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
// vulkan/mod.rs -- Vulkan backend for axon_gpu.
// P58.1 M1: instance creation, physical device enumeration, logical device.
// Safe wrapper over ash — all unsafe blocks are minimal and justified.

pub mod instance;
pub mod physical;

pub use instance::VulkanInstance;
pub use physical::VulkanPhysicalDevice;
