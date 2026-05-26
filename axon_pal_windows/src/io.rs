use axon_core::prelude::*;
use axon_pal::{traits::PalIo, types::RawFd};
use crate::WindowsPal;

#[cfg(windows)]
extern "system" {
    fn ReadFile(hFile: isize, buf: *mut u8, nToRead: u32, nRead: *mut u32, ov: *mut ()) -> i32;
    fn WriteFile(hFile: isize, buf: *const u8, nToWrite: u32, nWritten: *mut u32, ov: *mut ()) -> i32;
    fn FlushFileBuffers(hFile: isize) -> i32;
}

impl PalIo for WindowsPal {
    fn read(fd: RawFd, buf: &mut [u8]) -> AxonResult<usize> {
        if buf.is_empty() { return AxonResult::Ok(0); }
        #[cfg(windows)]
        {
            let mut n: u32 = 0;
            let ok = unsafe { ReadFile(fd.0 as isize, buf.as_mut_ptr(), buf.len() as u32, &mut n, core::ptr::null_mut()) };
            if ok == 0 { AxonResult::Err(AxonError::io("ReadFile failed")) } else { AxonResult::Ok(n as usize) }
        }
        #[cfg(not(windows))]
        AxonResult::Err(AxonError::not_implemented("axon_pal_windows: not on Windows"))
    }
    fn write(fd: RawFd, buf: &[u8]) -> AxonResult<usize> {
        if buf.is_empty() { return AxonResult::Ok(0); }
        #[cfg(windows)]
        {
            let mut n: u32 = 0;
            let ok = unsafe { WriteFile(fd.0 as isize, buf.as_ptr(), buf.len() as u32, &mut n, core::ptr::null_mut()) };
            if ok == 0 { AxonResult::Err(AxonError::io("WriteFile failed")) } else { AxonResult::Ok(n as usize) }
        }
        #[cfg(not(windows))]
        AxonResult::Err(AxonError::not_implemented("axon_pal_windows: not on Windows"))
    }
    fn flush(fd: RawFd) -> AxonResult<()> {
        #[cfg(windows)]
        {
            let ok = unsafe { FlushFileBuffers(fd.0 as isize) };
            if ok == 0 { AxonResult::Err(AxonError::io("FlushFileBuffers failed")) } else { AxonResult::Ok(()) }
        }
        #[cfg(not(windows))]
        AxonResult::Err(AxonError::not_implemented("axon_pal_windows: not on Windows"))
    }
}
