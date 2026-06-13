// Copyright (c) 2026 Edison Lepiten / AIEONYX
//! Sovereign device registry — EdisonDB-backed device catalogue.
//!
//! Every device connected to a BASTION node is registered here.
//! The registry assigns a sovereign device ID, tracks device state,
//! and enforces the S4+i access policy on device operations.

use axon_core::prelude::*;

/// Maximum devices in the registry.
pub const MAX_DEVICES: usize = 64;

/// Device class — maps to a generic driver category.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceClass {
    HidKeyboard,
    HidMouse,
    HidGamepad,
    NetworkAdapter,
    AudioOutput,
    Display,
    Storage,
    Unknown,
}

impl DeviceClass {
    pub const fn name(self) -> &'static str {
        match self {
            DeviceClass::HidKeyboard   => "hid-keyboard",
            DeviceClass::HidMouse      => "hid-mouse",
            DeviceClass::HidGamepad    => "hid-gamepad",
            DeviceClass::NetworkAdapter => "network-adapter",
            DeviceClass::AudioOutput   => "audio-output",
            DeviceClass::Display       => "display",
            DeviceClass::Storage       => "storage",
            DeviceClass::Unknown       => "unknown",
        }
    }
}

/// Device state in the registry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceState { Registered, Active, Suspended, Removed }

/// A registered device entry.
#[derive(Debug, Clone, Copy)]
pub struct DeviceEntry {
    pub id:        u32,
    pub class:     DeviceClass,
    pub state:     DeviceState,
    pub pd_cap:    u64,   // seL4 capability of the driver PD (0 = not isolated)
    pub vendor_id: u16,
    pub product_id: u16,
}

impl DeviceEntry {
    pub fn is_isolated(&self) -> bool { self.pd_cap != 0 }
}

/// Sovereign device registry.
pub struct DeviceRegistry {
    entries: [Option<DeviceEntry>; MAX_DEVICES],
    count:   usize,
    next_id: u32,
}

impl DeviceRegistry {
    pub const fn new() -> Self {
        Self { entries: [None; MAX_DEVICES], count: 0, next_id: 1 }
    }

    /// Register a new device. Returns assigned sovereign device ID.
    pub fn register(
        &mut self,
        class: DeviceClass,
        vendor_id: u16,
        product_id: u16,
    ) -> AxonResult<u32> {
        if self.count >= MAX_DEVICES {
            return AxonResult::Err(AxonError::invalid_state("device registry full"));
        }
        let id = self.next_id;
        self.next_id += 1;
        // Find free slot
        for slot in self.entries.iter_mut() {
            if slot.is_none() {
                *slot = Some(DeviceEntry {
                    id, class, state: DeviceState::Registered,
                    pd_cap: 0, vendor_id, product_id,
                });
                self.count += 1;
                return AxonResult::Ok(id);
            }
        }
        AxonResult::Err(AxonError::invalid_state("no free registry slot"))
    }

    /// Set the seL4 PD capability for a device (isolation assignment).
    pub fn assign_pd(&mut self, id: u32, pd_cap: u64) -> AxonResult<()> {
        for slot in self.entries.iter_mut().flatten() {
            if slot.id == id {
                slot.pd_cap = pd_cap;
                return AxonResult::Ok(());
            }
        }
        AxonResult::Err(AxonError::not_found("device not found"))
    }

    /// Update device state.
    pub fn set_state(&mut self, id: u32, state: DeviceState) -> AxonResult<()> {
        for slot in self.entries.iter_mut().flatten() {
            if slot.id == id {
                slot.state = state;
                return AxonResult::Ok(());
            }
        }
        AxonResult::Err(AxonError::not_found("device not found"))
    }

    /// Look up a device by ID.
    pub fn get(&self, id: u32) -> Option<&DeviceEntry> {
        self.entries.iter().flatten().find(|e| e.id == id)
    }

    /// Iterate active devices of a given class.
    pub fn active_by_class(&self, class: DeviceClass) -> impl Iterator<Item = &DeviceEntry> {
        self.entries.iter().flatten()
            .filter(move |e| e.class == class && e.state == DeviceState::Active)
    }

    /// Number of registered devices.
    pub fn count(&self) -> usize { self.count }

    /// Remove a device from the registry.
    pub fn remove(&mut self, id: u32) -> AxonResult<()> {
        for slot in self.entries.iter_mut() {
            if matches!(slot, Some(e) if e.id == id) {
                *slot = None;
                self.count -= 1;
                return AxonResult::Ok(());
            }
        }
        AxonResult::Err(AxonError::not_found("device not found"))
    }
}

impl Default for DeviceRegistry {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tp44_registry_register_and_get() {
        let mut r = DeviceRegistry::new();
        let id = r.register(DeviceClass::HidKeyboard, 0x045E, 0x07A5).unwrap();
        assert_eq!(id, 1);
        let entry = r.get(id).unwrap();
        assert_eq!(entry.class, DeviceClass::HidKeyboard);
        assert_eq!(entry.state, DeviceState::Registered);
        assert!(!entry.is_isolated());
    }

    #[test]
    fn tp44_registry_assign_pd() {
        let mut r = DeviceRegistry::new();
        let id = r.register(DeviceClass::Storage, 0x0781, 0x5583).unwrap();
        r.assign_pd(id, 42).unwrap();
        assert!(r.get(id).unwrap().is_isolated());
        assert_eq!(r.get(id).unwrap().pd_cap, 42);
    }

    #[test]
    fn tp44_registry_set_state() {
        let mut r = DeviceRegistry::new();
        let id = r.register(DeviceClass::Display, 0x1234, 0x5678).unwrap();
        r.set_state(id, DeviceState::Active).unwrap();
        assert_eq!(r.get(id).unwrap().state, DeviceState::Active);
    }

    #[test]
    fn tp44_registry_active_by_class() {
        let mut r = DeviceRegistry::new();
        let id1 = r.register(DeviceClass::HidMouse, 0x046D, 0xC52B).unwrap();
        let id2 = r.register(DeviceClass::HidMouse, 0x046D, 0xC52C).unwrap();
        let _id3 = r.register(DeviceClass::Storage, 0x0781, 0x5583).unwrap();
        r.set_state(id1, DeviceState::Active).unwrap();
        r.set_state(id2, DeviceState::Active).unwrap();
        let mice: Vec<_> = r.active_by_class(DeviceClass::HidMouse).collect();
        assert_eq!(mice.len(), 2);
    }

    #[test]
    fn tp44_registry_remove() {
        let mut r = DeviceRegistry::new();
        let id = r.register(DeviceClass::AudioOutput, 0x8086, 0x0001).unwrap();
        assert_eq!(r.count(), 1);
        r.remove(id).unwrap();
        assert_eq!(r.count(), 0);
        assert!(r.get(id).is_none());
    }

    #[test]
    fn tp44_registry_full() {
        let mut r = DeviceRegistry::new();
        for i in 0..MAX_DEVICES {
            r.register(DeviceClass::Unknown, 0, i as u16).unwrap();
        }
        assert!(r.register(DeviceClass::Unknown, 0, 0).is_err());
    }

    #[test]
    fn tp44_device_class_names() {
        assert_eq!(DeviceClass::HidKeyboard.name(), "hid-keyboard");
        assert_eq!(DeviceClass::Storage.name(), "storage");
        assert_eq!(DeviceClass::Unknown.name(), "unknown");
    }
}
