use axon_core::prelude::*;
pub(crate) fn errno_to_axon_error() -> AxonError {
    let e = unsafe { *libc::__errno_location() };
    let (kind, msg) = match e {
        libc::ENOENT       => (ErrorKind::NotFound,         "no such file or directory"),
        libc::EACCES       => (ErrorKind::PermissionDenied, "permission denied"),
        libc::EPERM        => (ErrorKind::PermissionDenied, "operation not permitted"),
        libc::ETIMEDOUT    => (ErrorKind::TimedOut,         "connection timed out"),
        libc::EINVAL       => (ErrorKind::InvalidInput,     "invalid argument"),
        libc::EBADF        => (ErrorKind::InvalidInput,     "bad file descriptor"),
        libc::EEXIST       => (ErrorKind::InvalidState,     "file already exists"),
        libc::ENOTDIR      => (ErrorKind::InvalidInput,     "not a directory"),
        libc::EISDIR       => (ErrorKind::InvalidInput,     "is a directory"),
        libc::ENOTEMPTY    => (ErrorKind::InvalidState,     "directory not empty"),
        libc::ECONNREFUSED => (ErrorKind::Io,               "connection refused"),
        libc::ECONNRESET   => (ErrorKind::Io,               "connection reset by peer"),
        libc::EADDRINUSE   => (ErrorKind::InvalidState,     "address already in use"),
        libc::EAGAIN       => (ErrorKind::Io,               "resource temporarily unavailable"),
        libc::EINTR        => (ErrorKind::Io,               "interrupted by signal"),
        libc::ENOMEM       => (ErrorKind::Io,               "out of memory"),
        libc::ENOSPC       => (ErrorKind::Io,               "no space left on device"),
        libc::EPIPE        => (ErrorKind::Io,               "broken pipe"),
        _                  => (ErrorKind::Io,               "linux syscall error"),
    };
    AxonError::new(kind, msg).with_code(e as u32)
}
