// Copyright (c) 2026 Edison Lepiten / AIEONYX
//! Vendor driver plug-in interface.
//!
//! Third-party drivers implement the DriverPlugin trait and register
//! with the sovereign driver stack. The stack enforces:
//!   - PD isolation before plugin activation
//!   - Registry entry before any device access
//!   - Capability scope enforcement via CapabilityBroker

use axon_core::prelude::*;
use super::registry::{DeviceClass, DeviceRegistry};
use super::pd_isolation::{DriverPd, PdIsolationManager};

/// Driver plug-in metadata.
#[derive(Debug, Clone, Copy)]
pub struct PluginInfo {
    pub name:       &'static str,
    pub version:    u32,
    pub class:      DeviceClass,
    pub vendor_id:  u16,
    pub product_id: u16,
}

/// Vendor driver plug-in interface.
pub trait DriverPlugin: Send {
    fn info(&self) -> PluginInfo;
    /// Called when the device is detected and PD is ready.
    fn on_attach(&mut self, device_id: u32) -> AxonResult<()>;
    /// Called when the device is removed or PD is stopped.
    fn on_detach(&mut self, device_id: u32) -> AxonResult<()>;
    /// Called periodically to poll the device.
    fn on_poll(&mut self, device_id: u32) -> AxonResult<()>;
}

/// Sovereign driver stack — orchestrates registry, isolation, and plugins.
pub struct SovereignDriverStack {
    registry:  DeviceRegistry,
    isolation: PdIsolationManager,
    /// Maps device_id → pd_idx for cleanup on detach.
    pd_map: [(u32, usize); 32],
    pd_map_count: usize,
}

impl SovereignDriverStack {
    pub const fn new() -> Self {
        Self {
            registry:  DeviceRegistry::new(),
            isolation: PdIsolationManager::new(),
            pd_map: [(0, 0); 32],
            pd_map_count: 0,
        }
    }

    /// Attach a driver plugin — register device, start PD, call on_attach.
    pub fn attach(
        &mut self,
        plugin: &mut dyn DriverPlugin,
        pd: DriverPd,
    ) -> AxonResult<u32> {
        let info = plugin.info();
        // Register device
        let device_id = match self.registry.register(info.class, info.vendor_id, info.product_id) {
            AxonResult::Ok(id) => id,
            AxonResult::Err(e) => return AxonResult::Err(e),
        };
        let pd_idx = match self.isolation.register(pd) {
            AxonResult::Ok(i) => i,
            AxonResult::Err(e) => {
                let _ = self.registry.remove(device_id); // rollback
                return AxonResult::Err(e);
            }
        };
        if let AxonResult::Err(e) = self.isolation.start(pd_idx) {
            let _ = self.registry.remove(device_id);
            let _ = self.isolation.remove(pd_idx);
            return AxonResult::Err(e);
        }
        let pd_cap = self.isolation.get(pd_idx).map(|p| p.tcb_cap).unwrap_or(0);
        if let AxonResult::Err(e) = self.registry.assign_pd(device_id, pd_cap) {
            let _ = self.registry.remove(device_id);
            let _ = self.isolation.stop(pd_idx);
            let _ = self.isolation.remove(pd_idx);
            return AxonResult::Err(e);
        }
        if let AxonResult::Err(e) = plugin.on_attach(device_id) {
            let _ = self.registry.remove(device_id);
            let _ = self.isolation.stop(pd_idx);
            let _ = self.isolation.remove(pd_idx);
            return AxonResult::Err(e);
        }
        // Store device_id → pd_idx mapping for detach cleanup
        if self.pd_map_count < 32 {
            self.pd_map[self.pd_map_count] = (device_id, pd_idx);
            self.pd_map_count += 1;
        }
        AxonResult::Ok(device_id)
    }

    /// Detach a driver plugin — call on_detach, stop PD, remove registry entry.
    pub fn detach(
        &mut self,
        plugin: &mut dyn DriverPlugin,
        device_id: u32,
    ) -> AxonResult<()> {
        if let AxonResult::Err(e) = plugin.on_detach(device_id) { return AxonResult::Err(e); }
        if let AxonResult::Err(e) = self.registry.remove(device_id) { return AxonResult::Err(e); }
        // Stop and remove the PD to free the isolation slot
        for i in 0..self.pd_map_count {
            if self.pd_map[i].0 == device_id {
                let pd_idx = self.pd_map[i].1;
                let _ = self.isolation.stop(pd_idx);
                let _ = self.isolation.remove(pd_idx);
                self.pd_map[i] = self.pd_map[self.pd_map_count - 1];
                self.pd_map_count -= 1;
                break;
            }
        }
        AxonResult::Ok(())
    }

    pub fn registry(&self) -> &DeviceRegistry { &self.registry }
}

impl Default for SovereignDriverStack {
    fn default() -> Self { Self::new() }
}


#[cfg(test)]
mod tests {
    use super::*;
    use super::super::registry::DeviceClass;
    use super::super::pd_isolation::DriverPd;

    struct TestPlugin { attached: bool }

    impl DriverPlugin for TestPlugin {
        fn info(&self) -> PluginInfo {
            PluginInfo { name: "test", version: 1, class: DeviceClass::HidKeyboard,
                         vendor_id: 0x045E, product_id: 0x07A5 }
        }
        fn on_attach(&mut self, _id: u32) -> AxonResult<()> {
            self.attached = true; AxonResult::Ok(())
        }
        fn on_detach(&mut self, _id: u32) -> AxonResult<()> {
            self.attached = false; AxonResult::Ok(())
        }
        fn on_poll(&mut self, _id: u32) -> AxonResult<()> { AxonResult::Ok(()) }
    }

    #[test]
    fn tp44_plugin_attach_detach() {
        let mut stack = SovereignDriverStack::new();
        let mut plugin = TestPlugin { attached: false };
        let pd = DriverPd::new(10, 11, 0x9000_0000, 0x1000, 0);
        let id = stack.attach(&mut plugin, pd).unwrap();
        assert!(plugin.attached);
        assert_eq!(stack.registry().count(), 1);
        stack.detach(&mut plugin, id).unwrap();
        assert!(!plugin.attached);
        assert_eq!(stack.registry().count(), 0);
    }

    #[test]
    fn tp44_plugin_info() {
        let p = TestPlugin { attached: false };
        assert_eq!(p.info().name, "test");
        assert_eq!(p.info().class, DeviceClass::HidKeyboard);
    }
}
