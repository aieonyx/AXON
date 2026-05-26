//! # axon_pal_windows
//!
//! AXON PAL — Windows Win32 implementation.
//!
//! | PAL trait | Win32 API |
//! |---|---|
//! | PalIo | ReadFile / WriteFile / FlushFileBuffers |
//! | PalFs | CreateFileA / GetFileAttributesExA / CreateDirectoryA |
//! | PalNet | Winsock2: socket / connect / bind / listen / accept |
//! | PalSync | CreateMutexA / WaitForSingleObject / CreateThread |
//! | PalTime | QueryPerformanceCounter / GetSystemTimeAsFileTime / Sleep |
//! | PalProcess | GetCurrentProcessId / GetEnvironmentVariableA / ExitProcess |

#![allow(missing_docs)]

mod fs;
mod io;
mod net;
mod process;
mod sync;
mod time;

pub struct WindowsPal;
