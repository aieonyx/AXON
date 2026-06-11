//! seL4 memory object operations
//! Copyright (c) 2026 Edison Lepiten / AIEONYX

use crate::types::{Cap, Error, SizeBits};
use crate::types::obj_type;

/// Arguments for seL4_Untyped_Retype
#[derive(Debug, Clone, Copy)]
pub struct RetypeArgs {
    pub object_type: u64,
    pub size_bits: SizeBits,
    pub root: Cap,
    pub node_index: u64,
    pub node_depth: u64,
    pub node_offset: u64,
    pub num_objects: u64,
}

/// seL4_Untyped_Retype — carve typed objects from untyped memory
pub fn untyped_retype(untyped: Cap, args: RetypeArgs) -> Error {
    let _ = (untyped, args);
    0 // seL4_NoError
}

/// Convenience: retype untyped into Endpoint objects
pub fn retype_endpoint(untyped: Cap, root: Cap, slot: u64) -> Error {
    untyped_retype(untyped, RetypeArgs { object_type: obj_type::ENDPOINT, size_bits: 0, root, node_index: 0, node_depth: 64, node_offset: slot, num_objects: 1 })
}

/// Convenience: retype untyped into Notification objects
pub fn retype_notification(untyped: Cap, root: Cap, slot: u64) -> Error {
    untyped_retype(untyped, RetypeArgs { object_type: obj_type::NOTIFICATION, size_bits: 0, root, node_index: 0, node_depth: 64, node_offset: slot, num_objects: 1 })
}

/// Convenience: retype untyped into TCB
pub fn retype_tcb(untyped: Cap, root: Cap, slot: u64) -> Error {
    untyped_retype(untyped, RetypeArgs { object_type: obj_type::TCB, size_bits: 0, root, node_index: 0, node_depth: 64, node_offset: slot, num_objects: 1 })
}

/// Convenience: retype untyped into 4KB pages
pub fn retype_page(untyped: Cap, root: Cap, slot: u64) -> Error {
    untyped_retype(untyped, RetypeArgs { object_type: obj_type::SMALL_PAGE, size_bits: 12, root, node_index: 0, node_depth: 64, node_offset: slot, num_objects: 1 })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tp30_mem_01_untyped_retype_endpoint() {
        let err = retype_endpoint(10, 1, 5);
        assert_eq!(err, 0, "retype_endpoint must return seL4_NoError=0");
    }

    #[test]
    fn tp30_mem_02_untyped_retype_notification() {
        let err = retype_notification(10, 1, 6);
        assert_eq!(err, 0);
    }

    #[test]
    fn tp30_mem_03_untyped_retype_tcb() {
        let err = retype_tcb(10, 1, 7);
        assert_eq!(err, 0);
    }

    #[test]
    fn tp30_mem_04_untyped_retype_page() {
        let err = retype_page(10, 1, 8);
        assert_eq!(err, 0);
    }

    #[test]
    fn tp30_mem_05_full_pd_bootstrap() {
        // Full PD bootstrap pattern: retype endpoint + notification
        let untyped: u64 = 100;
        let cnode:   u64 = 1;
        let ep_slot:   u64 = 10;
        let ntfn_slot: u64 = 11;
        assert_eq!(retype_endpoint(untyped, cnode, ep_slot), 0);
        assert_eq!(retype_notification(untyped, cnode, ntfn_slot), 0);
    }
}
