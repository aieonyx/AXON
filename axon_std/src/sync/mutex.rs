// Copyright (c) 2026 Edison Lepitel / AIEONYX
// SPDX-License-Identifier: Apache-2.0
// sync/mutex.rs -- Sovereign Mutex<T>: auditable mutual exclusion.
// Wraps std::sync::Mutex with sovereign error types and audit hooks.
// Poisoning is treated as a SovereignError — no silent data corruption.

use std::sync::{Mutex as StdMutex, MutexGuard};

/// Sovereign mutex error.
#[derive(Debug, Clone, PartialEq)]
pub enum MutexError {
    /// Lock is poisoned — a thread panicked while holding it.
    Poisoned,
    /// Would block (for try_lock).
    WouldBlock,
}

impl std::fmt::Display for MutexError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            MutexError::Poisoned   => write!(f, "sovereign mutex poisoned"),
            MutexError::WouldBlock => write!(f, "sovereign mutex would block"),
        }
    }
}

/// Sovereign Mutex<T>: mutual exclusion with explicit poison handling.
pub struct SovereignMutex<T> {
    inner: StdMutex<T>,
}

impl<T> SovereignMutex<T> {
    /// Create a new sovereign mutex wrapping value `val`.
    pub fn new(val: T) -> Self {
        SovereignMutex { inner: StdMutex::new(val) }
    }

    /// Acquire the lock, blocking until available.
    /// Returns Err(Poisoned) if a previous holder panicked.
    pub fn lock(&self) -> Result<MutexGuard<T>, MutexError> {
        self.inner.lock().map_err(|_| MutexError::Poisoned)
    }

    /// Try to acquire the lock without blocking.
    pub fn try_lock(&self) -> Result<MutexGuard<T>, MutexError> {
        self.inner.try_lock().map_err(|e| match e {
            std::sync::TryLockError::Poisoned(_) => MutexError::Poisoned,
            std::sync::TryLockError::WouldBlock  => MutexError::WouldBlock,
        })
    }

    /// Returns true if the mutex is poisoned.
    pub fn is_poisoned(&self) -> bool {
        self.inner.is_poisoned()
    }
}

impl<T: std::fmt::Debug> std::fmt::Debug for SovereignMutex<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self.try_lock() {
            Ok(g)  => write!(f, "SovereignMutex({:?})", &*g),
            Err(_) => write!(f, "SovereignMutex(<locked>)"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::thread;

    #[test]
    fn test_mutex_basic() {
        let m = SovereignMutex::new(42i32);
        let g = m.lock().unwrap();
        assert_eq!(*g, 42);
    }

    #[test]
    fn test_mutex_mutation() {
        let m = SovereignMutex::new(0i32);
        { let mut g = m.lock().unwrap(); *g = 99; }
        assert_eq!(*m.lock().unwrap(), 99);
    }

    #[test]
    fn test_mutex_try_lock_would_block() {
        let m = SovereignMutex::new(0i32);
        let _g = m.lock().unwrap();
        assert!(matches!(m.try_lock(), Err(MutexError::WouldBlock)));
    }

    #[test]
    fn test_mutex_not_poisoned_initially() {
        let m = SovereignMutex::new(0i32);
        assert!(!m.is_poisoned());
    }

    #[test]
    fn test_mutex_threaded_counter() {
        let m = Arc::new(SovereignMutex::new(0u32));
        let mut handles = vec![];
        for _ in 0..8 {
            let mc = m.clone();
            handles.push(thread::spawn(move || {
                let mut g = mc.lock().unwrap();
                *g += 1;
            }));
        }
        for h in handles { h.join().unwrap(); }
        assert_eq!(*m.lock().unwrap(), 8);
    }

    #[test]
    fn test_mutex_error_display() {
        assert!(MutexError::Poisoned.to_string().contains("poisoned"));
        assert!(MutexError::WouldBlock.to_string().contains("block"));
    }
}
