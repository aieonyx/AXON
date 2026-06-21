// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
// vulkan/instance.rs -- VkInstance creation and lifecycle.

use ash::{vk, Entry, Instance};
use crate::error::{GpuError, GpuResult};

pub struct VulkanInstance {
    pub entry:    Entry,
    pub instance: Instance,
}

impl VulkanInstance {
    /// Create a Vulkan instance for compute (no display surface required).
    /// Returns Err if Vulkan loader or driver is unavailable.
    pub fn new() -> GpuResult<Self> {
        // Load Vulkan entry point
        let entry = unsafe {
            Entry::load().map_err(|e| {
                GpuError::BackendError(format!("Vulkan loader not found: {}", e))
            })?
        };

        // Application info — compute-only, no presentation
        let app_name    = c"AIEONYX Sovereign Compute";
        let engine_name = c"AXON GPU";

        let app_info = vk::ApplicationInfo::default()
            .application_name(app_name)
            .application_version(vk::make_api_version(0, 0, 58, 1))
            .engine_name(engine_name)
            .engine_version(vk::make_api_version(0, 0, 58, 1))
            .api_version(vk::API_VERSION_1_2);

        let create_info = vk::InstanceCreateInfo::default()
            .application_info(&app_info);

        let instance = unsafe {
            entry.create_instance(&create_info, None).map_err(|e| {
                GpuError::BackendError(format!("VkInstance creation failed: {:?}", e))
            })?
        };

        Ok(VulkanInstance { entry, instance })
    }
}

impl Drop for VulkanInstance {
    fn drop(&mut self) {
        unsafe { self.instance.destroy_instance(None); }
    }
}
