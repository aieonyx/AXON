// ============================================================
// axon_rt — AXON Runtime
// Copyright © 2026 Edison Lepiten — AIEONYX
// github.com/aieonyx/axon
//
// Provides runtime support for AXON-transpiled Rust programs:
//   - defer()     — RAII scope guard (AXON 'defer' statement)
//   - Deferred    — the guard type
//   - with scopes — handled via Rust's own drop semantics
//
// This crate is automatically linked into all AXON programs.
// It is intentionally minimal — no_std compatible by design.
// ============================================================

// ── Defer Guard ───────────────────────────────────────────────
//
// AXON:
//   let@ channel = ipc.open_channel()?
//   defer channel.close()
//
// Generated Rust:
//   let channel = ipc.open_channel()?;
//   let _guard = axon_rt::defer(|| channel.close());
//
// When `_guard` goes out of scope (any exit path), it calls
// channel.close() — matching AXON's defer semantics exactly.

/// A scope guard that executes a closure when dropped.
/// Created by the `defer` function — never construct directly.
pub struct Deferred<F: FnOnce()> {
    action: Option<F>,
}

impl<F: FnOnce()> Drop for Deferred<F> {
    fn drop(&mut self) {
        if let Some(f) = self.action.take() {
            f();
        }
    }
}

/// Create a deferred action that runs when the returned guard
/// goes out of scope. Equivalent to AXON's `defer expr` statement.
///
/// # Example
/// ```rust
/// let _guard = axon_rt::defer(|| println!("cleaned up"));
/// // "cleaned up" prints when _guard is dropped
/// ```
pub fn defer<F: FnOnce()>(action: F) -> Deferred<F> {
    Deferred { action: Some(action) }
}

// ── Capability Stubs ──────────────────────────────────────────
// Placeholder until axon_std implements real capabilities.
// Phase 3 stubs — Phase 5 replaces with seL4 capability calls.

/// Mark a value as capability-pinned (stub for Phase 3)
#[inline(always)]
pub fn cap_pin<T>(value: T) -> T { value }

// ── Provenance Stubs ──────────────────────────────────────────

/// Mark data as tainted (stub — Phase 5 enforces at type level)
#[inline(always)]
pub fn taint<T>(value: T) -> T { value }

/// Assert data is clean (stub — Phase 5 verifies statically)
#[inline(always)]
pub fn clean<T>(value: T) -> T { value }

// ── Tests ─────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    #[test]
    fn test_defer_runs_on_drop() {
        let ran = Arc::new(Mutex::new(false));
        let ran_clone = ran.clone();
        {
            let _guard = defer(move || {
                *ran_clone.lock().unwrap() = true;
            });
            assert!(!*ran.lock().unwrap(), "should not have run yet");
        }
        assert!(*ran.lock().unwrap(), "defer should have run on drop");
    }

    #[test]
    fn test_defer_runs_on_early_return() {
        let ran = Arc::new(Mutex::new(false));
        let ran_clone = ran.clone();

        fn early(ran: Arc<Mutex<bool>>) {
            let _guard = defer(move || {
                *ran.lock().unwrap() = true;
            });
            return; // early exit
        }

        early(ran_clone);
        assert!(*ran.lock().unwrap(), "defer should run on early return");
    }

    #[test]
    fn test_defer_lifo_order() {
        let order = Arc::new(Mutex::new(Vec::new()));
        {
            let o1 = order.clone();
            let _g1 = defer(move || o1.lock().unwrap().push(1));
            let o2 = order.clone();
            let _g2 = defer(move || o2.lock().unwrap().push(2));
            let o3 = order.clone();
            let _g3 = defer(move || o3.lock().unwrap().push(3));
        }
        // LIFO — last deferred runs first
        assert_eq!(*order.lock().unwrap(), vec![3, 2, 1]);
    }

    #[test]
    fn test_defer_with_axon_classify_pattern() {
        // Simulate the Aegis Monitor pattern:
        //   let@ channel = open()?
        //   defer channel.close()
        let closed = Arc::new(Mutex::new(false));
        let closed_clone = closed.clone();

        struct FakeChannel { closed: Arc<Mutex<bool>> }
        impl FakeChannel {
            fn close(self) { *self.closed.lock().unwrap() = true; }
        }

        let channel = FakeChannel { closed: closed_clone };
        {
            let _guard = defer(move || channel.close());
            // channel is in use here
            assert!(!*closed.lock().unwrap());
        }
        assert!(*closed.lock().unwrap(), "channel should be closed after scope");
    }
}
