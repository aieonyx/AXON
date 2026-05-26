use axon_core::prelude::*;
use axon_pal::{traits::PalTime, types::{Duration, SystemTime}};
use crate::WindowsPal;

impl PalTime for WindowsPal {
    fn now_monotonic() -> AxonResult<Duration> {
        use std::time::Instant;
        // Use a lazy static start time so elapsed() gives us monotonic nanos
        use std::sync::OnceLock;
        static START: OnceLock<Instant> = OnceLock::new();
        let start = START.get_or_init(Instant::now);
        let elapsed = start.elapsed();
        AxonResult::Ok(Duration::new(elapsed.as_secs(), elapsed.subsec_nanos()))
    }
    fn now_system() -> AxonResult<SystemTime> {
        use std::time::{SystemTime as StdST, UNIX_EPOCH};
        let nanos = StdST::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(0);
        AxonResult::Ok(SystemTime(nanos))
    }
    fn sleep(duration: Duration) -> AxonResult<()> {
        std::thread::sleep(std::time::Duration::new(duration.secs, duration.nanos));
        AxonResult::Ok(())
    }
    fn process_start_time() -> AxonResult<SystemTime> { Self::now_system() }
}
