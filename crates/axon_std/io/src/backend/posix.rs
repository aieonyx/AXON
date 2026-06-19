// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0

use crate::error::{IoError, IoResult};
use std::ffi::CString;
use std::os::raw::c_int;

pub struct RawFd(pub c_int);

impl Drop for RawFd {
    fn drop(&mut self) {
        if self.0 >= 0 {
            unsafe { libc::close(self.0) };
        }
    }
}

pub fn open_read(path: &str) -> IoResult<RawFd> {
    let cpath = CString::new(path).map_err(|_| IoError::InvalidPath)?;
    let fd = unsafe { libc::open(cpath.as_ptr(), libc::O_RDONLY) };
    if fd < 0 { Err(map_errno()) } else { Ok(RawFd(fd)) }
}

pub fn open_write(path: &str) -> IoResult<RawFd> {
    let cpath = CString::new(path).map_err(|_| IoError::InvalidPath)?;
    let fd = unsafe {
        libc::open(cpath.as_ptr(), libc::O_WRONLY | libc::O_CREAT | libc::O_TRUNC, 0o644)
    };
    if fd < 0 { Err(map_errno()) } else { Ok(RawFd(fd)) }
}

pub fn read_to_end(fd: &RawFd) -> IoResult<Vec<u8>> {
    let mut buf = Vec::new();
    let mut chunk = [0u8; 4096];
    loop {
        let n = unsafe {
            libc::read(fd.0, chunk.as_mut_ptr() as *mut libc::c_void, chunk.len())
        };
        match n {
            0 => break,
            n if n < 0 => return Err(map_errno()),
            n => buf.extend_from_slice(&chunk[..n as usize]),
        }
    }
    Ok(buf)
}

pub fn write_all(fd: &RawFd, buf: &[u8]) -> IoResult<()> {
    let mut written = 0;
    while written < buf.len() {
        let n = unsafe {
            libc::write(fd.0, buf[written..].as_ptr() as *const libc::c_void, buf.len() - written)
        };
        match n {
            0 => return Err(IoError::WriteZero),
            n if n < 0 => return Err(map_errno()),
            n => written += n as usize,
        }
    }
    Ok(())
}

pub fn flush(_fd: &RawFd) -> IoResult<()> { Ok(()) }

pub fn stdout_write(buf: &[u8]) -> IoResult<()> {
    write_all(&RawFd(libc::STDOUT_FILENO), buf)
}

pub fn stderr_write(buf: &[u8]) -> IoResult<()> {
    write_all(&RawFd(libc::STDERR_FILENO), buf)
}

pub fn stdout_flush() -> IoResult<()> { Ok(()) }

pub fn stdin_read_line() -> IoResult<String> {
    let mut result = Vec::new();
    let mut byte = [0u8; 1];
    loop {
        let n = unsafe {
            libc::read(libc::STDIN_FILENO, byte.as_mut_ptr() as *mut libc::c_void, 1)
        };
        match n {
            0 => break,
            n if n < 0 => return Err(map_errno()),
            _ => {
                if byte[0] == b'\n' { break; }
                result.push(byte[0]);
            }
        }
    }
    String::from_utf8(result).map_err(|_| IoError::Unknown(-1))
}

pub fn path_exists(path: &str) -> bool {
    let Ok(cpath) = CString::new(path) else { return false };
    let mut stat: libc::stat = unsafe { std::mem::zeroed() };
    unsafe { libc::stat(cpath.as_ptr(), &mut stat) == 0 }
}

pub fn path_is_file(path: &str) -> bool {
    let Ok(cpath) = CString::new(path) else { return false };
    let mut stat: libc::stat = unsafe { std::mem::zeroed() };
    if unsafe { libc::stat(cpath.as_ptr(), &mut stat) } != 0 { return false; }
    (stat.st_mode & libc::S_IFMT) == libc::S_IFREG
}

pub fn path_is_dir(path: &str) -> bool {
    let Ok(cpath) = CString::new(path) else { return false };
    let mut stat: libc::stat = unsafe { std::mem::zeroed() };
    if unsafe { libc::stat(cpath.as_ptr(), &mut stat) } != 0 { return false; }
    (stat.st_mode & libc::S_IFMT) == libc::S_IFDIR
}

fn map_errno() -> IoError {
    let e = unsafe { *libc::__errno_location() };
    match e {
        libc::ENOENT            => IoError::NotFound,
        libc::EACCES
        | libc::EPERM           => IoError::PermissionDenied,
        libc::EPIPE             => IoError::BrokenPipe,
        other                   => IoError::Unknown(other),
    }
}
