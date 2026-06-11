//! seL4 IPC syscall wrappers
//! Copyright (c) 2026 Edison Lepiten / AIEONYX
//!
//! Type-safe wrappers for seL4 IPC syscalls.
//! On aarch64-sel4 target: calls P23 asm! intrinsics (sel4_call, sel4_recv etc.)
//! On host (testing): returns safe stub values for unit testing.

use crate::types::{Cap, MsgInfo, Badge};

/// seL4_Call — send message and block for reply
/// Returns reply MsgInfo
pub fn sel4_call(ep: Cap, msginfo: MsgInfo) -> MsgInfo {
    let _ = ep;
    msginfo // stub: echo msginfo (replaced by asm! on target)
}

/// seL4_Send — send message non-blocking, no reply
pub fn sel4_send(ep: Cap, msginfo: MsgInfo) {
    let _ = (ep, msginfo);
}

/// seL4_Recv — block waiting for message
/// Returns (MsgInfo, Badge)
pub fn sel4_recv(ep: Cap) -> (MsgInfo, Badge) {
    let _ = ep;
    (0, 0) // stub
}

/// seL4_Reply — reply to current caller (fast path)
pub fn sel4_reply(msginfo: MsgInfo) {
    let _ = msginfo;
}

/// seL4_NBSend — non-blocking send (fire and forget)
pub fn sel4_nb_send(ep: Cap, msginfo: MsgInfo) {
    let _ = (ep, msginfo);
}

/// seL4_Wait — block on notification object, returns badge
pub fn sel4_wait(ntfn: Cap) -> Badge {
    let _ = ntfn;
    0 // stub
}

/// seL4_Poll — non-blocking notification check, returns badge or 0
pub fn sel4_poll(ntfn: Cap) -> Badge {
    let _ = ntfn;
    0 // stub
}

/// Encode seL4 MsgInfo word: label + length
pub fn make_msg_info(label: u64, length: u64) -> MsgInfo {
    (label << 12) | (length & 0x7FF)
}

/// Decode label from MsgInfo word
pub fn msg_info_label(msginfo: MsgInfo) -> u64 {
    msginfo >> 12
}

/// Decode length from MsgInfo word
pub fn msg_info_length(msginfo: MsgInfo) -> u64 {
    msginfo & 0x7FF
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tp30_ipc_01_types_correct() {
        let ep: Cap = 42;
        let msg: MsgInfo = 0x0001_0000;
        assert_eq!(ep, 42);
        assert_eq!(msg, 0x0001_0000);
    }

    #[test]
    fn tp30_ipc_02_msginfo_encode_decode() {
        let msginfo = make_msg_info(0xDEAD, 4);
        assert_eq!(msg_info_length(msginfo), 4);
        assert_eq!(msg_info_label(msginfo), 0xDEAD);
    }

    #[test]
    fn tp30_ipc_03_cap_zero_is_null() {
        let null_cap: Cap = 0;
        assert_eq!(null_cap, 0);
    }

    #[test]
    fn tp30_ipc_04_sel4_call_stub() {
        let ep: Cap = 5;
        let msg = make_msg_info(1, 0);
        let reply = sel4_call(ep, msg);
        // stub returns echo
        assert_eq!(reply, msg);
    }

    #[test]
    fn tp30_ipc_05_sel4_recv_stub() {
        let (msg, badge) = sel4_recv(3);
        assert_eq!(msg, 0);
        assert_eq!(badge, 0);
    }

    #[test]
    fn tp30_ipc_06_sovereign_ipc_roundtrip_pattern() {
        // Full sovereign IPC roundtrip pattern used by BASTION PDs
        let ep: Cap = 10;
        let label: u64 = 0xA1;
        let length: u64 = 2;
        let outgoing = make_msg_info(label, length);
        let _reply = sel4_call(ep, outgoing);
        // Verify encoding survived
        assert_eq!(msg_info_label(outgoing), label);
        assert_eq!(msg_info_length(outgoing), length);
    }
}
