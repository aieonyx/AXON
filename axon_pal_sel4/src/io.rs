//! PalIo for seL4 — IPC to a console/serial server.
//!
//! seL4 has no native I/O. The AXON root task communicates with a
//! serial driver protection domain via IPC endpoint calls.
//! The driver endpoint capability slot is conventionally CPtr(16).

use axon_core::prelude::*;
use axon_pal::{traits::PalIo, types::RawFd};
use crate::Sel4Pal;
use crate::sel4::{MessageInfo, syscall};

/// Conventional capability slot for the console server endpoint.
const CONSOLE_EP: usize = 16;

/// IPC message labels for the console server protocol.
mod label {
    pub const WRITE: usize = 1;
    pub const READ:  usize = 2;
    pub const FLUSH: usize = 3;
}

impl PalIo for Sel4Pal {
    fn read(_fd: RawFd, buf: &mut [u8]) -> AxonResult<usize> {
        if buf.is_empty() { return AxonResult::Ok(0); }
        // Safety: IPC buffer is mapped by seL4 kernel at boot.
        // We send a READ request to the console server and receive data.
        unsafe {
            let ipc = crate::sel4::ipc_buffer();
            ipc.msg[0] = label::READ;
            ipc.msg[1] = buf.len();
            let reply_info = syscall::sel4_call(CONSOLE_EP, MessageInfo::new(2).0);
            let reply = MessageInfo(reply_info);
            if reply.length() < 1 {
                return AxonResult::Err(AxonError::io("seL4 console read: empty reply"));
            }
            let n = ipc.msg[0].min(buf.len());
            // Data follows in MR[1..1+n] as packed bytes
            for i in 0..n {
                buf[i] = ((ipc.msg[1 + i/8] >> ((i % 8) * 8)) & 0xFF) as u8;
            }
            AxonResult::Ok(n)
        }
    }

    fn write(_fd: RawFd, buf: &[u8]) -> AxonResult<usize> {
        if buf.is_empty() { return AxonResult::Ok(0); }
        // Safety: IPC buffer mapped; we pack bytes into MRs and call console server.
        unsafe {
            let ipc = crate::sel4::ipc_buffer();
            ipc.msg[0] = label::WRITE;
            let n = buf.len().min(110 * 8); // max bytes fitting in MRs
            ipc.msg[1] = n;
            // Pack bytes into MRs (8 bytes per MR)
            for (i, &byte) in buf[..n].iter().enumerate() {
                ipc.msg[2 + i/8] |= (byte as usize) << ((i % 8) * 8);
            }
            let mr_count = 2 + (n + 7) / 8;
            syscall::sel4_call(CONSOLE_EP, MessageInfo::new(mr_count).0);
        }
        AxonResult::Ok(buf.len().min(110 * 8))
    }

    fn flush(_fd: RawFd) -> AxonResult<()> {
        // Safety: send FLUSH to console server.
        unsafe {
            let ipc = crate::sel4::ipc_buffer();
            ipc.msg[0] = label::FLUSH;
            syscall::sel4_call(CONSOLE_EP, MessageInfo::new(1).0);
        }
        AxonResult::Ok(())
    }
}
