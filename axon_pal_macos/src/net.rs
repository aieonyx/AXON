#![allow(clippy::all, dead_code, unused_imports, unused_variables)]
use axon_core::prelude::*;
use axon_pal::{traits::PalNet, types::{RawFd, SocketAddr}};
use crate::{MacOsPal, error::last_os_axon_error};

#[cfg(target_os = "macos")]
fn to_sockaddr_in(addr: SocketAddr) -> AxonResult<(libc::sockaddr_in, libc::socklen_t)> {
    match addr {
        SocketAddr::V4 { ip, port } => {
            // macOS sockaddr_in has sin_len field unlike Linux
            let sa = libc::sockaddr_in {
                sin_len:    core::mem::size_of::<libc::sockaddr_in>() as u8,
                sin_family: libc::AF_INET as libc::sa_family_t,
                sin_port:   port.to_be(),
                sin_addr:   libc::in_addr { s_addr: u32::from_be_bytes(ip).to_be() },
                sin_zero:   [0; 8],
            };
            AxonResult::Ok((sa, core::mem::size_of::<libc::sockaddr_in>() as libc::socklen_t))
        }
        SocketAddr::V6 { .. } => AxonResult::Err(AxonError::not_implemented("IPv6 not yet implemented")),
    }
}

impl PalNet for MacOsPal {
    fn tcp_connect(addr: SocketAddr) -> AxonResult<RawFd> {
        #[cfg(target_os = "macos")]
        {
            let (sa, sa_len) = axon_try!(to_sockaddr_in(addr));
            let fd = unsafe { libc::socket(libc::AF_INET, libc::SOCK_STREAM, 0) };
            if fd < 0 { return AxonResult::Err(last_os_axon_error()); }
            let ret = unsafe { libc::connect(fd, &sa as *const libc::sockaddr_in as *const libc::sockaddr, sa_len) };
            if ret < 0 { unsafe { libc::close(fd); } AxonResult::Err(last_os_axon_error()) } else { AxonResult::Ok(RawFd(fd as u32)) }
        }
        #[cfg(not(target_os = "macos"))]
        AxonResult::Err(AxonError::not_implemented("axon_pal_macos::net: not on macOS"))
    }
    fn tcp_listen(addr: SocketAddr, backlog: u32) -> AxonResult<RawFd> {
        #[cfg(target_os = "macos")]
        {
            let (mut sa, sa_len) = axon_try!(to_sockaddr_in(addr));
            let fd = unsafe { libc::socket(libc::AF_INET, libc::SOCK_STREAM, 0) };
            if fd < 0 { return AxonResult::Err(last_os_axon_error()); }
            let opt: libc::c_int = 1;
            unsafe { libc::setsockopt(fd, libc::SOL_SOCKET, libc::SO_REUSEADDR, &opt as *const _ as *const libc::c_void, core::mem::size_of_val(&opt) as libc::socklen_t); }
            if unsafe { libc::bind(fd, &mut sa as *mut libc::sockaddr_in as *mut libc::sockaddr, sa_len) } < 0
                { unsafe { libc::close(fd); } return AxonResult::Err(last_os_axon_error()); }
            if unsafe { libc::listen(fd, backlog as libc::c_int) } < 0
                { unsafe { libc::close(fd); } return AxonResult::Err(last_os_axon_error()); }
            AxonResult::Ok(RawFd(fd as u32))
        }
        #[cfg(not(target_os = "macos"))]
        AxonResult::Err(AxonError::not_implemented("axon_pal_macos::net: not on macOS"))
    }
    fn tcp_accept(fd: RawFd) -> AxonResult<(RawFd, SocketAddr)> {
        #[cfg(target_os = "macos")]
        {
            let mut sa: libc::sockaddr_in = unsafe { core::mem::zeroed() };
            let mut sa_len = core::mem::size_of::<libc::sockaddr_in>() as libc::socklen_t;
            let conn = unsafe { libc::accept(fd.0 as libc::c_int, &mut sa as *mut libc::sockaddr_in as *mut libc::sockaddr, &mut sa_len) };
            if conn < 0 { AxonResult::Err(last_os_axon_error()) }
            else { AxonResult::Ok((RawFd(conn as u32), SocketAddr::V4 { ip: sa.sin_addr.s_addr.to_ne_bytes(), port: u16::from_be(sa.sin_port) })) }
        }
        #[cfg(not(target_os = "macos"))]
        AxonResult::Err(AxonError::not_implemented("axon_pal_macos::net: not on macOS"))
    }
    fn udp_bind(addr: SocketAddr) -> AxonResult<RawFd> {
        #[cfg(not(target_os = "macos"))]
        return AxonResult::Err(AxonError::not_implemented("axon_pal_macos::net: not on macOS"));
        #[cfg(target_os = "macos")]
        {
            let (mut sa, sa_len) = axon_try!(to_sockaddr_in(addr));
            let fd = unsafe { libc::socket(libc::AF_INET, libc::SOCK_DGRAM, 0) };
            if fd < 0 { return AxonResult::Err(last_os_axon_error()); }
            if unsafe { libc::bind(fd, &mut sa as *mut libc::sockaddr_in as *mut libc::sockaddr, sa_len) } < 0
                { unsafe { libc::close(fd); } AxonResult::Err(last_os_axon_error()) }
            else { AxonResult::Ok(RawFd(fd as u32)) }
        }
    }
    fn udp_send_to(_fd: RawFd, _buf: &[u8], _addr: SocketAddr) -> AxonResult<usize> {
        AxonResult::Err(AxonError::not_implemented("axon_pal_macos::udp_send_to"))
    }
    fn udp_recv_from(_fd: RawFd, _buf: &mut [u8]) -> AxonResult<(usize, SocketAddr)> {
        AxonResult::Err(AxonError::not_implemented("axon_pal_macos::udp_recv_from"))
    }
    fn close(fd: RawFd) -> AxonResult<()> {
        #[cfg(target_os = "macos")]
        {
            let ret = unsafe { libc::close(fd.0 as libc::c_int) };
            if ret < 0 { AxonResult::Err(last_os_axon_error()) } else { AxonResult::Ok(()) }
        }
        #[cfg(not(target_os = "macos"))]
        AxonResult::Err(AxonError::not_implemented("axon_pal_macos::close: not on macOS"))
    }
}
