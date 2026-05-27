//! PalSync for seL4 — Notification objects (mutex) + TCB (threads).

use axon_core::prelude::*;
use axon_pal::{traits::PalSync, types::RawHandle};

impl PalSync for Sel4Pal {
    fn mutex_new() -> AxonResult<RawHandle> {
        // A seL4 mutex is a Notification object capability.
        // The caller must have pre-allocated a Notification cap.
        // For now, return a sentinel — real impl requires CNode allocation.
        AxonResult::Err(AxonError::not_implemented("seL4 Sync: mutex requires Notification cap allocation"))
    }

    fn mutex_lock(handle: RawHandle) -> AxonResult<()> {
        if handle.is_invalid() { return AxonResult::Err(AxonError::invalid_input("invalid handle")); }
        // seL4_Wait on the Notification object — blocks until signalled.
        // Safety: handle.0 is a valid Notification capability slot.
        unsafe { syscall::sel4_wait(handle.0 as usize); }
        AxonResult::Ok(())
    }

    fn mutex_unlock(handle: RawHandle) -> AxonResult<()> {
        if handle.is_invalid() { return AxonResult::Err(AxonError::invalid_input("invalid handle")); }
        // seL4_Signal on the Notification object — wakes a waiter.
        // Safety: handle.0 is a valid Notification capability slot.
        unsafe { syscall::sel4_signal(handle.0 as usize); }
        AxonResult::Ok(())
    }

    fn mutex_destroy(_handle: RawHandle) -> AxonResult<()> {
        // seL4 caps are revoked via CNode_Delete — deferred to capability manager.
        AxonResult::Err(AxonError::not_implemented("seL4 Sync: cap revocation via CNode_Delete"))
    }

    fn thread_spawn(_f: fn()) -> AxonResult<RawHandle> {
        // seL4 threads require: TCB cap alloc + VSpace + IPC buffer setup + TCB_Configure.
        // This is a multi-step operation requiring the capability broker.
        AxonResult::Err(AxonError::not_implemented("seL4 Sync: thread requires TCB + VSpace setup"))
    }

    fn thread_join(_handle: RawHandle) -> AxonResult<()> {
        // seL4 has no native join — requires a Notification rendezvous.
        AxonResult::Err(AxonError::not_implemented("seL4 Sync: join requires Notification rendezvous"))
    }

    fn thread_yield() {
        // Safety: seL4_Yield is always safe — gives up the timeslice.
        unsafe { syscall::sel4_yield(); }
    }
}
