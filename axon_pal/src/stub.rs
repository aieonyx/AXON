use axon_core::prelude::*;
use axon_core::types::U32;
use crate::traits::{PalFs, PalIo, PalNet, PalProcess, PalSync, PalTime};
use crate::types::*;

pub struct StubPal;

impl PalIo for StubPal {
    fn read(_fd: RawFd, _buf: &mut [u8]) -> AxonResult<usize> { AxonResult::Err(AxonError::not_implemented("StubPal::read")) }
    fn write(_fd: RawFd, _buf: &[u8]) -> AxonResult<usize> { AxonResult::Err(AxonError::not_implemented("StubPal::write")) }
    fn flush(_fd: RawFd) -> AxonResult<()> { AxonResult::Err(AxonError::not_implemented("StubPal::flush")) }
}
impl PalFs for StubPal {
    fn open(_p: &AxonPath, _f: OpenFlags) -> AxonResult<RawFd> { AxonResult::Err(AxonError::not_implemented("StubPal::open")) }
    fn close(_fd: RawFd) -> AxonResult<()> { AxonResult::Err(AxonError::not_implemented("StubPal::close")) }
    fn stat(_p: &AxonPath) -> AxonResult<FileStat> { AxonResult::Err(AxonError::not_implemented("StubPal::stat")) }
    fn mkdir(_p: &AxonPath, _m: u32) -> AxonResult<()> { AxonResult::Err(AxonError::not_implemented("StubPal::mkdir")) }
    fn remove(_p: &AxonPath) -> AxonResult<()> { AxonResult::Err(AxonError::not_implemented("StubPal::remove")) }
    fn rename(_f: &AxonPath, _t: &AxonPath) -> AxonResult<()> { AxonResult::Err(AxonError::not_implemented("StubPal::rename")) }
}
impl PalNet for StubPal {
    fn tcp_connect(_a: SocketAddr) -> AxonResult<RawFd> { AxonResult::Err(AxonError::not_implemented("StubPal::tcp_connect")) }
    fn tcp_listen(_a: SocketAddr, _b: u32) -> AxonResult<RawFd> { AxonResult::Err(AxonError::not_implemented("StubPal::tcp_listen")) }
    fn tcp_accept(_fd: RawFd) -> AxonResult<(RawFd, SocketAddr)> { AxonResult::Err(AxonError::not_implemented("StubPal::tcp_accept")) }
    fn udp_bind(_a: SocketAddr) -> AxonResult<RawFd> { AxonResult::Err(AxonError::not_implemented("StubPal::udp_bind")) }
    fn udp_send_to(_fd: RawFd, _b: &[u8], _a: SocketAddr) -> AxonResult<usize> { AxonResult::Err(AxonError::not_implemented("StubPal::udp_send_to")) }
    fn udp_recv_from(_fd: RawFd, _b: &mut [u8]) -> AxonResult<(usize, SocketAddr)> { AxonResult::Err(AxonError::not_implemented("StubPal::udp_recv_from")) }
    fn close(_fd: RawFd) -> AxonResult<()> { AxonResult::Err(AxonError::not_implemented("StubPal::net::close")) }
}
impl PalSync for StubPal {
    fn mutex_new() -> AxonResult<RawHandle> { AxonResult::Err(AxonError::not_implemented("StubPal::mutex_new")) }
    fn mutex_lock(_h: RawHandle) -> AxonResult<()> { AxonResult::Err(AxonError::not_implemented("StubPal::mutex_lock")) }
    fn mutex_unlock(_h: RawHandle) -> AxonResult<()> { AxonResult::Err(AxonError::not_implemented("StubPal::mutex_unlock")) }
    fn mutex_destroy(_h: RawHandle) -> AxonResult<()> { AxonResult::Err(AxonError::not_implemented("StubPal::mutex_destroy")) }
    fn thread_spawn(_f: fn()) -> AxonResult<RawHandle> { AxonResult::Err(AxonError::not_implemented("StubPal::thread_spawn")) }
    fn thread_join(_h: RawHandle) -> AxonResult<()> { AxonResult::Err(AxonError::not_implemented("StubPal::thread_join")) }
    fn thread_yield() {}
}
impl PalTime for StubPal {
    fn now_monotonic() -> AxonResult<Duration> { AxonResult::Err(AxonError::not_implemented("StubPal::now_monotonic")) }
    fn now_system() -> AxonResult<SystemTime> { AxonResult::Err(AxonError::not_implemented("StubPal::now_system")) }
    fn sleep(_d: Duration) -> AxonResult<()> { AxonResult::Err(AxonError::not_implemented("StubPal::sleep")) }
    fn process_start_time() -> AxonResult<SystemTime> { AxonResult::Err(AxonError::not_implemented("StubPal::process_start_time")) }
}
impl PalProcess for StubPal {
    fn args() -> AxonResult<&'static [&'static str]> { AxonResult::Ok(&[]) }
    fn env_var(_k: &str) -> AxonResult<&'static str> { AxonResult::Err(AxonError::not_found("StubPal: env vars not available")) }
    fn exit(_code: U32) -> ! { loop {} }
    fn pid() -> AxonResult<U32> { AxonResult::Err(AxonError::not_implemented("StubPal::pid")) }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::{PalFs, PalIo, PalNet, PalSync, PalTime};
    use axon_core::error::ErrorKind;

    #[test] fn stub_io_read()  { assert!(StubPal::read(RawFd::STDIN, &mut [0u8;8]).is_err()); }
    #[test] fn stub_io_write() { assert!(StubPal::write(RawFd::STDOUT, b"hi").is_err()); }
    #[test] fn stub_io_flush() { assert!(StubPal::flush(RawFd::STDOUT).is_err()); }
    #[test] fn stub_fs_open()   { assert!(StubPal::open(&AxonPath::new("/x"), OpenFlags::READ).is_err()); }
    #[test] fn stub_fs_stat()   { assert!(StubPal::stat(&AxonPath::new("/x")).is_err()); }
    #[test] fn stub_fs_exists() { assert!(!StubPal::exists(&AxonPath::new("/x"))); }
    #[test] fn stub_net_tcp_connect() { assert!(StubPal::tcp_connect(SocketAddr::loopback(8080)).is_err()); }
    #[test] fn stub_net_udp_bind() { assert!(StubPal::udp_bind(SocketAddr::loopback(9090)).is_err()); }
    #[test] fn stub_sync_mutex_new()  { assert!(StubPal::mutex_new().is_err()); }
    #[test] fn stub_sync_thread_yield_noop() { StubPal::thread_yield(); }
    #[test] fn stub_time_monotonic() { assert!(StubPal::now_monotonic().is_err()); }
    #[test] fn stub_process_args_empty() { assert_eq!(StubPal::args().unwrap().len(), 0); }
    #[test] fn stub_process_env_var_notfound() { assert!(StubPal::env_var("HOME").is_err()); }
    #[test] fn stub_error_kinds_are_not_implemented() {
        let e = StubPal::read(RawFd::STDIN, &mut [0u8]).err().unwrap();
        assert_eq!(e.kind, ErrorKind::NotImplemented);
        let e = StubPal::mutex_new().err().unwrap();
        assert_eq!(e.kind, ErrorKind::NotImplemented);
    }
}
