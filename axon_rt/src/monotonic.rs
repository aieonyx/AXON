// ============================================================
// axon_rt::monotonic — Monotonic clock
// Copyright © 2026 Edison Lepiten — AIEONYX
// SPEC: 6A-01 DWC
//
// Returns nanoseconds elapsed since process start.
// Guaranteed non-decreasing. Never returns 0 in production
// (0 is the trivial sentinel value).
//
// PHASE7: replace with sel4_sys::clock_gettime(CLOCK_MONOTONIC)
//         for the seL4 kernel port.
// ============================================================

use std::sync::OnceLock;
use std::time::Instant;

static EPOCH: OnceLock<Instant> = OnceLock::new();

/// Returns monotonic nanoseconds since first call.
/// Thread-safe. Allocation-free after first call.
/// SPEC: 6A-01
#[inline]
pub fn monotonic_ns() -> u64 {
    let epoch = EPOCH.get_or_init(Instant::now);
    epoch.elapsed().as_nanos() as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_monotonic_non_decreasing() {
        let a = monotonic_ns();
        let b = monotonic_ns();
        assert!(b >= a, "monotonic_ns must be non-decreasing");
    }

    #[test]
    fn test_monotonic_nonzero_after_first() {
        let _ = monotonic_ns(); // init epoch
        std::thread::sleep(std::time::Duration::from_nanos(100));
        let t = monotonic_ns();
        assert!(t > 0, "monotonic_ns must be > 0 after init");
    }
}
