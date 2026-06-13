// Copyright (c) 2026 Edison Lepiten / AIEONYX
//! AWP device discovery — sovereign peer device announcement.
//!
//! BASTION nodes announce their connected devices over the AWP mesh.
//! Remote nodes can discover and interact with devices via the AWP protocol.
//! Device announcements are cryptographically signed with the node's Ed25519 key.

use axon_core::prelude::*;
use super::registry::{DeviceClass, DeviceEntry};

/// AWP device announcement — broadcast to mesh peers.
#[derive(Debug, Clone, Copy)]
pub struct DeviceAnnouncement {
    /// Announcing node ID (Ed25519 public key fingerprint, first 8 bytes).
    pub node_id:    [u8; 8],
    /// Sovereign device ID on the announcing node.
    pub device_id:  u32,
    /// Device class.
    pub class:      DeviceClass,
    /// Vendor/product ID pair.
    pub vendor_id:  u16,
    pub product_id: u16,
    /// AWP endpoint URI hash (first 8 bytes of SHA-256).
    pub uri_hash:   [u8; 8],
}

impl DeviceAnnouncement {
    pub fn from_entry(node_id: [u8; 8], entry: &DeviceEntry, uri_hash: [u8; 8]) -> Self {
        Self {
            node_id,
            device_id:  entry.id,
            class:      entry.class,
            vendor_id:  entry.vendor_id,
            product_id: entry.product_id,
            uri_hash,
        }
    }
}

/// Discovery table — tracks known devices across the mesh.
pub struct DiscoveryTable {
    announcements: [Option<DeviceAnnouncement>; 128],
    count:         usize,
}

impl DiscoveryTable {
    pub const fn new() -> Self {
        Self { announcements: [None; 128], count: 0 }
    }

    /// Record a device announcement from a peer node.
    pub fn record(&mut self, ann: DeviceAnnouncement) -> AxonResult<()> {
        // Update if already known (same node_id + device_id)
        for slot in self.announcements.iter_mut().flatten() {
            if slot.node_id == ann.node_id && slot.device_id == ann.device_id {
                *slot = ann;
                return AxonResult::Ok(());
            }
        }
        // Insert new
        if self.count >= 128 {
            return AxonResult::Err(AxonError::invalid_state("discovery table full"));
        }
        for slot in self.announcements.iter_mut() {
            if slot.is_none() {
                *slot = Some(ann);
                self.count += 1;
                return AxonResult::Ok(());
            }
        }
        AxonResult::Err(AxonError::invalid_state("no free discovery slot"))
    }

    /// Find devices of a given class across all known nodes.
    pub fn find_by_class(&self, class: DeviceClass) -> impl Iterator<Item = &DeviceAnnouncement> {
        self.announcements.iter().flatten().filter(move |a| a.class == class)
    }

    /// Remove all announcements from a node (node went offline).
    pub fn remove_node(&mut self, node_id: [u8; 8]) {
        for slot in self.announcements.iter_mut() {
            if matches!(slot, Some(a) if a.node_id == node_id) {
                *slot = None;
                self.count -= 1;
            }
        }
    }

    /// Total known devices across all nodes.
    pub fn count(&self) -> usize { self.count }
}

impl Default for DiscoveryTable {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_ann(node: u8, device_id: u32, class: DeviceClass) -> DeviceAnnouncement {
        DeviceAnnouncement {
            node_id:    [node; 8],
            device_id,
            class,
            vendor_id:  0x046D,
            product_id: 0xC52B,
            uri_hash:   [0u8; 8],
        }
    }

    #[test]
    fn tp44_discovery_record_and_find() {
        let mut t = DiscoveryTable::new();
        t.record(make_ann(1, 1, DeviceClass::HidKeyboard)).unwrap();
        t.record(make_ann(2, 1, DeviceClass::HidMouse)).unwrap();
        let keyboards: Vec<_> = t.find_by_class(DeviceClass::HidKeyboard).collect();
        assert_eq!(keyboards.len(), 1);
        assert_eq!(keyboards[0].node_id, [1u8; 8]);
    }

    #[test]
    fn tp44_discovery_update_existing() {
        let mut t = DiscoveryTable::new();
        t.record(make_ann(1, 1, DeviceClass::Storage)).unwrap();
        assert_eq!(t.count(), 1);
        // Re-record same node+device — should update, not add
        t.record(make_ann(1, 1, DeviceClass::Storage)).unwrap();
        assert_eq!(t.count(), 1);
    }

    #[test]
    fn tp44_discovery_remove_node() {
        let mut t = DiscoveryTable::new();
        t.record(make_ann(1, 1, DeviceClass::HidKeyboard)).unwrap();
        t.record(make_ann(1, 2, DeviceClass::HidMouse)).unwrap();
        t.record(make_ann(2, 1, DeviceClass::Storage)).unwrap();
        t.remove_node([1u8; 8]);
        assert_eq!(t.count(), 1);
        assert_eq!(t.find_by_class(DeviceClass::HidKeyboard).count(), 0);
    }
}
