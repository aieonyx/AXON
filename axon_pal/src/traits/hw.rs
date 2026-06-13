// Copyright (c) 2026 Edison Lepiten / AIEONYX
//! Hardware peripheral abstraction traits — UART, GPIO, Timer.
//!
//! These traits define the sovereign driver PAL interface.
//! Implementations live in axon_pal_linux (host) and axon_pal_sel4 (target).

use axon_core::prelude::*;

// ── UART ─────────────────────────────────────────────────────────────────────

/// UART baud rate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BaudRate {
    B9600   = 9600,
    B19200  = 19200,
    B38400  = 38400,
    B57600  = 57600,
    B115200 = 115200,
}

/// UART configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UartConfig {
    pub baud:      BaudRate,
    pub data_bits: u8,   // 5-8
    pub stop_bits: u8,   // 1-2
    pub parity:    bool, // true = even parity
}

impl UartConfig {
    /// Standard 115200 8N1 — default for BASTION nodes.
    pub const DEFAULT: Self = Self {
        baud:      BaudRate::B115200,
        data_bits: 8,
        stop_bits: 1,
        parity:    false,
    };
}

/// UART peripheral abstraction.
pub trait PalUart {
    /// Initialise UART with the given configuration.
    fn uart_init(port: u32, config: UartConfig) -> AxonResult<()>;
    /// Write a byte to the UART TX FIFO. Blocks until space available.
    fn uart_write_byte(port: u32, byte: u8) -> AxonResult<()>;
    /// Write a buffer to UART. Returns bytes written.
    fn uart_write(port: u32, buf: &[u8]) -> AxonResult<usize> {
        let mut written = 0;
        for &b in buf {
            axon_try!(Self::uart_write_byte(port, b));
            written += 1;
        }
        AxonResult::Ok(written)
    }
    /// Read a byte from UART RX FIFO. Returns None if no data available.
    fn uart_read_byte(port: u32) -> AxonResult<Option<u8>>;
    /// Flush UART TX FIFO — wait until all bytes transmitted.
    fn uart_flush(port: u32) -> AxonResult<()>;
}

// ── GPIO ─────────────────────────────────────────────────────────────────────

/// GPIO pin direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GpioDirection { Input, Output }

/// GPIO pin state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GpioState { Low, High }

impl GpioState {
    pub fn is_high(self) -> bool { matches!(self, GpioState::High) }
    pub fn is_low(self)  -> bool { matches!(self, GpioState::Low)  }
    pub fn toggle(self)  -> Self {
        match self { GpioState::High => GpioState::Low, GpioState::Low => GpioState::High }
    }
}

/// GPIO peripheral abstraction.
pub trait PalGpio {
    /// Configure a pin as input or output.
    fn gpio_set_direction(pin: u32, dir: GpioDirection) -> AxonResult<()>;
    /// Set output pin high or low.
    fn gpio_write(pin: u32, state: GpioState) -> AxonResult<()>;
    /// Read input pin state.
    fn gpio_read(pin: u32) -> AxonResult<GpioState>;
    /// Toggle output pin state.
    fn gpio_toggle(pin: u32) -> AxonResult<()> {
        let current = axon_try!(Self::gpio_read(pin));
        Self::gpio_write(pin, current.toggle())
    }
}

// ── Timer ─────────────────────────────────────────────────────────────────────

/// Timer tick count — monotonic, hardware-sourced.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct TimerTicks(pub u64);

impl TimerTicks {
    pub const ZERO: Self = Self(0);
    pub fn elapsed_since(self, earlier: TimerTicks) -> u64 {
        self.0.saturating_sub(earlier.0)
    }
}

/// Timer peripheral abstraction.
pub trait PalTimer {
    /// Returns the current hardware tick count.
    fn timer_ticks() -> AxonResult<TimerTicks>;
    /// Returns the timer frequency in Hz (ticks per second).
    fn timer_freq_hz() -> u64;
    /// Convert ticks to microseconds.
    fn ticks_to_us(ticks: u64) -> u64 {
        ticks * 1_000_000 / Self::timer_freq_hz().max(1)
    }
    /// Busy-wait for approximately `us` microseconds.
    fn timer_delay_us(us: u64) -> AxonResult<()> {
        let start = axon_try!(Self::timer_ticks());
        let target = start.0 + us * Self::timer_freq_hz() / 1_000_000;
        loop {
            let now = axon_try!(Self::timer_ticks());
            if now.0 >= target { break; }
        }
        AxonResult::Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tp39_uart_config_default() {
        let c = UartConfig::DEFAULT;
        assert_eq!(c.baud, BaudRate::B115200);
        assert_eq!(c.data_bits, 8);
        assert_eq!(c.stop_bits, 1);
        assert!(!c.parity);
    }

    #[test]
    fn tp39_gpio_state_toggle() {
        assert_eq!(GpioState::High.toggle(), GpioState::Low);
        assert_eq!(GpioState::Low.toggle(),  GpioState::High);
    }

    #[test]
    fn tp39_gpio_state_predicates() {
        assert!(GpioState::High.is_high());
        assert!(GpioState::Low.is_low());
        assert!(!GpioState::High.is_low());
    }

    #[test]
    fn tp39_timer_ticks_elapsed() {
        let t0 = TimerTicks(100);
        let t1 = TimerTicks(350);
        assert_eq!(t1.elapsed_since(t0), 250);
    }

    #[test]
    fn tp39_timer_ticks_saturating() {
        let t0 = TimerTicks(500);
        let t1 = TimerTicks(100);
        assert_eq!(t1.elapsed_since(t0), 0); // saturating_sub
    }

    #[test]
    fn tp39_baud_rate_values() {
        assert_eq!(BaudRate::B115200 as u32, 115200);
        assert_eq!(BaudRate::B9600   as u32, 9600);
    }
}
