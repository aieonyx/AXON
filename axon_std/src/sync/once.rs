// Copyright (c) 2026 Edison Lepitel / AIEONYX
// SPDX-License-Identifier: Apache-2.0
// sync/once.rs -- Sovereign Once: one-time initialisation barrier.
// Used for lazy static init, capability registration, key ceremony results.

use std::sync::Once as StdOnce;

/// Sovereign one-time initialisation barrier.
/// Guarantees a closure runs exactly once, even under concurrent callers.
pub struct SovereignOnce {
    inner: StdOnce,
}

impl SovereignOnce {
    pub const fn new() -> Self {
        SovereignOnce { inner: StdOnce::new() }
    }

    /// Run `f` exactly once. Subsequent calls are no-ops.
    pub fn call_once<F: FnOnce()>(&self, f: F) {
        self.inner.call_once(f);
    }

    /// Returns true if `call_once` has already completed.
    pub fn is_completed(&self) -> bool {
        self.inner.is_completed()
    }
}

impl Default for SovereignOnce {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, atomic::{AtomicU32, Ordering}};
    use std::thread;

    #[test]
    fn test_once_runs_exactly_once() {
        let once = SovereignOnce::new();
        let count = Arc::new(AtomicU32::new(0));
        let c = count.clone();
        once.call_once(|| { c.fetch_add(1, Ordering::SeqCst); });
        once.call_once(|| { c.fetch_add(1, Ordering::SeqCst); });
        assert_eq!(count.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn test_once_is_completed() {
        let once = SovereignOnce::new();
        assert!(!once.is_completed());
        once.call_once(|| {});
        assert!(once.is_completed());
    }

    #[test]
    fn test_once_threaded() {
        let once = Arc::new(SovereignOnce::new());
        let count = Arc::new(AtomicU32::new(0));
        let mut handles = vec![];
        for _ in 0..8 {
            let oc = once.clone();
            let cc = count.clone();
            handles.push(thread::spawn(move || {
                oc.call_once(|| { cc.fetch_add(1, Ordering::SeqCst); });
            }));
        }
        for h in handles { h.join().unwrap(); }
        assert_eq!(count.load(Ordering::SeqCst), 1);
    }
}
