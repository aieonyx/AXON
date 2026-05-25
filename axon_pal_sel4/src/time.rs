//! PalTime for seL4 — timer server IPC.
use axon_core::prelude::*;
use axon_pal::{traits::PalTime, types::{Duration, SystemTime}};
use crate::Sel4Pal;

/// Timer server endpoint capability slot.


impl PalTime for Sel4Pal {
    fn now_monotonic() -> AxonResult<Duration> {
        AxonResult::Err(AxonError::not_implemented("seL4 Time: requires timer server IPC"))
    }
    fn now_system() -> AxonResult<SystemTime> {
        AxonResult::Err(AxonError::not_implemented("seL4 Time: requires timer server IPC"))
    }
    fn sleep(duration: Duration) -> AxonResult<()> {
        // Busy-wait approximation using seL4_Yield.
        // Real impl: timer server IPC with timeout notification.
        let _ = duration;
        unsafe { crate::sel4::syscall::sel4_yield(); }
        AxonResult::Err(AxonError::not_implemented("seL4 Time: sleep requires timer server"))
    }
    fn process_start_time() -> AxonResult<SystemTime> {
        AxonResult::Err(AxonError::not_implemented("seL4 Time: requires timer server IPC"))
    }
}
