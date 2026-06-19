// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
//
// seL4 backend — stub-complete at P45. Full IPC wiring at P48.

use crate::error::{IoError, IoResult};

pub struct RawFd(pub u32);

impl Drop for RawFd {
    fn drop(&mut self) {
        // P48: seL4 IPC close on file server capability
    }
}

pub fn open_read(_path: &str)          -> IoResult<RawFd>   { Err(IoError::Sel4IpcFault(0)) }
pub fn open_write(_path: &str)         -> IoResult<RawFd>   { Err(IoError::Sel4IpcFault(0)) }
pub fn read_to_end(_fd: &RawFd)        -> IoResult<Vec<u8>> { Err(IoError::Sel4IpcFault(0)) }
pub fn write_all(_fd: &RawFd, _b: &[u8]) -> IoResult<()>   { Err(IoError::Sel4IpcFault(0)) }
pub fn flush(_fd: &RawFd)              -> IoResult<()>       { Ok(()) }
pub fn stdout_write(_buf: &[u8])       -> IoResult<()>       { Err(IoError::Sel4IpcFault(0)) }
pub fn stderr_write(_buf: &[u8])       -> IoResult<()>       { Err(IoError::Sel4IpcFault(0)) }
pub fn stdout_flush()                  -> IoResult<()>       { Ok(()) }
pub fn stdin_read_line()               -> IoResult<String>   { Err(IoError::Sel4IpcFault(0)) }
pub fn path_exists(_path: &str)        -> bool               { false }
pub fn path_is_file(_path: &str)       -> bool               { false }
pub fn path_is_dir(_path: &str)        -> bool               { false }
