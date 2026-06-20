// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
use std::net::TcpListener;
use crate::addr::AxonAddr;
use crate::conn::AxonConn;
use crate::error::{NetError, NetResult};
pub struct AxonListener {
    listener: TcpListener,
    pub local: AxonAddr,
}
impl AxonListener {
    pub fn bind(addr: &AxonAddr) -> NetResult<Self> {
        let listener = TcpListener::bind(addr.socket)
            .map_err(|e| NetError::BindFailed(e.to_string()))?;
        let local_addr = listener.local_addr()
            .map_err(|e| NetError::BindFailed(e.to_string()))?;
        Ok(AxonListener { listener, local: AxonAddr::from_socket(local_addr) })
    }
    pub fn accept(&self) -> NetResult<AxonConn> {
        let (stream, remote_addr) = self.listener.accept()
            .map_err(|e| NetError::ConnectFailed(e.to_string()))?;
        let local_addr = stream.local_addr()
            .map_err(|e| NetError::ConnectFailed(e.to_string()))?;
        Ok(AxonConn::from_parts(
            stream,
            AxonAddr::from_socket(local_addr),
            AxonAddr::from_socket(remote_addr),
        ))
    }
    pub fn local_addr(&self) -> &AxonAddr { &self.local }
}
impl std::fmt::Debug for AxonListener {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "AxonListener {{ local: {} }}", self.local)
    }
}
