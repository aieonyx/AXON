// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
// vulkan/memory.rs -- Memory type selection helper.

use ash::vk;
use crate::error::{GpuError, GpuResult};

pub fn find_memory_type(
    mem_props:   &vk::PhysicalDeviceMemoryProperties,
    type_filter: u32,
    properties:  vk::MemoryPropertyFlags,
) -> GpuResult<u32> {
    for i in 0..mem_props.memory_type_count {
        let type_ok = (type_filter & (1 << i)) != 0;
        let prop_ok = mem_props.memory_types[i as usize].property_flags.contains(properties);
        if type_ok && prop_ok { return Ok(i); }
    }
    Err(GpuError::BackendError(format!(
        "no memory type for filter={:#010x} props={:?}", type_filter, properties
    )))
}
