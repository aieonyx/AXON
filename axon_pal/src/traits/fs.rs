use axon_core::prelude::*;
use crate::types::{AxonPath, FileStat, OpenFlags, RawFd};
pub trait PalFs {
    fn open(path: &AxonPath, flags: OpenFlags) -> AxonResult<RawFd>;
    fn close(fd: RawFd) -> AxonResult<()>;
    fn stat(path: &AxonPath) -> AxonResult<FileStat>;
    fn mkdir(path: &AxonPath, mode: u32) -> AxonResult<()>;
    fn remove(path: &AxonPath) -> AxonResult<()>;
    fn rename(from: &AxonPath, to: &AxonPath) -> AxonResult<()>;
    fn exists(path: &AxonPath) -> bool { Self::stat(path).is_ok() }
}
