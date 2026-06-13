// Copyright (c) 2026 Edison Lepiten / AIEONYX
//! axon_drivers — Phoenix generic + sovereign driver stack.
//!
//! generic/   — hardware-class drivers (USB HID, CDC-ECM, HDA, VESA/GOP, Mass Storage)
//! sovereign/ — per-driver seL4 PD isolation, EdisonDB registry, AWP discovery, plugin API
#![allow(missing_docs)]
extern crate alloc;

pub mod generic;
pub mod sovereign;
