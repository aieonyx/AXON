// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
// sync/channel.rs -- Sovereign Channel<T>: bounded MPSC queue.
// Maps naturally to seL4 IPC endpoints — one sender slot, one receiver.
// Thread-safe via Mutex<VecDeque>. Bounded to prevent runaway allocation.
// P axon_std: sync primitive for seL4 IPC bridge.

use std::collections::VecDeque;
use std::sync::{Arc, Condvar, Mutex};

/// Error type for channel operations.
#[derive(Debug, Clone, PartialEq)]
pub enum ChannelError {
    /// Channel is full — send would exceed capacity.
    Full,
    /// Channel is closed — no more senders exist.
    Closed,
    /// Receive timed out (future: with timeout support).
    Empty,
}

impl std::fmt::Display for ChannelError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ChannelError::Full   => write!(f, "channel full"),
            ChannelError::Closed => write!(f, "channel closed"),
            ChannelError::Empty  => write!(f, "channel empty"),
        }
    }
}

/// Internal shared state between Sender and Receiver.
struct Inner<T> {
    queue:    VecDeque<T>,
    capacity: usize,
    closed:   bool,
}

/// Sending half of a sovereign channel.
/// Clone to create multiple senders (MPSC).
#[derive(Clone)]
pub struct Sender<T> {
    inner:  Arc<(Mutex<Inner<T>>, Condvar)>,
}

/// Receiving half of a sovereign channel.
/// Only one receiver per channel (single consumer).
pub struct Receiver<T> {
    inner: Arc<(Mutex<Inner<T>>, Condvar)>,
}

/// Create a bounded sovereign channel with given capacity.
/// Analogous to opening a seL4 IPC endpoint with a message buffer.
pub fn channel<T>(capacity: usize) -> (Sender<T>, Receiver<T>) {
    assert!(capacity > 0, "channel capacity must be > 0");
    let inner = Arc::new((
        Mutex::new(Inner {
            queue:    VecDeque::with_capacity(capacity),
            capacity,
            closed:   false,
        }),
        Condvar::new(),
    ));
    (Sender { inner: inner.clone() }, Receiver { inner })
}

impl<T> Sender<T> {
    /// Send a value. Returns Err(Full) if capacity exceeded.
    /// Non-blocking — sovereign systems prefer explicit back-pressure.
    pub fn send(&self, val: T) -> Result<(), ChannelError> {
        let (lock, cvar) = &*self.inner;
        let mut state = lock.lock().unwrap();
        if state.closed { return Err(ChannelError::Closed); }
        if state.queue.len() >= state.capacity { return Err(ChannelError::Full); }
        state.queue.push_back(val);
        cvar.notify_one();
        Ok(())
    }

    /// Close this sender. When all senders are dropped, channel is closed.
    pub fn close(&self) {
        let (lock, cvar) = &*self.inner;
        let mut state = lock.lock().unwrap();
        state.closed = true;
        cvar.notify_all();
    }

    /// Returns true if the channel is at capacity.
    pub fn is_full(&self) -> bool {
        let (lock, _) = &*self.inner;
        let state = lock.lock().unwrap();
        state.queue.len() >= state.capacity
    }

    /// Current number of items in the channel.
    pub fn len(&self) -> usize {
        let (lock, _) = &*self.inner;
        lock.lock().unwrap().queue.len()
    }
}

impl<T> Receiver<T> {
    /// Try to receive a value without blocking.
    /// Returns Err(Empty) if nothing available.
    pub fn try_recv(&self) -> Result<T, ChannelError> {
        let (lock, _) = &*self.inner;
        let mut state = lock.lock().unwrap();
        state.queue.pop_front().ok_or(ChannelError::Empty)
    }

    /// Block until a value is available or channel is closed.
    pub fn recv(&self) -> Result<T, ChannelError> {
        let (lock, cvar) = &*self.inner;
        let mut state = lock.lock().unwrap();
        loop {
            if let Some(val) = state.queue.pop_front() {
                return Ok(val);
            }
            if state.closed { return Err(ChannelError::Closed); }
            state = cvar.wait(state).unwrap();
        }
    }

    /// Returns true if channel has pending messages.
    pub fn has_messages(&self) -> bool {
        let (lock, _) = &*self.inner;
        !lock.lock().unwrap().queue.is_empty()
    }

    /// Drain all pending messages.
    pub fn drain(&self) -> Vec<T> {
        let (lock, _) = &*self.inner;
        let mut state = lock.lock().unwrap();
        state.queue.drain(..).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_channel_send_recv() {
        let (tx, rx) = channel::<i32>(4);
        tx.send(42).unwrap();
        assert_eq!(rx.try_recv().unwrap(), 42);
    }

    #[test]
    fn test_channel_fifo_order() {
        let (tx, rx) = channel::<i32>(8);
        for i in 0..5 { tx.send(i).unwrap(); }
        for i in 0..5 { assert_eq!(rx.try_recv().unwrap(), i); }
    }

    #[test]
    fn test_channel_full_returns_err() {
        let (tx, _rx) = channel::<i32>(2);
        tx.send(1).unwrap();
        tx.send(2).unwrap();
        assert_eq!(tx.send(3), Err(ChannelError::Full));
    }

    #[test]
    fn test_channel_empty_returns_err() {
        let (_tx, rx) = channel::<i32>(4);
        assert_eq!(rx.try_recv(), Err(ChannelError::Empty));
    }

    #[test]
    fn test_channel_closed() {
        let (tx, _rx) = channel::<i32>(4);
        tx.close();
        assert_eq!(tx.send(1), Err(ChannelError::Closed));
    }

    #[test]
    fn test_channel_drain() {
        let (tx, rx) = channel::<i32>(8);
        for i in 0..4 { tx.send(i).unwrap(); }
        let drained = rx.drain();
        assert_eq!(drained, vec![0,1,2,3]);
        assert!(!rx.has_messages());
    }

    #[test]
    fn test_channel_threaded_send_recv() {
        let (tx, rx) = channel::<u32>(16);
        let handle = thread::spawn(move || {
            for i in 0..8u32 { tx.send(i).unwrap(); }
        });
        handle.join().unwrap();
        for i in 0..8u32 {
            assert_eq!(rx.try_recv().unwrap(), i);
        }
    }

    #[test]
    fn test_channel_blocking_recv() {
        let (tx, rx) = channel::<&str>(4);
        let handle = thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_millis(10));
            tx.send("sovereign").unwrap();
        });
        assert_eq!(rx.recv().unwrap(), "sovereign");
        handle.join().unwrap();
    }

    #[test]
    fn test_channel_len() {
        let (tx, rx) = channel::<i32>(8);
        assert_eq!(tx.len(), 0);
        tx.send(1).unwrap();
        tx.send(2).unwrap();
        assert_eq!(tx.len(), 2);
        rx.try_recv().unwrap();
        assert_eq!(tx.len(), 1);
    }

    #[test]
    fn test_channel_is_full() {
        let (tx, _rx) = channel::<i32>(2);
        assert!(!tx.is_full());
        tx.send(1).unwrap();
        tx.send(2).unwrap();
        assert!(tx.is_full());
    }
}
