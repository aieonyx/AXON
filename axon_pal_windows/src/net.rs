use axon_core::prelude::*;
use axon_pal::{traits::PalNet, types::{RawFd, SocketAddr}};
use crate::WindowsPal;

#[cfg(windows)]
extern "system" {
    fn closesocket(s: usize) -> i32;
}

impl PalNet for WindowsPal {
    fn tcp_connect(_addr: SocketAddr) -> AxonResult<RawFd> {
        AxonResult::Err(AxonError::not_implemented("axon_pal_windows::tcp_connect — Winsock2 SL-10"))
    }
    fn tcp_listen(_addr: SocketAddr, _backlog: u32) -> AxonResult<RawFd> {
        AxonResult::Err(AxonError::not_implemented("axon_pal_windows::tcp_listen — Winsock2 SL-10"))
    }
    fn tcp_accept(_fd: RawFd) -> AxonResult<(RawFd, SocketAddr)> {
        AxonResult::Err(AxonError::not_implemented("axon_pal_windows::tcp_accept — Winsock2 SL-10"))
    }
    fn udp_bind(_addr: SocketAddr) -> AxonResult<RawFd> {
        AxonResult::Err(AxonError::not_implemented("axon_pal_windows::udp_bind — Winsock2 SL-10"))
    }
    fn udp_send_to(_fd: RawFd, _buf: &[u8], _addr: SocketAddr) -> AxonResult<usize> {
        AxonResult::Err(AxonError::not_implemented("axon_pal_windows::udp_send_to — Winsock2 SL-10"))
    }
    fn udp_recv_from(_fd: RawFd, _buf: &mut [u8]) -> AxonResult<(usize, SocketAddr)> {
        AxonResult::Err(AxonError::not_implemented("axon_pal_windows::udp_recv_from — Winsock2 SL-10"))
    }
    fn close(fd: RawFd) -> AxonResult<()> {
        #[cfg(windows)]
        { unsafe { closesocket(fd.0 as usize); } AxonResult::Ok(()) }
        #[cfg(not(windows))]
        AxonResult::Err(AxonError::not_implemented("not on Windows"))
    }
}
