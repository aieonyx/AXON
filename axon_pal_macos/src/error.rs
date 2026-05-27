#![allow(clippy::all, dead_code, unused_imports, unused_variables)]
use axon_core::prelude::*;

/// Convert the last OS error to an AxonError.
/// Uses std::io::Error on macOS — avoids __errno_location() (Linux-only).
pub(crate) fn last_os_axon_error() -> AxonError {
    let e = std::io::Error::last_os_error();
    let code = e.raw_os_error().unwrap_or(0) as u32;
    let (kind, msg) = match e.kind() {
        std::io::ErrorKind::NotFound         => (ErrorKind::NotFound,         "no such file or directory"),
        std::io::ErrorKind::PermissionDenied => (ErrorKind::PermissionDenied, "permission denied"),
        std::io::ErrorKind::TimedOut         => (ErrorKind::TimedOut,         "timed out"),
        std::io::ErrorKind::InvalidInput     => (ErrorKind::InvalidInput,     "invalid input"),
        std::io::ErrorKind::AddrInUse        => (ErrorKind::InvalidState,     "address already in use"),
        std::io::ErrorKind::ConnectionRefused => (ErrorKind::Io,              "connection refused"),
        std::io::ErrorKind::ConnectionReset  => (ErrorKind::Io,               "connection reset"),
        std::io::ErrorKind::OutOfMemory      => (ErrorKind::Io,               "out of memory"),
        _                                    => (ErrorKind::Io,               "darwin syscall error"),
    };
    AxonError::new(kind, msg).with_code(code)
}
