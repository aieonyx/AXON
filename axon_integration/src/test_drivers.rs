// Copyright (c) 2026 Edison Lepiten / AIEONYX
//! Phase 43 integration tests — Phoenix generic drivers.

use axon_drivers::generic::{
    StubHidDriver, HidDeviceType, HidReport, HidDriver,
    StubCdcEcm, CdcEcmDriver, EtherFrame, MacAddr,
    StubDisplay, DisplayDriver, Colour,
    StubStorage, StorageDriver, BLOCK_SIZE,
    StubAudio, AudioDriver, AudioConfig,
};

#[test]
fn p43_hid_keyboard_inject_poll() {
    let mut drv = StubHidDriver::new(HidDeviceType::Keyboard);
    drv.inject(HidReport::empty(HidDeviceType::Keyboard));
    assert!(drv.poll().unwrap().is_some());
    assert!(drv.poll().unwrap().is_none());
}

#[test]
fn p43_net_send_recv_frame() {
    let mac = MacAddr([0x01, 0x02, 0x03, 0x04, 0x05, 0x06]);
    let mut drv = StubCdcEcm::new(mac);
    assert!(drv.link_up());
    let frame = EtherFrame::new(MacAddr::BROADCAST, mac, 0x0800);
    drv.inject(frame);
    assert!(drv.recv().unwrap().is_some());
}

#[test]
fn p43_display_fill_and_pixel() {
    let mut d = StubDisplay::new(100, 100);
    d.fill(Colour::GREEN).unwrap();
    assert_eq!(d.pixel_at(50, 50), Colour::GREEN);
    d.set_pixel(0, 0, Colour::RED).unwrap();
    assert_eq!(d.pixel_at(0, 0), Colour::RED);
}

#[test]
fn p43_storage_write_read_roundtrip() {
    let mut s = StubStorage::new(8);
    let mut buf = [0u8; BLOCK_SIZE];
    buf[0] = 0xFF;
    s.write_block(3, &buf).unwrap();
    let mut out = [0u8; BLOCK_SIZE];
    s.read_block(3, &mut out).unwrap();
    assert_eq!(out[0], 0xFF);
}

#[test]
fn p43_audio_full_cycle() {
    let mut a = StubAudio::new(AudioConfig::CD_QUALITY);
    a.start().unwrap();
    let n = a.write_pcm(&[0u8; 512]).unwrap();
    assert_eq!(n, 512);
    a.stop().unwrap();
    assert!(a.write_pcm(&[0u8; 64]).is_err());
}
