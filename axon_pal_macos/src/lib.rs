#![allow(clippy::all, dead_code, unused_imports, unused_variables)]
#![cfg_attr(not(target_os = "macos"), allow(clippy::all, dead_code, unused_imports))]
//! # axon_pal_macos
//!
//! AXON PAL — macOS Darwin XNU implementation.
//!
//! macOS is POSIX-compatible. This PAL mirrors axon_pal_linux with
//! Darwin-specific differences:
//!
//! | Difference | Linux | macOS |
//! |---|---|---|
//! | errno access | `__errno_location()` | `std::io::Error::last_os_error()` |
//! | flush | `fdatasync(2)` | `fcntl(F_FULLFSYNC)` |
//! | monotonic clock | `CLOCK_MONOTONIC` | `CLOCK_MONOTONIC` (10.12+) |
//!
//! All other POSIX interfaces (pthreads, sockets, stat, etc.) are identical.

#![allow(missing_docs)]

mod error;
mod fs;
mod io;
mod net;
mod process;
mod sync;
mod time;

pub struct MacOsPal;
