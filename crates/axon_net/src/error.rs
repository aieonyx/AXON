// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
use std::fmt;
#[derive(Debug)]
pub enum NetError {
    BindFailed(String), ConnectFailed(String), SendFailed(String),
    RecvFailed(String), InvalidAddr(String), Timeout, Closed,
}
impl fmt::Display for NetError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            NetError::BindFailed(s)    => write!(f, "bind failed: {}", s),
            NetError::ConnectFailed(s) => write!(f, "connect failed: {}", s),
            NetError::SendFailed(s)    => write!(f, "send failed: {}", s),
            NetError::RecvFailed(s)    => write!(f, "recv failed: {}", s),
            NetError::InvalidAddr(s)   => write!(f, "invalid addr: {}", s),
            NetError::Timeout          => write!(f, "timeout"),
            NetError::Closed           => write!(f, "connection closed"),
        }
    }
}
pub type NetResult<T> = Result<T, NetError>;
