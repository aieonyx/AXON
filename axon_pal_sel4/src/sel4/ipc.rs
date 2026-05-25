//! seL4 IPC helpers — read/write message registers via IPC buffer.

use super::{ipc_buffer, MessageInfo};

/// Write `data` into message registers 0..n and return a MessageInfo.
///
/// # Safety
///
/// The IPC buffer must be mapped and accessible.
pub unsafe fn write_mrs(data: &[usize]) -> MessageInfo {
    let buf = ipc_buffer();
    let n = data.len().min(super::SEL4_MSG_MAX_LENGTH);
    buf.msg[..n].copy_from_slice(&data[..n]);
    MessageInfo::new(n)
}

/// Read `n` message registers from the IPC buffer.
///
/// # Safety
///
/// The IPC buffer must be mapped and accessible.
pub unsafe fn read_mrs(n: usize) -> &'static [usize] {
    let buf = ipc_buffer();
    &buf.msg[..n.min(super::SEL4_MSG_MAX_LENGTH)]
}
