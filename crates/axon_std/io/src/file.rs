// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0

use crate::backend::active::RawFd;
use crate::error::{IoError, IoResult};

pub struct File {
    pub(crate) fd: RawFd,
}

impl File {
    pub(crate) fn from_fd(fd: RawFd) -> Self { Self { fd } }
}

pub fn open(path: &str) -> IoResult<File> {
    crate::backend::active::open_read(path).map(File::from_fd)
}

pub fn create(path: &str) -> IoResult<File> {
    crate::backend::active::open_write(path).map(File::from_fd)
}

pub fn read_to_end(file: &File) -> IoResult<Vec<u8>> {
    crate::backend::active::read_to_end(&file.fd)
}

pub fn read_to_string(path: &str) -> IoResult<String> {
    let file = open(path)?;
    let bytes = read_to_end(&file)?;
    String::from_utf8(bytes).map_err(|_| IoError::Unknown(-1))
}

pub fn write_all(file: &File, buf: &[u8]) -> IoResult<()> {
    crate::backend::active::write_all(&file.fd, buf)
}

pub fn flush(file: &File) -> IoResult<()> {
    crate::backend::active::flush(&file.fd)
}

pub fn close(file: File) -> IoResult<()> {
    drop(file);
    Ok(())
}
