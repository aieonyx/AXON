// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::Duration;
use crate::addr::AxonAddr;
use crate::error::{NetError, NetResult};
pub struct AxonConn {
    stream: TcpStream,
    pub local: AxonAddr,
    pub remote: AxonAddr,
}
impl AxonConn {
    pub fn connect(remote: &AxonAddr) -> NetResult<Self> {
        let stream = TcpStream::connect(remote.socket)
            .map_err(|e| NetError::ConnectFailed(e.to_string()))?;
        let local_addr = stream.local_addr()
            .map_err(|e| NetError::ConnectFailed(e.to_string()))?;
        Ok(AxonConn { stream, local: AxonAddr::from_socket(local_addr), remote: remote.clone() })
    }
    pub(crate) fn from_parts(stream: TcpStream, local: AxonAddr, remote: AxonAddr) -> Self {
        AxonConn { stream, local, remote }
    }
    pub fn send(&mut self, data: &[u8]) -> NetResult<usize> {
        self.stream.write(data).map_err(|e| NetError::SendFailed(e.to_string()))
    }
    pub fn recv(&mut self, buf: &mut [u8]) -> NetResult<usize> {
        self.stream.read(buf).map_err(|e| NetError::RecvFailed(e.to_string()))
    }
    pub fn send_all(&mut self, data: &[u8]) -> NetResult<()> {
        self.stream.write_all(data).map_err(|e| NetError::SendFailed(e.to_string()))
    }
    pub fn set_read_timeout(&mut self, ms: u64) -> NetResult<()> {
        self.stream.set_read_timeout(Some(Duration::from_millis(ms)))
            .map_err(|e| NetError::ConnectFailed(e.to_string()))
    }
    pub fn close(self) -> NetResult<()> {
        self.stream.shutdown(std::net::Shutdown::Both).map_err(|_| NetError::Closed)
    }
}
impl std::fmt::Debug for AxonConn {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "AxonConn {{ local: {}, remote: {} }}", self.local, self.remote)
    }
}
