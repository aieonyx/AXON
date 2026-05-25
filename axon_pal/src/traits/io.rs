use axon_core::prelude::*;
use crate::types::RawFd;
pub trait PalIo {
    fn read(fd: RawFd, buf: &mut [u8]) -> AxonResult<usize>;
    fn write(fd: RawFd, buf: &[u8]) -> AxonResult<usize>;
    fn flush(fd: RawFd) -> AxonResult<()>;
    fn write_all(fd: RawFd, mut buf: &[u8]) -> AxonResult<()> {
        while !buf.is_empty() {
            let n = axon_try!(Self::write(fd, buf));
            if n == 0 { return AxonResult::Err(AxonError::io("write_all: zero-length write")); }
            buf = &buf[n..];
        }
        AxonResult::Ok(())
    }
}
