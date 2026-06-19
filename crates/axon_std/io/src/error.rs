// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0

#[derive(Debug, PartialEq)]
pub enum IoError {
    NotFound,
    PermissionDenied,
    UnexpectedEof,
    WriteZero,
    BrokenPipe,
    InvalidPath,
    Sel4IpcFault(u32),
    Unknown(i32),
}

impl core::fmt::Display for IoError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            IoError::NotFound         => write!(f, "io: path not found"),
            IoError::PermissionDenied => write!(f, "io: permission denied"),
            IoError::UnexpectedEof    => write!(f, "io: unexpected end of file"),
            IoError::WriteZero        => write!(f, "io: write returned zero bytes"),
            IoError::BrokenPipe       => write!(f, "io: broken pipe"),
            IoError::InvalidPath      => write!(f, "io: invalid path"),
            IoError::Sel4IpcFault(c)  => write!(f, "io: seL4 IPC fault ({})", c),
            IoError::Unknown(e)       => write!(f, "io: unknown error ({})", e),
        }
    }
}

pub type IoResult<T> = Result<T, IoError>;
