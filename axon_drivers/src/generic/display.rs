// Copyright (c) 2026 Edison Lepiten / AIEONYX
//! VESA/GOP display framebuffer driver.
//!
//! Provides a linear framebuffer abstraction over VESA BIOS Extensions
//! or UEFI GOP (Graphics Output Protocol).

use axon_core::prelude::*;

/// Pixel format.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PixelFormat { Rgb888, Bgr888, Rgba8888 }

impl PixelFormat {
    pub fn bytes_per_pixel(self) -> usize {
        match self { PixelFormat::Rgb888 | PixelFormat::Bgr888 => 3, PixelFormat::Rgba8888 => 4 }
    }
}

/// Display mode — resolution and pixel format.
#[derive(Debug, Clone, Copy)]
pub struct DisplayMode {
    pub width:        u32,
    pub height:       u32,
    pub stride:       u32,  // bytes per scanline
    pub pixel_format: PixelFormat,
}

impl DisplayMode {
    pub fn framebuffer_size(&self) -> usize {
        (self.stride * self.height) as usize
    }
}

/// A 24-bit RGB colour.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Colour { pub r: u8, pub g: u8, pub b: u8 }

impl Colour {
    pub const BLACK:   Colour = Colour { r: 0,   g: 0,   b: 0   };
    pub const WHITE:   Colour = Colour { r: 255, g: 255, b: 255 };
    pub const RED:     Colour = Colour { r: 255, g: 0,   b: 0   };
    pub const GREEN:   Colour = Colour { r: 0,   g: 255, b: 0   };
    pub const BLUE:    Colour = Colour { r: 0,   g: 0,   b: 255 };

    pub fn to_rgb888(self) -> [u8; 3] { [self.r, self.g, self.b] }
    pub fn to_bgr888(self) -> [u8; 3] { [self.b, self.g, self.r] }
}

/// Framebuffer display driver.
pub trait DisplayDriver {
    fn mode(&self) -> DisplayMode;
    fn set_pixel(&mut self, x: u32, y: u32, colour: Colour) -> AxonResult<()>;
    fn fill(&mut self, colour: Colour) -> AxonResult<()>;
    fn flush(&mut self) -> AxonResult<()> { AxonResult::Ok(()) }
}

/// Host stub display driver — in-memory framebuffer.
pub struct StubDisplay {
    mode: DisplayMode,
    buffer: alloc::vec::Vec<u8>,
}

extern crate alloc;

impl StubDisplay {
    pub fn new(width: u32, height: u32) -> Self {
        let mode = DisplayMode {
            width, height,
            stride: width * 3,
            pixel_format: PixelFormat::Rgb888,
        };
        let size = mode.framebuffer_size();
        Self { mode, buffer: alloc::vec![0u8; size] }
    }

    pub fn pixel_at(&self, x: u32, y: u32) -> Colour {
        debug_assert!(x < self.mode.width, "x out of bounds");
        debug_assert!(y < self.mode.height, "y out of bounds");
        let off = (y * self.mode.stride + x * 3) as usize;
        Colour { r: self.buffer[off], g: self.buffer[off+1], b: self.buffer[off+2] }
    }
}

impl DisplayDriver for StubDisplay {
    fn mode(&self) -> DisplayMode { self.mode }

    fn set_pixel(&mut self, x: u32, y: u32, colour: Colour) -> AxonResult<()> {
        if x >= self.mode.width || y >= self.mode.height {
            return AxonResult::Err(AxonError::invalid_input("pixel out of bounds"));
        }
        let off = (y * self.mode.stride + x * 3) as usize;
        let px = colour.to_rgb888();
        self.buffer[off]   = px[0];
        self.buffer[off+1] = px[1];
        self.buffer[off+2] = px[2];
        AxonResult::Ok(())
    }

    fn fill(&mut self, colour: Colour) -> AxonResult<()> {
        let px = colour.to_rgb888();
        for chunk in self.buffer.chunks_mut(3) {
            chunk[0] = px[0]; chunk[1] = px[1]; chunk[2] = px[2];
        }
        AxonResult::Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tp43_display_set_pixel() {
        let mut d = StubDisplay::new(64, 64);
        d.set_pixel(10, 10, Colour::RED).unwrap();
        assert_eq!(d.pixel_at(10, 10), Colour::RED);
        assert_eq!(d.pixel_at(0, 0), Colour::BLACK);
    }

    #[test]
    fn tp43_display_fill() {
        let mut d = StubDisplay::new(32, 32);
        d.fill(Colour::BLUE).unwrap();
        assert_eq!(d.pixel_at(0, 0), Colour::BLUE);
        assert_eq!(d.pixel_at(31, 31), Colour::BLUE);
    }

    #[test]
    fn tp43_display_out_of_bounds() {
        let mut d = StubDisplay::new(16, 16);
        assert!(d.set_pixel(16, 0, Colour::WHITE).is_err());
        assert!(d.set_pixel(0, 16, Colour::WHITE).is_err());
    }

    #[test]
    fn tp43_display_mode() {
        let d = StubDisplay::new(1920, 1080);
        assert_eq!(d.mode().width, 1920);
        assert_eq!(d.mode().height, 1080);
        assert_eq!(d.mode().framebuffer_size(), 1920 * 1080 * 3);
    }

    #[test]
    fn tp43_colour_conversions() {
        assert_eq!(Colour::RED.to_rgb888(), [255, 0, 0]);
        assert_eq!(Colour::RED.to_bgr888(), [0, 0, 255]);
    }
}
