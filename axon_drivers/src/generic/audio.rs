// Copyright (c) 2026 Edison Lepiten / AIEONYX
//! Intel HDA (High Definition Audio) driver stub.
//!
//! Provides PCM audio output via Intel HDA codec interface.
//! On aarch64-seL4: wired to HDA MMIO registers (P44).

use axon_core::prelude::*;

/// Audio sample format.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SampleFormat { S16Le, S24Le, S32Le, F32Le }

impl SampleFormat {
    pub fn bytes_per_sample(self) -> usize {
        match self { SampleFormat::S16Le => 2, SampleFormat::S24Le => 3,
                     SampleFormat::S32Le | SampleFormat::F32Le => 4 }
    }
}

/// Audio stream configuration.
#[derive(Debug, Clone, Copy)]
pub struct AudioConfig {
    pub sample_rate:  u32,
    pub channels:     u8,
    pub format:       SampleFormat,
}

impl AudioConfig {
    pub const CD_QUALITY: Self = Self {
        sample_rate: 44100, channels: 2, format: SampleFormat::S16Le,
    };
    pub const STUDIO_QUALITY: Self = Self {
        sample_rate: 96000, channels: 2, format: SampleFormat::S24Le,
    };
    pub fn bytes_per_frame(&self) -> usize {
        self.channels as usize * self.format.bytes_per_sample()
    }
}

/// HDA audio driver interface.
pub trait AudioDriver {
    fn config(&self) -> AudioConfig;
    fn write_pcm(&mut self, samples: &[u8]) -> AxonResult<usize>;
    fn set_volume(&mut self, volume: u8) -> AxonResult<()>; // 0-255
    fn start(&mut self) -> AxonResult<()>;
    fn stop(&mut self) -> AxonResult<()>;
}

/// Host stub audio driver — discards samples, tracks volume.
pub struct StubAudio {
    config:  AudioConfig,
    volume:  u8,
    running: bool,
    bytes_written: usize,
}

impl StubAudio {
    pub fn new(config: AudioConfig) -> Self {
        Self { config, volume: 128, running: false, bytes_written: 0 }
    }
    pub fn bytes_written(&self) -> usize { self.bytes_written }
}

impl AudioDriver for StubAudio {
    fn config(&self) -> AudioConfig { self.config }
    fn write_pcm(&mut self, samples: &[u8]) -> AxonResult<usize> {
        if !self.running {
            return AxonResult::Err(AxonError::invalid_state("audio stream not started"));
        }
        self.bytes_written += samples.len();
        AxonResult::Ok(samples.len())
    }
    fn set_volume(&mut self, volume: u8) -> AxonResult<()> {
        self.volume = volume;
        AxonResult::Ok(())
    }
    fn start(&mut self) -> AxonResult<()> { self.running = true;  AxonResult::Ok(()) }
    fn stop(&mut self)  -> AxonResult<()> { self.running = false; AxonResult::Ok(()) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tp43_audio_start_write_stop() {
        let mut a = StubAudio::new(AudioConfig::CD_QUALITY);
        a.start().unwrap();
        let n = a.write_pcm(&[0u8; 1024]).unwrap();
        assert_eq!(n, 1024);
        assert_eq!(a.bytes_written(), 1024);
        a.stop().unwrap();
    }

    #[test]
    fn tp43_audio_write_before_start_fails() {
        let mut a = StubAudio::new(AudioConfig::CD_QUALITY);
        assert!(a.write_pcm(&[0u8; 64]).is_err());
    }

    #[test]
    fn tp43_audio_volume() {
        let mut a = StubAudio::new(AudioConfig::CD_QUALITY);
        a.set_volume(200).unwrap();
        assert_eq!(a.volume, 200);
    }

    #[test]
    fn tp43_audio_config_bytes_per_frame() {
        assert_eq!(AudioConfig::CD_QUALITY.bytes_per_frame(), 4); // 2ch * 2 bytes
        assert_eq!(AudioConfig::STUDIO_QUALITY.bytes_per_frame(), 6); // 2ch * 3 bytes
    }
}
