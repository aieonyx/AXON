// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
// RTP packet framing -- sovereign implementation.
// Clean-room: studied RFC 3550 specification only. No code copied.
// P59.0: RTP header construction and parsing, PCM payload.
use crate::error::{MediaError, MediaResult};

pub const RTP_VERSION: u8 = 2;
pub const PT_PCMU:     u8 = 0;   // G.711 u-law
pub const PT_PCMA:     u8 = 8;   // G.711 a-law
pub const PT_L16_MONO: u8 = 11;  // Linear 16-bit mono
pub const PT_L16_STEREO: u8 = 10; // Linear 16-bit stereo

#[derive(Debug, Clone, PartialEq)]
pub struct RtpHeader {
    pub version:        u8,
    pub padding:        bool,
    pub extension:      bool,
    pub marker:         bool,
    pub payload_type:   u8,
    pub sequence:       u16,
    pub timestamp:      u32,
    pub ssrc:           u32,
}

impl RtpHeader {
    pub fn new(payload_type: u8, sequence: u16, timestamp: u32, ssrc: u32) -> Self {
        RtpHeader {
            version: RTP_VERSION,
            padding: false,
            extension: false,
            marker: false,
            payload_type,
            sequence,
            timestamp,
            ssrc,
        }
    }

    pub fn to_bytes(&self) -> [u8; 12] {
        let mut h = [0u8; 12];
        h[0] = (self.version << 6)
             | ((self.padding   as u8) << 5)
             | ((self.extension as u8) << 4);
        h[1] = ((self.marker as u8) << 7) | (self.payload_type & 0x7f);
        h[2..4].copy_from_slice(&self.sequence.to_be_bytes());
        h[4..8].copy_from_slice(&self.timestamp.to_be_bytes());
        h[8..12].copy_from_slice(&self.ssrc.to_be_bytes());
        h
    }

    pub fn from_bytes(b: &[u8]) -> MediaResult<Self> {
        if b.len() < 12 {
            return Err(MediaError::DecodingFailed("RTP header too short".into()));
        }
        let version = (b[0] >> 6) & 0x3;
        if version != RTP_VERSION {
            return Err(MediaError::DecodingFailed(format!("bad RTP version: {}", version)));
        }
        Ok(RtpHeader {
            version,
            padding:      (b[0] & 0x20) != 0,
            extension:    (b[0] & 0x10) != 0,
            marker:       (b[1] & 0x80) != 0,
            payload_type: b[1] & 0x7f,
            sequence:     u16::from_be_bytes([b[2], b[3]]),
            timestamp:    u32::from_be_bytes([b[4], b[5], b[6], b[7]]),
            ssrc:         u32::from_be_bytes([b[8], b[9], b[10], b[11]]),
        })
    }
}

#[derive(Debug, Clone)]
pub struct RtpPacket {
    pub header:  RtpHeader,
    pub payload: Vec<u8>,
}

impl RtpPacket {
    pub fn new(header: RtpHeader, payload: Vec<u8>) -> Self {
        RtpPacket { header, payload }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(12 + self.payload.len());
        out.extend_from_slice(&self.header.to_bytes());
        out.extend_from_slice(&self.payload);
        out
    }

    pub fn from_bytes(b: &[u8]) -> MediaResult<Self> {
        if b.len() < 12 {
            return Err(MediaError::DecodingFailed("packet too short".into()));
        }
        let header  = RtpHeader::from_bytes(&b[..12])?;
        let payload = b[12..].to_vec();
        Ok(RtpPacket { header, payload })
    }

    pub fn total_bytes(&self) -> usize { 12 + self.payload.len() }
}

pub struct RtpSession {
    pub ssrc:      u32,
    pub sequence:  u16,
    pub timestamp: u32,
    pub pt:        u8,
}

impl RtpSession {
    pub fn new(ssrc: u32, pt: u8) -> Self {
        RtpSession { ssrc, sequence: 0, timestamp: 0, pt }
    }

    pub fn next_packet(&mut self, payload: Vec<u8>, ts_increment: u32) -> RtpPacket {
        let hdr = RtpHeader::new(self.pt, self.sequence, self.timestamp, self.ssrc);
        self.sequence  = self.sequence.wrapping_add(1);
        self.timestamp = self.timestamp.wrapping_add(ts_increment);
        RtpPacket::new(hdr, payload)
    }
}
