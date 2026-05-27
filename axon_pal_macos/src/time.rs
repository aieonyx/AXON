#![allow(clippy::all, dead_code, unused_imports, unused_variables)]
use axon_core::prelude::*;
use axon_pal::{traits::PalTime, types::{Duration, SystemTime}};
use crate::{MacOsPal, error::last_os_axon_error};

impl PalTime for MacOsPal {
    fn now_monotonic() -> AxonResult<Duration> {
        let mut ts: libc::timespec = unsafe { core::mem::zeroed() };
        let ret = unsafe { libc::clock_gettime(libc::CLOCK_MONOTONIC, &mut ts) };
        if ret < 0 { return AxonResult::Err(last_os_axon_error()); }
        AxonResult::Ok(Duration::new(ts.tv_sec as u64, ts.tv_nsec as u32))
    }
    fn now_system() -> AxonResult<SystemTime> {
        let mut ts: libc::timespec = unsafe { core::mem::zeroed() };
        let ret = unsafe { libc::clock_gettime(libc::CLOCK_REALTIME, &mut ts) };
        if ret < 0 { return AxonResult::Err(last_os_axon_error()); }
        AxonResult::Ok(SystemTime(ts.tv_sec as u64 * 1_000_000_000 + ts.tv_nsec as u64))
    }
    fn sleep(duration: Duration) -> AxonResult<()> {
        let req = libc::timespec {
            tv_sec:  duration.secs as libc::time_t,
            tv_nsec: duration.nanos as libc::c_long,
        };
        let mut rem: libc::timespec = unsafe { core::mem::zeroed() };
        let ret = unsafe { libc::nanosleep(&req, &mut rem) };
        if ret < 0 { AxonResult::Err(last_os_axon_error()) } else { AxonResult::Ok(()) }
    }
    fn process_start_time() -> AxonResult<SystemTime> { Self::now_system() }
}
