//! PalNet for seL4 — IPC to a network server protection domain.
use axon_core::prelude::*;
use axon_pal::{traits::PalNet, types::{RawFd, SocketAddr}};
use crate::Sel4Pal;

impl PalNet for Sel4Pal {
    fn tcp_connect(_addr: SocketAddr) -> AxonResult<RawFd> {
        AxonResult::Err(AxonError::not_implemented("seL4 Net: requires network server IPC"))
    }
    fn tcp_listen(_addr: SocketAddr, _backlog: u32) -> AxonResult<RawFd> {
        AxonResult::Err(AxonError::not_implemented("seL4 Net: requires network server IPC"))
    }
    fn tcp_accept(_fd: RawFd) -> AxonResult<(RawFd, SocketAddr)> {
        AxonResult::Err(AxonError::not_implemented("seL4 Net: requires network server IPC"))
    }
    fn udp_bind(_addr: SocketAddr) -> AxonResult<RawFd> {
        AxonResult::Err(AxonError::not_implemented("seL4 Net: requires network server IPC"))
    }
    fn udp_send_to(_fd: RawFd, _buf: &[u8], _addr: SocketAddr) -> AxonResult<usize> {
        AxonResult::Err(AxonError::not_implemented("seL4 Net: requires network server IPC"))
    }
    fn udp_recv_from(_fd: RawFd, _buf: &mut [u8]) -> AxonResult<(usize, SocketAddr)> {
        AxonResult::Err(AxonError::not_implemented("seL4 Net: requires network server IPC"))
    }
    fn close(_fd: RawFd) -> AxonResult<()> {
        AxonResult::Err(AxonError::not_implemented("seL4 Net: requires network server IPC"))
    }
}
