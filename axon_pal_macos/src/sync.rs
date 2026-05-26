use axon_core::prelude::*;
use axon_pal::{traits::PalSync, types::RawHandle};
use crate::MacOsPal;

impl PalSync for MacOsPal {
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
        if h.is_invalid() { return AxonResult::Err(AxonError::invalid_input("invalid handle")); }
        let ret = unsafe { libc::pthread_mutex_lock(h.0 as *mut libc::pthread_mutex_t) };
        if ret != 0 { AxonResult::Err(AxonError::io("pthread_mutex_lock failed").with_code(ret as u32)) } else { AxonResult::Ok(()) }
    }
    fn mutex_unlock(h: RawHandle) -> AxonResult<()> {
        if h.is_invalid() { return AxonResult::Err(AxonError::invalid_input("invalid handle")); }
        let ret = unsafe { libc::pthread_mutex_unlock(h.0 as *mut libc::pthread_mutex_t) };
        if ret != 0 { AxonResult::Err(AxonError::io("pthread_mutex_unlock failed").with_code(ret as u32)) } else { AxonResult::Ok(()) }
    }
    fn mutex_destroy(h: RawHandle) -> AxonResult<()> {
        if h.is_invalid() { return AxonResult::Err(AxonError::invalid_input("invalid handle")); }
        let mutex = unsafe { Box::from_raw(h.0 as *mut libc::pthread_mutex_t) };
        let ret = unsafe { libc::pthread_mutex_destroy(Box::into_raw(mutex)) };
        if ret != 0 { AxonResult::Err(AxonError::io("pthread_mutex_destroy failed").with_code(ret as u32)) } else { AxonResult::Ok(()) }
    }
    fn thread_spawn(f: fn()) -> AxonResult<RawHandle> {
        extern "C" fn trampoline(arg: *mut libc::c_void) -> *mut libc::c_void {
            let f: fn() = unsafe { core::mem::transmute(arg) };
            f(); core::ptr::null_mut()
        }
        let mut tid: libc::pthread_t = unsafe { core::mem::zeroed() };
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
