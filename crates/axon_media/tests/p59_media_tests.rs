// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
// P59 QA -- axon_media sovereign audio/video tests
// Pass bar: 22/22
// P3 Doctrine check: complements axon_net (P56), axon_gpu (P58), axon_crypto (P57)
use axon_media::{
    PcmConfig, PcmFrame, SAMPLE_RATE_8KHZ, SAMPLE_RATE_44KHZ,
    RtpHeader, RtpPacket, RtpSession, RTP_VERSION, PT_L16_MONO,
    VideoFrame, PixelFormat, MediaError,
};

// ── PCM tests ─────────────────────────────────────────────────────────────────

#[test]
fn test_pcm_telephony_config() {
    let cfg = PcmConfig::telephony();
    assert_eq!(cfg.sample_rate, SAMPLE_RATE_8KHZ);
    assert_eq!(cfg.channels, 1);
    assert_eq!(cfg.bit_depth, 16);
}

#[test]
fn test_pcm_config_validate_valid() {
    assert!(PcmConfig::telephony().validate().is_ok());
    assert!(PcmConfig::stereo_44().validate().is_ok());
}

#[test]
fn test_pcm_config_validate_invalid() {
    let bad = PcmConfig { sample_rate: 0, channels: 1, bit_depth: 16 };
    assert!(bad.validate().is_err());
    let bad2 = PcmConfig { sample_rate: 8000, channels: 0, bit_depth: 16 };
    assert!(bad2.validate().is_err());
}

#[test]
fn test_pcm_bytes_per_second() {
    let cfg = PcmConfig::telephony();
    assert_eq!(cfg.bytes_per_second(), 16_000);
}

#[test]
fn test_pcm_silence() {
    let frame = PcmFrame::silence(PcmConfig::telephony(), 80).unwrap();
    assert_eq!(frame.samples.len(), 80);
    assert!(frame.samples.iter().all(|&s| s == 0));
}

#[test]
fn test_pcm_to_from_bytes_roundtrip() {
    let cfg     = PcmConfig::telephony();
    let samples = vec![100i16, -200, 300, -400];
    let frame   = PcmFrame::new(cfg.clone(), samples.clone()).unwrap();
    let bytes   = frame.to_bytes();
    let decoded = PcmFrame::from_bytes(cfg, &bytes).unwrap();
    assert_eq!(decoded.samples, samples);
}

#[test]
fn test_pcm_duration_ms() {
    let cfg   = PcmConfig::telephony(); // 8000 Hz mono
    let frame = PcmFrame::silence(cfg, 80).unwrap(); // 80 samples = 10ms
    assert_eq!(frame.duration_ms(), 10);
}

#[test]
fn test_pcm_mix() {
    let cfg = PcmConfig::telephony();
    let a   = PcmFrame::new(cfg.clone(), vec![100, 200]).unwrap();
    let b   = PcmFrame::new(cfg, vec![50, 100]).unwrap();
    let m   = a.mix(&b).unwrap();
    assert_eq!(m.samples, vec![150, 300]);
}

#[test]
fn test_pcm_amplitude_scale() {
    let cfg   = PcmConfig::telephony();
    let frame = PcmFrame::new(cfg, vec![100, 200, -100]).unwrap();
    let scaled = frame.amplitude_scale(2.0);
    assert_eq!(scaled.samples, vec![200, 400, -200]);
}

#[test]
fn test_pcm_amplitude_clamp() {
    let cfg   = PcmConfig::telephony();
    let frame = PcmFrame::new(cfg, vec![i16::MAX]).unwrap();
    let scaled = frame.amplitude_scale(2.0);
    assert_eq!(scaled.samples[0], i16::MAX);
}

// ── RTP tests ─────────────────────────────────────────────────────────────────

