use axon_core::prelude::*;
use axon_pal::{traits::PalTime, types::{Duration, SystemTime}};
use crate::{LinuxPal, error::errno_to_axon_error};

impl PalTime for LinuxPal {
    fn now_monotonic() -> AxonResult<Duration> {
        let mut ts: libc::timespec = unsafe { core::mem::zeroed() };
        let ret = unsafe { libc::clock_gettime(libc::CLOCK_MONOTONIC, &mut ts) };
        if ret < 0 { return AxonResult::Err(errno_to_axon_error()); }
        AxonResult::Ok(Duration::new(ts.tv_sec as u64, ts.tv_nsec as u32))
    }
    fn now_system() -> AxonResult<SystemTime> {
        let mut ts: libc::timespec = unsafe { core::mem::zeroed() };
        let ret = unsafe { libc::clock_gettime(libc::CLOCK_REALTIME, &mut ts) };
        if ret < 0 { return AxonResult::Err(errno_to_axon_error()); }
        AxonResult::Ok(SystemTime(ts.tv_sec as u64 * 1_000_000_000 + ts.tv_nsec as u64))
    }
    fn sleep(duration: Duration) -> AxonResult<()> {
        let req = libc::timespec { tv_sec: duration.secs as libc::time_t, tv_nsec: duration.nanos as libc::c_long };
        let mut rem: libc::timespec = unsafe { core::mem::zeroed() };
        let ret = unsafe { libc::nanosleep(&req, &mut rem) };
        if ret < 0 { AxonResult::Err(errno_to_axon_error()) } else { AxonResult::Ok(()) }
    }
    fn process_start_time() -> AxonResult<SystemTime> { Self::now_system() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axon_pal::traits::PalTime;
    #[test] fn time_monotonic_is_positive() { let t = LinuxPal::now_monotonic().unwrap(); assert!(t.secs > 0 || t.nanos > 0); }
    #[test] fn time_monotonic_increases() {
        let t1 = LinuxPal::now_monotonic().unwrap();
        for _ in 0..10_000_000u64 { core::hint::black_box(0u8); }
        let t2 = LinuxPal::now_monotonic().unwrap(); assert!(t2 >= t1);
    }
    #[test] fn time_system_is_after_unix_epoch() { assert!(LinuxPal::now_system().unwrap().0 > 1_577_836_800_000_000_000); }
    #[test] fn time_sleep_short() {
        let t1 = LinuxPal::now_monotonic().unwrap();
        LinuxPal::sleep(Duration::from_millis(1)).unwrap();
        let t2 = LinuxPal::now_monotonic().unwrap();
        assert!(t2.as_millis() >= t1.as_millis() + 1);
    }
    #[test] fn time_duration_since_now() {
        let t1 = LinuxPal::now_system().unwrap();
        LinuxPal::sleep(Duration::from_millis(1)).unwrap();
        let t2 = LinuxPal::now_system().unwrap();
        assert!(t2.duration_since(t1).unwrap().as_millis() >= 1);
    }
}
