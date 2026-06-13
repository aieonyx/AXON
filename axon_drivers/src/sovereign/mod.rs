// Copyright (c) 2026 Edison Lepiten / AIEONYX
//! Sovereign driver stack — seL4 PD isolation, device registry, AWP discovery.

pub mod discovery;
pub mod pd_isolation;
pub mod plugin;
pub mod registry;

pub use discovery::{DeviceAnnouncement, DiscoveryTable};
pub use pd_isolation::{DriverPd, PdIsolationManager};
pub use plugin::{DriverPlugin, PluginInfo, SovereignDriverStack};
pub use registry::{DeviceClass, DeviceEntry, DeviceRegistry, DeviceState};
