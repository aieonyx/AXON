use axon_core::prelude::*;
use axon_pal::{traits::PalSync, types::RawHandle};
use crate::LinuxPal;

impl PalSync for LinuxPal {
    fn mutex_new() -> AxonResult<RawHandle> {
        let mutex = Box::new(unsafe {
            let mut m: libc::pthread_mutex_t = libc::PTHREAD_MUTEX_INITIALIZER;
            let ret = libc::pthread_mutex_init(&mut m, core::ptr::null());
            if ret != 0 { return AxonResult::Err(AxonError::io("pthread_mutex_init failed").with_code(ret as u32)); }
            m
        });
        AxonResult::Ok(RawHandle(Box::into_raw(mutex) as u64))
    }
    fn mutex_lock(h: RawHandle) -> AxonResult<()> {
        if h.is_invalid() { return AxonResult::Err(AxonError::invalid_input("invalid mutex handle")); }
        let ret = unsafe { libc::pthread_mutex_lock(h.0 as *mut libc::pthread_mutex_t) };
        if ret != 0 { AxonResult::Err(AxonError::io("pthread_mutex_lock failed").with_code(ret as u32)) } else { AxonResult::Ok(()) }
    }
    fn mutex_unlock(h: RawHandle) -> AxonResult<()> {
        if h.is_invalid() { return AxonResult::Err(AxonError::invalid_input("invalid mutex handle")); }
        let ret = unsafe { libc::pthread_mutex_unlock(h.0 as *mut libc::pthread_mutex_t) };
        if ret != 0 { AxonResult::Err(AxonError::io("pthread_mutex_unlock failed").with_code(ret as u32)) } else { AxonResult::Ok(()) }
    }
    fn mutex_destroy(h: RawHandle) -> AxonResult<()> {
        if h.is_invalid() { return AxonResult::Err(AxonError::invalid_input("invalid mutex handle")); }
        let mutex = unsafe { Box::from_raw(h.0 as *mut libc::pthread_mutex_t) };
        let ret = unsafe { libc::pthread_mutex_destroy(Box::into_raw(mutex)) };
        if ret != 0 { AxonResult::Err(AxonError::io("pthread_mutex_destroy failed").with_code(ret as u32)) } else { AxonResult::Ok(()) }
    }
    fn thread_spawn(f: fn()) -> AxonResult<RawHandle> {
        extern "C" fn trampoline(arg: *mut libc::c_void) -> *mut libc::c_void {
            let f: fn() = unsafe { core::mem::transmute(arg) };
            f(); core::ptr::null_mut()
        }
        let mut tid: libc::pthread_t = 0;
        let ret = unsafe { libc::pthread_create(&mut tid, core::ptr::null(), trampoline, f as *mut libc::c_void) };
        if ret != 0 { AxonResult::Err(AxonError::io("pthread_create failed").with_code(ret as u32)) }
        else         { AxonResult::Ok(RawHandle(tid as u64)) }
    }
    fn thread_join(h: RawHandle) -> AxonResult<()> {
        let ret = unsafe { libc::pthread_join(h.0 as libc::pthread_t, core::ptr::null_mut()) };
        if ret != 0 { AxonResult::Err(AxonError::io("pthread_join failed").with_code(ret as u32)) } else { AxonResult::Ok(()) }
    }
    fn thread_yield() { unsafe { libc::sched_yield(); } }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axon_pal::traits::PalSync;
    use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
    #[test] fn sync_mutex_lock_unlock() {
        let h = LinuxPal::mutex_new().unwrap();
        LinuxPal::mutex_lock(h).unwrap(); LinuxPal::mutex_unlock(h).unwrap(); LinuxPal::mutex_destroy(h).unwrap();
    }
    #[test] fn sync_thread_yield_noop() { LinuxPal::thread_yield(); }
    #[test] fn sync_thread_spawn_and_join() {
        static RAN: AtomicBool = AtomicBool::new(false);
        fn worker() { RAN.store(true, Ordering::SeqCst); }
        let h = LinuxPal::thread_spawn(worker).unwrap();
        LinuxPal::thread_join(h).unwrap();
        assert!(RAN.load(Ordering::SeqCst));
    }
    #[test] fn sync_multiple_threads() {
        static COUNTER: AtomicU32 = AtomicU32::new(0);
        fn inc() { COUNTER.fetch_add(1, Ordering::SeqCst); }
        let h1 = LinuxPal::thread_spawn(inc).unwrap();
        let h2 = LinuxPal::thread_spawn(inc).unwrap();
        LinuxPal::thread_join(h1).unwrap(); LinuxPal::thread_join(h2).unwrap();
        assert_eq!(COUNTER.load(Ordering::SeqCst), 2);
    }
    #[test] fn sync_invalid_handle_returns_err() {
        assert!(LinuxPal::mutex_lock(RawHandle::INVALID).is_err());
        assert!(LinuxPal::mutex_unlock(RawHandle::INVALID).is_err());
        assert!(LinuxPal::mutex_destroy(RawHandle::INVALID).is_err());
    }
}
