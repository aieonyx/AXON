// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
// PCM audio encoding/decoding -- sovereign implementation.
// Clean-room: studied ITU-T G.711 and PCM spec only. No code copied.
// P59.0: 16-bit signed PCM, configurable sample rate and channels.
// P59.1: G.711 u-law/a-law compression added.
use crate::error::{MediaError, MediaResult};

pub const SAMPLE_RATE_8KHZ:  u32 = 8_000;
pub const SAMPLE_RATE_44KHZ: u32 = 44_100;
pub const SAMPLE_RATE_48KHZ: u32 = 48_000;

#[derive(Debug, Clone, PartialEq)]
pub struct PcmConfig {
    pub sample_rate: u32,
    pub channels:    u8,
    pub bit_depth:   u8,
}

impl PcmConfig {
    pub fn telephony() -> Self {
        PcmConfig { sample_rate: SAMPLE_RATE_8KHZ, channels: 1, bit_depth: 16 }
    }
    pub fn stereo_44() -> Self {
        PcmConfig { sample_rate: SAMPLE_RATE_44KHZ, channels: 2, bit_depth: 16 }
    }
    pub fn mono_48() -> Self {
        PcmConfig { sample_rate: SAMPLE_RATE_48KHZ, channels: 1, bit_depth: 16 }
    }
    pub fn validate(&self) -> MediaResult<()> {
        if self.sample_rate == 0 { return Err(MediaError::InvalidSampleRate(self.sample_rate)); }
        if self.channels == 0    { return Err(MediaError::InvalidChannels(self.channels)); }
        Ok(())
    }
    pub fn bytes_per_sample(&self) -> usize { (self.bit_depth / 8) as usize }
    pub fn bytes_per_frame(&self) -> usize  { self.bytes_per_sample() * self.channels as usize }
    pub fn bytes_per_second(&self) -> usize { self.bytes_per_frame() * self.sample_rate as usize }
}

#[derive(Debug, Clone)]
pub struct PcmFrame {
    pub config:  PcmConfig,
    pub samples: Vec<i16>,
}

impl PcmFrame {
    pub fn new(config: PcmConfig, samples: Vec<i16>) -> MediaResult<Self> {
        config.validate()?;
        Ok(PcmFrame { config, samples })
    }

    pub fn silence(config: PcmConfig, num_samples: usize) -> MediaResult<Self> {
        config.validate()?;
        Ok(PcmFrame { config, samples: vec![0i16; num_samples] })
    }

    pub fn duration_ms(&self) -> u64 {
        let total = self.samples.len() as u64;
        let rate  = self.config.sample_rate as u64;
        let ch    = self.config.channels as u64;
        if rate == 0 || ch == 0 { return 0; }
        (total * 1000) / (rate * ch)
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(self.samples.len() * 2);
        for &s in &self.samples {
            out.extend_from_slice(&s.to_le_bytes());
        }
        out
    }

    pub fn from_bytes(config: PcmConfig, bytes: &[u8]) -> MediaResult<Self> {
        config.validate()?;
        if bytes.len() % 2 != 0 {
            return Err(MediaError::DecodingFailed("odd byte count for 16-bit PCM".into()));
        }
        let samples = bytes.chunks(2)
            .map(|c| i16::from_le_bytes([c[0], c[1]]))
            .collect();
        Ok(PcmFrame { config, samples })
    }

    pub fn mix(&self, other: &PcmFrame) -> MediaResult<PcmFrame> {
        if self.config != other.config {
            return Err(MediaError::EncodingFailed("config mismatch in mix".into()));
        }
        let len = self.samples.len().min(other.samples.len());
        let samples = (0..len)
            .map(|i| self.samples[i].saturating_add(other.samples[i]))
            .collect();
        Ok(PcmFrame { config: self.config.clone(), samples })
    }

    pub fn amplitude_scale(&self, factor: f32) -> PcmFrame {
        let samples = self.samples.iter()
            .map(|&s| ((s as f32 * factor).clamp(i16::MIN as f32, i16::MAX as f32)) as i16)
            .collect();
        PcmFrame { config: self.config.clone(), samples }
    }
}
