use axon_core::prelude::*;
use axon_pal::{traits::PalIo, types::RawFd};
use crate::{MacOsPal, error::last_os_axon_error};

// F_FULLFSYNC = 51 on macOS — not in libc crate when compiling on Linux
#[cfg(target_os = "macos")]
const F_FULLFSYNC: libc::c_int = 51;

impl PalIo for MacOsPal {
    fn read(fd: RawFd, buf: &mut [u8]) -> AxonResult<usize> {
        if buf.is_empty() { return AxonResult::Ok(0); }
        #[cfg(target_os = "macos")]
        {
            let ret = unsafe { libc::read(fd.0 as libc::c_int, buf.as_mut_ptr() as *mut libc::c_void, buf.len()) };
            if ret < 0 { AxonResult::Err(last_os_axon_error()) } else { AxonResult::Ok(ret as usize) }
        }
        #[cfg(not(target_os = "macos"))]
        AxonResult::Err(AxonError::not_implemented("axon_pal_macos: not running on macOS"))
    }
    fn write(fd: RawFd, buf: &[u8]) -> AxonResult<usize> {
        if buf.is_empty() { return AxonResult::Ok(0); }
        #[cfg(target_os = "macos")]
        {
            let ret = unsafe { libc::write(fd.0 as libc::c_int, buf.as_ptr() as *const libc::c_void, buf.len()) };
            if ret < 0 { AxonResult::Err(last_os_axon_error()) } else { AxonResult::Ok(ret as usize) }
        }
        #[cfg(not(target_os = "macos"))]
        AxonResult::Err(AxonError::not_implemented("axon_pal_macos: not running on macOS"))
    }
    fn flush(fd: RawFd) -> AxonResult<()> {
        #[cfg(target_os = "macos")]
        {
            let ret = unsafe { libc::fcntl(fd.0 as libc::c_int, F_FULLFSYNC) };
            if ret < 0 {
                let e = std::io::Error::last_os_error();
                match e.raw_os_error() {
                    Some(libc::ENOTSUP) | Some(libc::EINVAL) => AxonResult::Ok(()),
                    _ => AxonResult::Err(last_os_axon_error()),
                }
            } else { AxonResult::Ok(()) }
        }
        #[cfg(not(target_os = "macos"))]
        AxonResult::Err(AxonError::not_implemented("axon_pal_macos: not running on macOS"))
    }
}
