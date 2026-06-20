// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
use std::net::SocketAddr;
use std::str::FromStr;
use crate::error::{NetError, NetResult};
#[derive(Debug, Clone, PartialEq)]
pub struct AxonAddr {
    pub socket: SocketAddr,
    pub fingerprint: Option<[u8; 32]>,
}
impl AxonAddr {
    pub fn from_str(s: &str) -> NetResult<Self> {
        SocketAddr::from_str(s)
            .map(|socket| AxonAddr { socket, fingerprint: None })
            .map_err(|e| NetError::InvalidAddr(e.to_string()))
    }
    pub fn from_socket(socket: SocketAddr) -> Self {
        AxonAddr { socket, fingerprint: None }
    }
    pub fn with_fingerprint(mut self, fp: [u8; 32]) -> Self {
        self.fingerprint = Some(fp); self
    }
    pub fn is_sovereign(&self) -> bool { self.fingerprint.is_some() }
    pub fn port(&self) -> u16 { self.socket.port() }
    pub fn ip(&self) -> std::net::IpAddr { self.socket.ip() }
}
impl std::fmt::Display for AxonAddr {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.socket)?;
        if self.fingerprint.is_some() { write!(f, "[sovereign]")?; }
        Ok(())
    }
}
