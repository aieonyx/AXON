#![allow(clippy::all)]
//! Raw seL4 syscall wrappers for aarch64.
//!
//! seL4 on aarch64 uses `svc #0` for all kernel entries.
//! Register convention:
//!   x7 = syscall number (negated for LLVM compat)
//!   x0 = arg0 / return value 0
//!   x1 = arg1 / return value 1
//!   ...
//!
//! Reference: seL4 Reference Manual §A.2 (aarch64 syscall ABI)

/// seL4 syscall numbers (aarch64).
pub mod nr {
    pub const CALL:        isize = -1;
    pub const REPLY_RECV:  isize = -2;
    pub const SEND:        isize = -3;
    pub const NBSEND:      isize = -4;
    pub const RECV:        isize = -5;
    pub const YIELD_:      isize = -6;
    pub const NBRECV:      isize = -7;
    pub const SIGNAL:      isize = -8;
}

/// seL4_Yield — give up the current timeslice.
///
/// # Safety
///
/// Safe to call at any time from a seL4 thread.
#[inline]
pub unsafe fn sel4_yield() {
    core::arch::asm!(
        "svc #0",
        in("x7") nr::YIELD_,
        options(nostack, nomem)
    );
}

/// seL4_Send — send a message to a capability (fire-and-forget).
///
/// # Safety
///
/// `dest` must be a valid endpoint capability in the caller's CSpace.
/// The IPC buffer must be set up before calling.
#[inline]
pub unsafe fn sel4_send(dest: usize, msginfo: usize) {
    core::arch::asm!(
        "svc #0",
        in("x7") nr::SEND,
        in("x0") dest,
        in("x1") msginfo,
        options(nostack)
    );
}

/// seL4_Recv — receive a message from an endpoint, blocking.
///
/// Returns (message_info, badge).
///
/// # Safety
///
/// `src` must be a valid endpoint capability. The IPC buffer must be mapped.
#[inline]
pub unsafe fn sel4_recv(src: usize, reply: usize) -> (usize, usize) {
    let msg_info: usize;
    let badge: usize;
    core::arch::asm!(
        "svc #0",
        in("x7") nr::RECV,
        in("x0") src,
        in("x1") reply,
        lateout("x0") msg_info,
        lateout("x1") badge,
        options(nostack)
    );
    (msg_info, badge)
}

/// seL4_Call — send + receive in one kernel entry (RPC pattern).
///
/// Returns the reply MessageInfo.
///
/// # Safety
///
/// `dest` must be a valid endpoint capability. IPC buffer must be mapped.
#[inline]
pub unsafe fn sel4_call(dest: usize, msginfo: usize) -> usize {
    let reply: usize;
    core::arch::asm!(
        "svc #0",
        in("x7") nr::CALL,
        in("x0") dest,
        in("x1") msginfo,
        lateout("x0") reply,
        options(nostack)
    );
    reply
}

/// seL4_Signal — signal a notification object.
///
/// # Safety
///
/// `dest` must be a valid notification capability.
#[inline]
pub unsafe fn sel4_signal(dest: usize) {
    core::arch::asm!(
        "svc #0",
        in("x7") nr::SIGNAL,
        in("x0") dest,
        options(nostack, nomem)
    );
}

/// seL4_Wait — wait on a notification object.
///
/// Returns the notification badge word.
///
/// # Safety
///
/// `src` must be a valid notification capability.
#[inline]
pub unsafe fn sel4_wait(src: usize) -> usize {
    let badge: usize;
    core::arch::asm!(
        "svc #0",
        in("x7") nr::RECV,
        in("x0") src,
        in("x1") 0usize,
        lateout("x0") _,
        lateout("x1") badge,
        options(nostack)
    );
    badge
}
