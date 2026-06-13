// Copyright (c) 2026 Edison Lepiten / AIEONYX
//! Phase 39 integration tests — Driver PAL (UART, GPIO, Timer).

use axon_pal::traits::hw::{
    PalUart, PalGpio, PalTimer, UartConfig, GpioState, TimerTicks,
};
use axon_pal_linux::hw::LinuxHwPal;

#[test]
fn p39_uart_init_and_write() {
    assert!(LinuxHwPal::uart_init(0, UartConfig::DEFAULT).is_ok());
    let n = LinuxHwPal::uart_write(0, b"sovereign").unwrap();
    assert_eq!(n, 9);
}

#[test]
fn p39_uart_read_returns_none_on_host() {
    assert_eq!(LinuxHwPal::uart_read_byte(0).unwrap(), None);
}

#[test]
fn p39_gpio_write_read_roundtrip() {
    LinuxHwPal::gpio_write(10, GpioState::High).unwrap();
    assert!(LinuxHwPal::gpio_read(10).unwrap().is_high());
    LinuxHwPal::gpio_write(10, GpioState::Low).unwrap();
    assert!(LinuxHwPal::gpio_read(10).unwrap().is_low());
}

#[test]
fn p39_gpio_toggle_cycles() {
    LinuxHwPal::gpio_write(11, GpioState::Low).unwrap();
    for _ in 0..4 {
        LinuxHwPal::gpio_toggle(11).unwrap();
    }
    // 4 toggles from Low → Low again
    assert!(LinuxHwPal::gpio_read(11).unwrap().is_low());
}

#[test]
fn p39_timer_ticks_advance() {
    let t0 = LinuxHwPal::timer_ticks().unwrap();
    let t1 = LinuxHwPal::timer_ticks().unwrap();
    assert!(t1 >= t0);
}

#[test]
fn p39_timer_ticks_to_us() {
    assert_eq!(LinuxHwPal::ticks_to_us(1_000_000), 1000);
}

#[test]
fn p39_gpio_out_of_range_error() {
    assert!(LinuxHwPal::gpio_write(64, GpioState::High).is_err());
}
