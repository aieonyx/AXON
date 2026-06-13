// Copyright (c) 2026 Edison Lepiten / AIEONYX
//! USB CDC-ECM network driver — Ethernet over USB.
//!
//! Implements the USB CDC-ECM class (USB CDC spec §3.3).
//! Provides Ethernet frame send/receive over a USB bulk endpoint pair.

use axon_core::prelude::*;

/// Maximum Ethernet frame size (1518 bytes + 4 CRC).
pub const MAX_FRAME_SIZE: usize = 1522;

/// MAC address — 6 bytes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MacAddr(pub [u8; 6]);

impl MacAddr {
    pub const ZERO: MacAddr = MacAddr([0u8; 6]);
    pub const BROADCAST: MacAddr = MacAddr([0xFF; 6]);

    pub fn is_broadcast(self) -> bool { self == Self::BROADCAST }
    pub fn is_multicast(self) -> bool { self.0[0] & 0x01 != 0 }
}

/// An Ethernet frame — header + payload.
#[derive(Debug, Clone)]
pub struct EtherFrame {
    pub dst:      MacAddr,
    pub src:      MacAddr,
    pub ethertype: u16,
    pub payload:  [u8; MAX_FRAME_SIZE],
    pub payload_len: usize,
}

impl EtherFrame {
    pub fn new(dst: MacAddr, src: MacAddr, ethertype: u16) -> Self {
        Self { dst, src, ethertype, payload: [0u8; MAX_FRAME_SIZE], payload_len: 0 }
    }
    pub fn total_len(&self) -> usize { 14 + self.payload_len }
}

/// CDC-ECM driver interface.
pub trait CdcEcmDriver {
    fn mac_addr(&self) -> MacAddr;
    fn send(&mut self, frame: &EtherFrame) -> AxonResult<()>;
    fn recv(&mut self) -> AxonResult<Option<EtherFrame>>;
    fn link_up(&self) -> bool;
}

/// Host stub CDC-ECM driver.
pub struct StubCdcEcm {
    mac: MacAddr,
    rx_queue: [Option<EtherFrame>; 8],
    head: usize,
    tail: usize,
    link: bool,
}

impl StubCdcEcm {
    pub fn new(mac: MacAddr) -> Self {
        Self { mac, rx_queue: core::array::from_fn(|_| None), head: 0, tail: 0, link: true }
    }
    pub fn inject(&mut self, frame: EtherFrame) -> bool {
        let next = (self.tail + 1) % 8;
        if next == self.head { return false; }
        self.rx_queue[self.tail] = Some(frame);
        self.tail = next;
        true
    }
}

impl CdcEcmDriver for StubCdcEcm {
    fn mac_addr(&self) -> MacAddr { self.mac }
    fn send(&mut self, _frame: &EtherFrame) -> AxonResult<()> { AxonResult::Ok(()) }
    fn recv(&mut self) -> AxonResult<Option<EtherFrame>> {
        if self.head == self.tail { return AxonResult::Ok(None); }
        let f = self.rx_queue[self.head].take();
        self.head = (self.head + 1) % 8;
        AxonResult::Ok(f)
    }
    fn link_up(&self) -> bool { self.link }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tp43_net_mac_addr() {
        let mac = MacAddr([0x00, 0x11, 0x22, 0x33, 0x44, 0x55]);
        assert!(!mac.is_broadcast());
        assert!(!mac.is_multicast());
        assert!(MacAddr::BROADCAST.is_broadcast());
    }

    #[test]
    fn tp43_net_stub_send_recv() {
        let mac = MacAddr([0xAA; 6]);
        let mut drv = StubCdcEcm::new(mac);
        assert!(drv.link_up());
        let frame = EtherFrame::new(MacAddr::BROADCAST, mac, 0x0800);
        assert!(drv.inject(frame));
        assert!(drv.recv().unwrap().is_some());
        assert!(drv.recv().unwrap().is_none());
    }

    #[test]
    fn tp43_net_mac_eq() {
        assert_eq!(MacAddr::ZERO, MacAddr([0u8; 6]));
        assert_ne!(MacAddr::ZERO, MacAddr::BROADCAST);
    }
}
