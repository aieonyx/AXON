// Copyright (c) 2026 Edison Lepiten / AIEONYX
//! Phoenix generic drivers — hardware-class drivers for BASTION nodes.

pub mod audio;
pub mod display;
pub mod hid;
pub mod net;
pub mod storage;

pub use audio::{AudioConfig, AudioDriver, SampleFormat, StubAudio};
pub use display::{Colour, DisplayDriver, DisplayMode, PixelFormat, StubDisplay};
pub use hid::{HidDeviceType, HidDriver, HidReport, KeyCode, KeyEvent, MouseEvent, StubHidDriver,
              parse_keyboard_report, parse_mouse_report};
pub use net::{CdcEcmDriver, EtherFrame, MacAddr, StubCdcEcm};
pub use storage::{StorageDriver, StorageInfo, StubStorage, BLOCK_SIZE};
