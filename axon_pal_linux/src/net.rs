use axon_core::prelude::*;
use axon_pal::{traits::PalNet, types::{RawFd, SocketAddr}};
use crate::{LinuxPal, error::errno_to_axon_error};

fn to_sockaddr_in(addr: SocketAddr) -> AxonResult<(libc::sockaddr_in, libc::socklen_t)> {
    match addr {
        SocketAddr::V4 { ip, port } => AxonResult::Ok((
            libc::sockaddr_in {
                sin_family: libc::AF_INET as libc::sa_family_t,
                sin_port: port.to_be(),
                sin_addr: libc::in_addr { s_addr: u32::from_be_bytes(ip).to_be() },
                sin_zero: [0; 8],
            },
            core::mem::size_of::<libc::sockaddr_in>() as libc::socklen_t,
        )),
        SocketAddr::V6 { .. } => AxonResult::Err(AxonError::not_implemented("IPv6 not yet implemented")),
    }
}
fn from_sockaddr_in(sa: &libc::sockaddr_in) -> SocketAddr {
    SocketAddr::V4 { ip: sa.sin_addr.s_addr.to_ne_bytes(), port: u16::from_be(sa.sin_port) }
}

impl PalNet for LinuxPal {
    fn tcp_connect(addr: SocketAddr) -> AxonResult<RawFd> {
        let (sa, sa_len) = axon_try!(to_sockaddr_in(addr));
        let fd = unsafe { libc::socket(libc::AF_INET, libc::SOCK_STREAM, 0) };
        if fd < 0 { return AxonResult::Err(errno_to_axon_error()); }
        let ret = unsafe { libc::connect(fd, &sa as *const libc::sockaddr_in as *const libc::sockaddr, sa_len) };
        if ret < 0 { unsafe { libc::close(fd); } AxonResult::Err(errno_to_axon_error()) }
        else        { AxonResult::Ok(RawFd(fd as u32)) }
    }
    fn tcp_listen(addr: SocketAddr, backlog: u32) -> AxonResult<RawFd> {
        let (mut sa, sa_len) = axon_try!(to_sockaddr_in(addr));
        let fd = unsafe { libc::socket(libc::AF_INET, libc::SOCK_STREAM, 0) };
        if fd < 0 { return AxonResult::Err(errno_to_axon_error()); }
        let opt: libc::c_int = 1;
        unsafe { libc::setsockopt(fd, libc::SOL_SOCKET, libc::SO_REUSEADDR, &opt as *const _ as *const libc::c_void, core::mem::size_of_val(&opt) as libc::socklen_t); }
        if unsafe { libc::bind(fd, &mut sa as *mut libc::sockaddr_in as *mut libc::sockaddr, sa_len) } < 0
            { unsafe { libc::close(fd); } return AxonResult::Err(errno_to_axon_error()); }
        if unsafe { libc::listen(fd, backlog as libc::c_int) } < 0
            { unsafe { libc::close(fd); } return AxonResult::Err(errno_to_axon_error()); }
        AxonResult::Ok(RawFd(fd as u32))
    }
    fn tcp_accept(fd: RawFd) -> AxonResult<(RawFd, SocketAddr)> {
        let mut sa: libc::sockaddr_in = unsafe { core::mem::zeroed() };
        let mut sa_len = core::mem::size_of::<libc::sockaddr_in>() as libc::socklen_t;
        let conn = unsafe { libc::accept(fd.0 as libc::c_int, &mut sa as *mut libc::sockaddr_in as *mut libc::sockaddr, &mut sa_len) };
        if conn < 0 { AxonResult::Err(errno_to_axon_error()) } else { AxonResult::Ok((RawFd(conn as u32), from_sockaddr_in(&sa))) }
    }
    fn udp_bind(addr: SocketAddr) -> AxonResult<RawFd> {
        let (mut sa, sa_len) = axon_try!(to_sockaddr_in(addr));
        let fd = unsafe { libc::socket(libc::AF_INET, libc::SOCK_DGRAM, 0) };
        if fd < 0 { return AxonResult::Err(errno_to_axon_error()); }
        if unsafe { libc::bind(fd, &mut sa as *mut libc::sockaddr_in as *mut libc::sockaddr, sa_len) } < 0
            { unsafe { libc::close(fd); } AxonResult::Err(errno_to_axon_error()) }
        else { AxonResult::Ok(RawFd(fd as u32)) }
    }
    fn udp_send_to(fd: RawFd, buf: &[u8], addr: SocketAddr) -> AxonResult<usize> {
        let (sa, sa_len) = axon_try!(to_sockaddr_in(addr));
        let ret = unsafe { libc::sendto(fd.0 as libc::c_int, buf.as_ptr() as *const libc::c_void, buf.len(), 0, &sa as *const libc::sockaddr_in as *const libc::sockaddr, sa_len) };
        if ret < 0 { AxonResult::Err(errno_to_axon_error()) } else { AxonResult::Ok(ret as usize) }
    }
    fn udp_recv_from(fd: RawFd, buf: &mut [u8]) -> AxonResult<(usize, SocketAddr)> {
        let mut sa: libc::sockaddr_in = unsafe { core::mem::zeroed() };
        let mut sa_len = core::mem::size_of::<libc::sockaddr_in>() as libc::socklen_t;
        let ret = unsafe { libc::recvfrom(fd.0 as libc::c_int, buf.as_mut_ptr() as *mut libc::c_void, buf.len(), 0, &mut sa as *mut libc::sockaddr_in as *mut libc::sockaddr, &mut sa_len) };
        if ret < 0 { AxonResult::Err(errno_to_axon_error()) } else { AxonResult::Ok((ret as usize, from_sockaddr_in(&sa))) }
    }
    fn close(fd: RawFd) -> AxonResult<()> {
        let ret = unsafe { libc::close(fd.0 as libc::c_int) };
        if ret < 0 { AxonResult::Err(errno_to_axon_error()) } else { AxonResult::Ok(()) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axon_pal::traits::{PalNet, PalSync};
    use std::sync::atomic::{AtomicU16, AtomicU32, Ordering};
    static PORT_COUNTER: AtomicU16 = AtomicU16::new(19500);
    fn next_port() -> u16 { PORT_COUNTER.fetch_add(1, Ordering::SeqCst) }
    #[test] fn net_tcp_listen_and_close() {
        let fd = LinuxPal::tcp_listen(SocketAddr::loopback(next_port()), 4).unwrap();
        LinuxPal::close(fd).unwrap();
    }
    #[test] fn net_tcp_connect_and_accept() {
        let port = next_port();
        let listen_addr = SocketAddr::loopback(port);
        let server_fd = LinuxPal::tcp_listen(listen_addr, 4).unwrap();
        static SFD: AtomicU32 = AtomicU32::new(0);
        SFD.store(server_fd.0, Ordering::SeqCst);
        fn accept_one() {
            let (conn,_) = LinuxPal::tcp_accept(RawFd(SFD.load(Ordering::SeqCst))).unwrap();
            LinuxPal::close(conn).unwrap();
        }
        let h = LinuxPal::thread_spawn(accept_one).unwrap();
        let client = LinuxPal::tcp_connect(listen_addr).unwrap();
        LinuxPal::thread_join(h).unwrap();
        LinuxPal::close(client).unwrap(); LinuxPal::close(server_fd).unwrap();
    }
    #[test] fn net_udp_send_recv() {
        let recv_addr = SocketAddr::loopback(next_port());
        let recv_fd = LinuxPal::udp_bind(recv_addr).unwrap();
        let send_fd = LinuxPal::udp_bind(SocketAddr::loopback(next_port())).unwrap();
        let msg = b"axon udp";
        LinuxPal::udp_send_to(send_fd, msg, recv_addr).unwrap();
        let mut buf = [0u8; 64];
        let (n, _) = LinuxPal::udp_recv_from(recv_fd, &mut buf).unwrap();
        assert_eq!(&buf[..n], msg);
        LinuxPal::close(send_fd).unwrap(); LinuxPal::close(recv_fd).unwrap();
    }
    #[test] fn net_connect_refused_returns_err() {
        assert!(LinuxPal::tcp_connect(SocketAddr::loopback(1)).is_err());
    }
}
