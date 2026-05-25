use axon_core::prelude::*;
use axon_pal::{traits::PalIo, types::RawFd};
use crate::{LinuxPal, error::errno_to_axon_error};

impl PalIo for LinuxPal {
    fn read(fd: RawFd, buf: &mut [u8]) -> AxonResult<usize> {
        if buf.is_empty() { return AxonResult::Ok(0); }
        let ret = unsafe { libc::read(fd.0 as libc::c_int, buf.as_mut_ptr() as *mut libc::c_void, buf.len()) };
        if ret < 0 { AxonResult::Err(errno_to_axon_error()) } else { AxonResult::Ok(ret as usize) }
    }
    fn write(fd: RawFd, buf: &[u8]) -> AxonResult<usize> {
        if buf.is_empty() { return AxonResult::Ok(0); }
        let ret = unsafe { libc::write(fd.0 as libc::c_int, buf.as_ptr() as *const libc::c_void, buf.len()) };
        if ret < 0 { AxonResult::Err(errno_to_axon_error()) } else { AxonResult::Ok(ret as usize) }
    }
    fn flush(fd: RawFd) -> AxonResult<()> {
        let ret = unsafe { libc::fdatasync(fd.0 as libc::c_int) };
        if ret < 0 {
            let e = unsafe { *libc::__errno_location() };
            if e == libc::EINVAL || e == libc::EROFS { AxonResult::Ok(()) }
            else { AxonResult::Err(errno_to_axon_error()) }
        } else { AxonResult::Ok(()) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axon_pal::traits::PalIo;
    #[test] fn io_write_and_read_roundtrip() {
        let path = std::format!("/tmp/axon_pal_io_{}", unsafe { libc::getpid() });
        let cpath = std::ffi::CString::new(path.as_str()).unwrap();
        let wfd = unsafe { libc::open(cpath.as_ptr(), libc::O_WRONLY|libc::O_CREAT|libc::O_TRUNC, 0o644) };
        assert!(wfd >= 0);
        let n = LinuxPal::write(RawFd(wfd as u32), b"AXON Linux PAL test").unwrap();
        assert_eq!(n, 19);
        LinuxPal::flush(RawFd(wfd as u32)).unwrap();
        unsafe { libc::close(wfd); }
        let rfd = unsafe { libc::open(cpath.as_ptr(), libc::O_RDONLY, 0) };
        assert!(rfd >= 0);
        let mut buf = [0u8; 64];
        let n = LinuxPal::read(RawFd(rfd as u32), &mut buf).unwrap();
        assert_eq!(&buf[..n], b"AXON Linux PAL test");
        unsafe { libc::close(rfd); libc::unlink(cpath.as_ptr()); }
    }
    #[test] fn io_write_all_roundtrip() {
        let path = std::format!("/tmp/axon_pal_io_all_{}", unsafe { libc::getpid() });
        let cpath = std::ffi::CString::new(path.as_str()).unwrap();
        let fd = unsafe { libc::open(cpath.as_ptr(), libc::O_WRONLY|libc::O_CREAT|libc::O_TRUNC, 0o644) };
        LinuxPal::write_all(RawFd(fd as u32), b"hello world").unwrap();
        unsafe { libc::close(fd); libc::unlink(cpath.as_ptr()); }
    }
    #[test] fn io_read_empty_buf_returns_zero() { assert_eq!(LinuxPal::read(RawFd::STDIN, &mut []).unwrap(), 0); }
    #[test] fn io_write_empty_buf_returns_zero() { assert_eq!(LinuxPal::write(RawFd::STDOUT, &[]).unwrap(), 0); }
    #[test] fn io_flush_stdout() { assert!(LinuxPal::flush(RawFd::STDOUT).is_ok()); }
    #[test] fn io_read_bad_fd_returns_err() {
        use axon_core::error::ErrorKind;
        let r = LinuxPal::read(RawFd(9999), &mut [0u8; 8]);
        assert!(r.is_err());
        assert_eq!(r.err().unwrap().kind, ErrorKind::InvalidInput);
    }
}
