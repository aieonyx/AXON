use axon_core::prelude::*;
use crate::types::{RawFd, SocketAddr};
pub trait PalNet {
    fn tcp_connect(addr: SocketAddr) -> AxonResult<RawFd>;
    fn tcp_listen(addr: SocketAddr, backlog: u32) -> AxonResult<RawFd>;
    fn tcp_accept(fd: RawFd) -> AxonResult<(RawFd, SocketAddr)>;
    fn udp_bind(addr: SocketAddr) -> AxonResult<RawFd>;
    fn udp_send_to(fd: RawFd, buf: &[u8], addr: SocketAddr) -> AxonResult<usize>;
    fn udp_recv_from(fd: RawFd, buf: &mut [u8]) -> AxonResult<(usize, SocketAddr)>;
    fn close(fd: RawFd) -> AxonResult<()>;
}
