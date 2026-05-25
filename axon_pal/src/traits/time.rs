use axon_core::prelude::*;
use crate::types::{Duration, SystemTime};
pub trait PalTime {
    fn now_monotonic() -> AxonResult<Duration>;
    fn now_system() -> AxonResult<SystemTime>;
    fn sleep(duration: Duration) -> AxonResult<()>;
    fn process_start_time() -> AxonResult<SystemTime>;
}
