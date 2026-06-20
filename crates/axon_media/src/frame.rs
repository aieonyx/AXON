// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
// VideoFrame -- sovereign video frame buffer.
// P59.0: RGB24 and YUV420 frame formats.
// P59.1: Hardware-accelerated encoding via axon_gpu.
use crate::error::{MediaError, MediaResult};

#[derive(Debug, Clone, PartialEq)]
pub enum PixelFormat {
    Rgb24,
    Rgba32,
    Yuv420,
    Gray8,
}

impl PixelFormat {
    pub fn bytes_per_pixel(&self) -> usize {
        match self {
            PixelFormat::Rgb24   => 3,
            PixelFormat::Rgba32  => 4,
            PixelFormat::Yuv420  => 0, // planar — use frame_size()
            PixelFormat::Gray8   => 1,
        }
    }
}

#[derive(Debug, Clone)]
pub struct VideoFrame {
    pub width:  u32,
    pub height: u32,
    pub format: PixelFormat,
    pub data:   Vec<u8>,
}

impl VideoFrame {
    pub fn new(width: u32, height: u32, format: PixelFormat) -> MediaResult<Self> {
        if width == 0 || height == 0 { return Err(MediaError::InvalidFrame); }
        let size = Self::frame_size(width, height, &format);
        Ok(VideoFrame { width, height, format, data: vec![0u8; size] })
    }

    pub fn from_data(width: u32, height: u32, format: PixelFormat, data: Vec<u8>) -> MediaResult<Self> {
        if width == 0 || height == 0 { return Err(MediaError::InvalidFrame); }
        let expected = Self::frame_size(width, height, &format);
        if data.len() != expected {
            return Err(MediaError::BufferTooSmall { needed: expected, got: data.len() });
        }
        Ok(VideoFrame { width, height, format, data })
    }

    pub fn frame_size(width: u32, height: u32, format: &PixelFormat) -> usize {
        let pixels = (width * height) as usize;
        match format {
            PixelFormat::Rgb24   => pixels * 3,
            PixelFormat::Rgba32  => pixels * 4,
            PixelFormat::Yuv420  => pixels + pixels / 2,
            PixelFormat::Gray8   => pixels,
        }
    }

    pub fn size_bytes(&self) -> usize { self.data.len() }

    pub fn get_pixel_rgb(&self, x: u32, y: u32) -> MediaResult<(u8, u8, u8)> {
        if x >= self.width || y >= self.height { return Err(MediaError::InvalidFrame); }
        match self.format {
            PixelFormat::Rgb24 => {
                let idx = ((y * self.width + x) * 3) as usize;
                Ok((self.data[idx], self.data[idx+1], self.data[idx+2]))
            }
            PixelFormat::Rgba32 => {
                let idx = ((y * self.width + x) * 4) as usize;
                Ok((self.data[idx], self.data[idx+1], self.data[idx+2]))
            }
            PixelFormat::Gray8 => {
                let idx = (y * self.width + x) as usize;
                let v = self.data[idx];
                Ok((v, v, v))
            }
            PixelFormat::Yuv420 => Err(MediaError::DecodingFailed("use yuv accessors".into())),
        }
    }

    pub fn set_pixel_rgb(&mut self, x: u32, y: u32, r: u8, g: u8, b: u8) -> MediaResult<()> {
        if x >= self.width || y >= self.height { return Err(MediaError::InvalidFrame); }
        match self.format {
            PixelFormat::Rgb24 => {
                let idx = ((y * self.width + x) * 3) as usize;
                self.data[idx]   = r;
                self.data[idx+1] = g;
                self.data[idx+2] = b;
                Ok(())
            }
            PixelFormat::Rgba32 => {
                let idx = ((y * self.width + x) * 4) as usize;
                self.data[idx]   = r;
                self.data[idx+1] = g;
                self.data[idx+2] = b;
                Ok(())
            }
            _ => Err(MediaError::EncodingFailed("format not supported for set_pixel_rgb".into())),
        }
    }

    pub fn fill(&mut self, r: u8, g: u8, b: u8) -> MediaResult<()> {
        match self.format {
            PixelFormat::Rgb24 => {
                for chunk in self.data.chunks_mut(3) {
                    chunk[0] = r; chunk[1] = g; chunk[2] = b;
                }
                Ok(())
            }
            PixelFormat::Rgba32 => {
                for chunk in self.data.chunks_mut(4) {
                    chunk[0] = r; chunk[1] = g; chunk[2] = b; chunk[3] = 255;
                }
                Ok(())
            }
            PixelFormat::Gray8 => {
                let lum = ((r as u16 * 299 + g as u16 * 587 + b as u16 * 114) / 1000) as u8;
                self.data.iter_mut().for_each(|p| *p = lum);
                Ok(())
            }
            PixelFormat::Yuv420 => Err(MediaError::EncodingFailed("yuv fill not supported at P59.0".into())),
        }
    }
}
