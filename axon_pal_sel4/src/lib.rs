//! # axon_pal_sel4
//!
//! AXON Platform Abstraction Layer — seL4 aarch64 implementation.
//!
//! Uses a sovereign, zero-dependency seL4 ABI layer built directly
//! into this crate. No third-party seL4 crate required.
//!
//! ## Architecture
//!
//! seL4 has no syscalls in the POSIX sense. Everything goes through
//! IPC capability invocations. The AXON PAL maps:
//!
//! | PAL trait | seL4 mechanism |
//! |---|---|
//! | PalIo | IPC to console server PD (CPtr 16) |
//! | PalFs | IPC to filesystem server PD (CPtr 17) |
//! | PalNet | IPC to network server PD |
//! | PalSync | Notification objects + TCB |
//! | PalTime | IPC to timer server PD (CPtr 18) |
//! | PalProcess | TCB capability space |
//!
//! ## Status
//!
//! IPC infrastructure: complete (syscall wrappers, MR packing, MessageInfo).
//! PalIo write/flush: complete — calls console server via seL4_Call.
//! PalSync lock/unlock/yield: complete — Notification + seL4_Yield.
//! PalProcess args/pid/exit: complete — sovereign seL4 semantics.
//! PalFs, PalNet, PalTime: stub — require server PD setup (Phase 7).

#![no_std]
#![allow(missing_docs)]

pub mod sel4;
mod fs;
mod io;
mod net;
mod process;
mod sync;
mod time;

/// The seL4 aarch64 PAL implementation.
pub struct Sel4Pal;
