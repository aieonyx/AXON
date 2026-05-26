use axon_core::prelude::*;
use axon_pal::{traits::PalSync, types::RawHandle};
use crate::WindowsPal;

impl PalSync for WindowsPal {
    fn mutex_new() -> AxonResult<RawHandle> {
        let m: Box<std::sync::Mutex<()>> = Box::new(std::sync::Mutex::new(()));
        AxonResult::Ok(RawHandle(Box::into_raw(m) as u64))
    }
    fn mutex_lock(h: RawHandle) -> AxonResult<()> {
        if h.is_invalid() { return AxonResult::Err(AxonError::invalid_input("invalid handle")); }
        let m = unsafe { &*(h.0 as *const std::sync::Mutex<()>) };
        match m.lock() {
            Ok(_guard) => AxonResult::Ok(()),
            Err(_)     => AxonResult::Err(AxonError::io("mutex poisoned")),
        }
    }
    fn mutex_unlock(_h: RawHandle) -> AxonResult<()> {
        AxonResult::Ok(()) // MutexGuard drops automatically
    }
    fn mutex_destroy(h: RawHandle) -> AxonResult<()> {
        if h.is_invalid() { return AxonResult::Err(AxonError::invalid_input("invalid handle")); }
        let _ = unsafe { Box::from_raw(h.0 as *mut std::sync::Mutex<()>) };
        AxonResult::Ok(())
    }
    fn thread_spawn(f: fn()) -> AxonResult<RawHandle> {
        let handle = std::thread::spawn(move || f());
        let h = Box::into_raw(Box::new(handle)) as u64;
        AxonResult::Ok(RawHandle(h))
    }
    fn thread_join(h: RawHandle) -> AxonResult<()> {
        if h.is_invalid() { return AxonResult::Err(AxonError::invalid_input("invalid handle")); }
        let handle = unsafe { Box::from_raw(h.0 as *mut std::thread::JoinHandle<()>) };
        match handle.join() {
            Ok(_)  => AxonResult::Ok(()),
            Err(_) => AxonResult::Err(AxonError::io("thread panicked")),
        }
    }
    fn thread_yield() { std::thread::yield_now(); }
}
