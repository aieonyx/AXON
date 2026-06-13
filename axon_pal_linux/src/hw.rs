// Copyright (c) 2026 Edison Lepiten / AIEONYX
//! Hardware PAL — Linux host stubs for UART, GPIO, Timer.
//!
//! On Linux: UART → /dev/ttyS*, GPIO → sysfs (stubbed), Timer → CLOCK_MONOTONIC.

use axon_core::prelude::*;
use axon_pal::traits::hw::{
    PalUart, PalGpio, PalTimer,
    UartConfig, GpioDirection, GpioState, TimerTicks,
};

pub struct LinuxHwPal;

// ── UART — stub (real impl would open /dev/ttyS<port>) ───────────────────────
impl PalUart for LinuxHwPal {
    fn uart_init(_port: u32, _config: UartConfig) -> AxonResult<()> {
        // Host stub — no real UART on x86_64 dev machine
        AxonResult::Ok(())
    }
    fn uart_write_byte(_port: u32, _byte: u8) -> AxonResult<()> {
        AxonResult::Ok(())
    }
    fn uart_read_byte(_port: u32) -> AxonResult<Option<u8>> {
        AxonResult::Ok(None)
    }
    fn uart_flush(_port: u32) -> AxonResult<()> {
        AxonResult::Ok(())
    }
}

// ── GPIO — stub (real impl would write to /sys/class/gpio/) ──────────────────

/// In-memory GPIO state for host testing.
static GPIO_STATE: core::sync::atomic::AtomicU64 = core::sync::atomic::AtomicU64::new(0);

impl PalGpio for LinuxHwPal {
    fn gpio_set_direction(_pin: u32, _dir: GpioDirection) -> AxonResult<()> {
        AxonResult::Ok(())
    }
    fn gpio_write(pin: u32, state: GpioState) -> AxonResult<()> {
        if pin >= 64 { return AxonResult::Err(AxonError::invalid_input("pin out of range")); }
        let mask = 1u64 << pin;
        let cur = GPIO_STATE.load(core::sync::atomic::Ordering::Relaxed);
        let new = match state {
            GpioState::High => cur |  mask,
            GpioState::Low  => cur & !mask,
        };
        GPIO_STATE.store(new, core::sync::atomic::Ordering::Relaxed);
        AxonResult::Ok(())
    }
    fn gpio_read(pin: u32) -> AxonResult<GpioState> {
        if pin >= 64 { return AxonResult::Err(AxonError::invalid_input("pin out of range")); }
        let cur = GPIO_STATE.load(core::sync::atomic::Ordering::Relaxed);
        if (cur >> pin) & 1 == 1 { AxonResult::Ok(GpioState::High) }
        else                      { AxonResult::Ok(GpioState::Low)  }
    }
}

// ── Timer — backed by std::time on Linux ─────────────────────────────────────

/// Linux timer epoch — first call to timer_ticks() sets this.
static TIMER_START: core::sync::atomic::AtomicU64 = core::sync::atomic::AtomicU64::new(0);

extern "C" {
    fn clock_gettime(clk_id: i32, tp: *mut TimeSpec) -> i32;
}

#[repr(C)]
struct TimeSpec { tv_sec: i64, tv_nsec: i64 }

const CLOCK_MONOTONIC: i32 = 1;

fn monotonic_ns() -> u64 {
    let mut ts = TimeSpec { tv_sec: 0, tv_nsec: 0 };
    unsafe { clock_gettime(CLOCK_MONOTONIC, &mut ts); }
    ts.tv_sec as u64 * 1_000_000_000 + ts.tv_nsec as u64
}

impl PalTimer for LinuxHwPal {
    fn timer_ticks() -> AxonResult<TimerTicks> {
        let now = monotonic_ns();
        let start = TIMER_START.load(core::sync::atomic::Ordering::Relaxed);
        if start == 0 {
            TIMER_START.store(now, core::sync::atomic::Ordering::Relaxed);
            return AxonResult::Ok(TimerTicks(0));
        }
        AxonResult::Ok(TimerTicks(now.saturating_sub(start)))
    }
    fn timer_freq_hz() -> u64 { 1_000_000_000 } // nanosecond resolution
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tp39_linux_uart_init_ok() {
        assert!(LinuxHwPal::uart_init(0, UartConfig::DEFAULT).is_ok());
    }

    #[test]
    fn tp39_linux_uart_write_byte_ok() {
        assert!(LinuxHwPal::uart_write_byte(0, b'A').is_ok());
    }

    #[test]
    fn tp39_linux_uart_read_byte_none() {
        assert_eq!(LinuxHwPal::uart_read_byte(0).unwrap(), None);
    }

    #[test]
    fn tp39_linux_uart_write_buf() {
        let n = LinuxHwPal::uart_write(0, b"hello").unwrap();
        assert_eq!(n, 5);
    }

    #[test]
    fn tp39_linux_gpio_write_read() {
        LinuxHwPal::gpio_write(5, GpioState::High).unwrap();
        assert_eq!(LinuxHwPal::gpio_read(5).unwrap(), GpioState::High);
        LinuxHwPal::gpio_write(5, GpioState::Low).unwrap();
        assert_eq!(LinuxHwPal::gpio_read(5).unwrap(), GpioState::Low);
    }

    #[test]
    fn tp39_linux_gpio_toggle() {
        LinuxHwPal::gpio_write(6, GpioState::Low).unwrap();
        LinuxHwPal::gpio_toggle(6).unwrap();
        assert_eq!(LinuxHwPal::gpio_read(6).unwrap(), GpioState::High);
        LinuxHwPal::gpio_toggle(6).unwrap();
        assert_eq!(LinuxHwPal::gpio_read(6).unwrap(), GpioState::Low);
    }

    #[test]
    fn tp39_linux_gpio_pin_out_of_range() {
        assert!(LinuxHwPal::gpio_write(64, GpioState::High).is_err());
        assert!(LinuxHwPal::gpio_read(64).is_err());
    }

    #[test]
    fn tp39_linux_timer_ticks_monotonic() {
        let t0 = LinuxHwPal::timer_ticks().unwrap();
        let t1 = LinuxHwPal::timer_ticks().unwrap();
        assert!(t1 >= t0);
    }

    #[test]
    fn tp39_linux_timer_freq() {
        assert_eq!(LinuxHwPal::timer_freq_hz(), 1_000_000_000);
    }

    #[test]
    fn tp39_linux_ticks_to_us() {
        // 1_000_000_000 ticks/sec → 1_000_000 ticks = 1ms = 1000us
        assert_eq!(LinuxHwPal::ticks_to_us(1_000_000), 1000);
    }
}
