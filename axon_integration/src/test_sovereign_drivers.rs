// Copyright (c) 2026 Edison Lepiten / AIEONYX
//! Phase 44 integration tests — sovereign driver stack.

use axon_drivers::sovereign::{
    DeviceRegistry, DeviceClass, DeviceState,
    PdIsolationManager, DriverPd,
    DiscoveryTable, DeviceAnnouncement,
    SovereignDriverStack, DriverPlugin, PluginInfo,
};

#[test]
fn p44_registry_full_lifecycle() {
    let mut r = DeviceRegistry::new();
    let id = r.register(DeviceClass::HidKeyboard, 0x045E, 0x07A5).unwrap();
    r.set_state(id, DeviceState::Active).unwrap();
    r.assign_pd(id, 99).unwrap();
    let e = r.get(id).unwrap();
    assert_eq!(e.state, DeviceState::Active);
    assert!(e.is_isolated());
    r.remove(id).unwrap();
    assert_eq!(r.count(), 0);
}

#[test]
fn p44_pd_isolation_start_stop() {
    let mut mgr = PdIsolationManager::new();
    let pd = DriverPd::new(10, 11, 0x9000_0000, 0x1000, 32);
    let idx = mgr.register(pd).unwrap();
    mgr.start(idx).unwrap();
    assert!(mgr.get(idx).unwrap().started);
    mgr.stop(idx).unwrap();
    assert!(!mgr.get(idx).unwrap().started);
}

#[test]
fn p44_discovery_cross_node() {
    let mut t = DiscoveryTable::new();
    t.record(DeviceAnnouncement {
        node_id: [0xAA; 8], device_id: 1,
        class: DeviceClass::Storage,
        vendor_id: 0x0781, product_id: 0x5583,
        uri_hash: [0u8; 8],
    }).unwrap();
    t.record(DeviceAnnouncement {
        node_id: [0xBB; 8], device_id: 1,
        class: DeviceClass::NetworkAdapter,
        vendor_id: 0x0B95, product_id: 0x7720,
        uri_hash: [0u8; 8],
    }).unwrap();
    assert_eq!(t.count(), 2);
    assert_eq!(t.find_by_class(DeviceClass::Storage).count(), 1);
}

#[test]
fn p44_sovereign_stack_plugin_lifecycle() {
    struct MockPlugin { pub calls: u32 }
    impl DriverPlugin for MockPlugin {
        fn info(&self) -> PluginInfo {
            PluginInfo { name: "mock", version: 1,
                class: DeviceClass::AudioOutput, vendor_id: 0x8086, product_id: 0x0001 }
        }
        fn on_attach(&mut self, _: u32) -> axon_core::prelude::AxonResult<()> {
            self.calls += 1; axon_core::prelude::AxonResult::Ok(())
        }
        fn on_detach(&mut self, _: u32) -> axon_core::prelude::AxonResult<()> {
            self.calls += 1; axon_core::prelude::AxonResult::Ok(())
        }
        fn on_poll(&mut self, _: u32) -> axon_core::prelude::AxonResult<()> {
            axon_core::prelude::AxonResult::Ok(())
        }
    }

    let mut stack = SovereignDriverStack::new();
    let mut plugin = MockPlugin { calls: 0 };
    let pd = DriverPd::new(20, 21, 0, 0, 0);
    let id = stack.attach(&mut plugin, pd).unwrap();
    assert_eq!(plugin.calls, 1);
    stack.detach(&mut plugin, id).unwrap();
    assert_eq!(plugin.calls, 2);
}