#[test]
fn test_rtp_header_to_from_bytes() {
    let hdr  = RtpHeader::new(PT_L16_MONO, 42, 1000, 0xdeadbeef);
    let bytes = hdr.to_bytes();
    let decoded = RtpHeader::from_bytes(&bytes).unwrap();
    assert_eq!(decoded.version,      RTP_VERSION);
    assert_eq!(decoded.payload_type, PT_L16_MONO);
    assert_eq!(decoded.sequence,     42);
    assert_eq!(decoded.timestamp,    1000);
    assert_eq!(decoded.ssrc,         0xdeadbeef);
}

#[test]
fn test_rtp_header_version_bits() {
    let hdr   = RtpHeader::new(PT_L16_MONO, 0, 0, 0);
    let bytes = hdr.to_bytes();
    assert_eq!((bytes[0] >> 6) & 0x3, RTP_VERSION);
}

#[test]
fn test_rtp_packet_roundtrip() {
    let hdr     = RtpHeader::new(PT_L16_MONO, 1, 160, 42);
    let payload = vec![0u8; 160];
    let pkt     = RtpPacket::new(hdr, payload.clone());
    let bytes   = pkt.to_bytes();
    let decoded = RtpPacket::from_bytes(&bytes).unwrap();
    assert_eq!(decoded.payload, payload);
    assert_eq!(decoded.header.sequence, 1);
}

#[test]
fn test_rtp_packet_total_bytes() {
    let hdr = RtpHeader::new(PT_L16_MONO, 0, 0, 0);
    let pkt = RtpPacket::new(hdr, vec![0u8; 160]);
    assert_eq!(pkt.total_bytes(), 172);
}

#[test]
fn test_rtp_session_sequence_increments() {
    let mut sess = RtpSession::new(0x1234, PT_L16_MONO);
    let p1 = sess.next_packet(vec![0u8; 10], 160);
    let p2 = sess.next_packet(vec![0u8; 10], 160);
    assert_eq!(p1.header.sequence, 0);
    assert_eq!(p2.header.sequence, 1);
}

#[test]
fn test_rtp_session_timestamp_increments() {
    let mut sess = RtpSession::new(0x1234, PT_L16_MONO);
    let p1 = sess.next_packet(vec![], 160);
    let p2 = sess.next_packet(vec![], 160);
    assert_eq!(p1.header.timestamp, 0);
    assert_eq!(p2.header.timestamp, 160);
}

#[test]
fn test_rtp_header_too_short_fails() {
    assert!(RtpHeader::from_bytes(&[0u8; 8]).is_err());
}

// ── VideoFrame tests ──────────────────────────────────────────────────────────

#[test]
fn test_video_frame_new_rgb24() {
    let f = VideoFrame::new(4, 4, PixelFormat::Rgb24).unwrap();
    assert_eq!(f.size_bytes(), 48);
}

#[test]
fn test_video_frame_zero_size_fails() {
    assert!(VideoFrame::new(0, 4, PixelFormat::Rgb24).is_err());
    assert!(VideoFrame::new(4, 0, PixelFormat::Rgb24).is_err());
}

#[test]
fn test_video_frame_set_get_pixel() {
    let mut f = VideoFrame::new(4, 4, PixelFormat::Rgb24).unwrap();
    f.set_pixel_rgb(1, 2, 255, 128, 0).unwrap();
    let (r, g, b) = f.get_pixel_rgb(1, 2).unwrap();
    assert_eq!((r, g, b), (255, 128, 0));
}

#[test]
fn test_video_frame_fill() {
    let mut f = VideoFrame::new(2, 2, PixelFormat::Rgb24).unwrap();
    f.fill(255, 0, 0).unwrap();
    let (r, g, b) = f.get_pixel_rgb(0, 0).unwrap();
    assert_eq!((r, g, b), (255, 0, 0));
}

#[test]
fn test_video_frame_yuv420_size() {
    let pixels = 4 * 4;
    assert_eq!(VideoFrame::frame_size(4, 4, &PixelFormat::Yuv420), pixels + pixels/2);
}
