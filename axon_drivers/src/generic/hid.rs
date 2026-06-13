// Copyright (c) 2026 Edison Lepiten / AIEONYX
//! USB HID driver — keyboard, mouse, gamepad input devices.
//!
//! Implements the USB HID class protocol (USB spec §9, HID spec §6).
//! On host: ring-buffer backed stub for testing.
//! On aarch64-seL4: wired to USB controller via MMIO PAL (P44).

use axon_core::prelude::*;

/// HID device type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HidDeviceType { Keyboard, Mouse, Gamepad }

/// A single HID input report — raw bytes from the device.
#[derive(Debug, Clone, Copy)]
pub struct HidReport {
    pub device: HidDeviceType,
    pub data:   [u8; 8],
    pub len:    u8,
}

impl HidReport {
    pub const fn empty(device: HidDeviceType) -> Self {
        Self { device, data: [0u8; 8], len: 0 }
    }
}

/// Keyboard key codes (HID Usage Table §10).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KeyCode(pub u8);

impl KeyCode {
    pub const NONE:  KeyCode = KeyCode(0x00);
    pub const A:     KeyCode = KeyCode(0x04);
    pub const ENTER: KeyCode = KeyCode(0x28);
    pub const SPACE: KeyCode = KeyCode(0x2C);
    pub const ESC:   KeyCode = KeyCode(0x29);
}

/// Decoded keyboard event from a HID report.
#[derive(Debug, Clone, Copy)]
pub struct KeyEvent {
    pub modifiers: u8,    // bit0=LCtrl, bit1=LShift, bit2=LAlt, bit3=LGui
    pub keycodes:  [KeyCode; 6],
}

impl KeyEvent {
    pub fn ctrl_held(self)  -> bool { self.modifiers & 0x01 != 0 }
    pub fn shift_held(self) -> bool { self.modifiers & 0x02 != 0 }
    pub fn alt_held(self)   -> bool { self.modifiers & 0x04 != 0 }
}

/// Decoded mouse event from a HID report.
#[derive(Debug, Clone, Copy)]
pub struct MouseEvent {
    pub buttons: u8,   // bit0=left, bit1=right, bit2=middle
    pub dx:      i8,
    pub dy:      i8,
    pub scroll:  i8,
}

impl MouseEvent {
    pub fn left_pressed(self)   -> bool { self.buttons & 0x01 != 0 }
    pub fn right_pressed(self)  -> bool { self.buttons & 0x02 != 0 }
    pub fn middle_pressed(self) -> bool { self.buttons & 0x04 != 0 }
}

/// USB HID driver interface.
pub trait HidDriver {
    /// Poll the device for a new input report. Returns None if no data.
    fn poll(&mut self) -> AxonResult<Option<HidReport>>;
    /// Device type this driver handles.
    fn device_type(&self) -> HidDeviceType;
}

/// Parse a keyboard HID report into a KeyEvent.
pub fn parse_keyboard_report(report: &HidReport) -> KeyEvent {
    if report.len < 8 { return KeyEvent { modifiers: 0, keycodes: [KeyCode::NONE; 6] }; }
    KeyEvent {
        modifiers: report.data[0],
        keycodes: [
            KeyCode(report.data[2]),
            KeyCode(report.data[3]),
            KeyCode(report.data[4]),
            KeyCode(report.data[5]),
            KeyCode(report.data[6]),
            KeyCode(report.data[7]),
        ],
    }
}

/// Parse a mouse HID report into a MouseEvent.
pub fn parse_mouse_report(report: &HidReport) -> MouseEvent {
    if report.len < 4 { return MouseEvent { buttons: 0, dx: 0, dy: 0, scroll: 0 }; }
    MouseEvent {
        buttons: report.data[0],
        dx:      report.data[1] as i8,
        dy:      report.data[2] as i8,
        scroll:  report.data[3] as i8,
    }
}

/// Host stub HID driver — ring buffer backed for testing.
pub struct StubHidDriver {
    device: HidDeviceType,
    queue:  [Option<HidReport>; 16],
    head:   usize,
    tail:   usize,
}

impl StubHidDriver {
    pub const fn new(device: HidDeviceType) -> Self {
        Self { device, queue: [None; 16], head: 0, tail: 0 }
    }

    /// Inject a report into the stub queue (for testing).
    pub fn inject(&mut self, report: HidReport) -> bool {
        let next = (self.tail + 1) % 16;
        if next == self.head { return false; } // full
        self.queue[self.tail] = Some(report);
        self.tail = next;
        true
    }
}

impl HidDriver for StubHidDriver {
    fn poll(&mut self) -> AxonResult<Option<HidReport>> {
        if self.head == self.tail { return AxonResult::Ok(None); }
        let r = self.queue[self.head].take();
        self.head = (self.head + 1) % 16;
        AxonResult::Ok(r)
    }
    fn device_type(&self) -> HidDeviceType { self.device }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tp43_hid_keyboard_report_parse() {
        let mut r = HidReport::empty(HidDeviceType::Keyboard);
        r.data[0] = 0x02; // LShift
        r.data[2] = 0x04; // 'A'
        r.len = 8;
        let ev = parse_keyboard_report(&r);
        assert!(ev.shift_held());
        assert!(!ev.ctrl_held());
        assert_eq!(ev.keycodes[0], KeyCode::A);
    }

    #[test]
    fn tp43_hid_mouse_report_parse() {
        let mut r = HidReport::empty(HidDeviceType::Mouse);
        r.data[0] = 0x01; // left button
        r.data[1] = 5u8;  // dx=+5
        r.data[2] = (-3i8) as u8; // dy=-3
        r.len = 4;
        let ev = parse_mouse_report(&r);
        assert!(ev.left_pressed());
        assert!(!ev.right_pressed());
        assert_eq!(ev.dx, 5);
        assert_eq!(ev.dy, -3);
    }

    #[test]
    fn tp43_hid_stub_driver_inject_poll() {
        let mut drv = StubHidDriver::new(HidDeviceType::Keyboard);
        assert_eq!(drv.poll().unwrap(), None);
        let r = HidReport::empty(HidDeviceType::Keyboard);
        assert!(drv.inject(r));
        assert!(drv.poll().unwrap().is_some());
        assert_eq!(drv.poll().unwrap(), None);
    }

    #[test]
    fn tp43_hid_stub_queue_overflow() {
        let mut drv = StubHidDriver::new(HidDeviceType::Mouse);
        for _ in 0..15 {
            assert!(drv.inject(HidReport::empty(HidDeviceType::Mouse)));
        }
        // Queue full — inject fails
        assert!(!drv.inject(HidReport::empty(HidDeviceType::Mouse)));
    }

    #[test]
    fn tp43_hid_device_type() {
        let drv = StubHidDriver::new(HidDeviceType::Gamepad);
        assert_eq!(drv.device_type(), HidDeviceType::Gamepad);
    }
}
