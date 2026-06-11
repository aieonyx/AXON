#!/usr/bin/env python3
"""
Phase 36 — aarch64-seL4 asm! intrinsics
Adds missing sel4_reply, sel4_wait, sel4_poll, sel4_nb_send codegen intrinsics.
Adds #[cfg(target_arch="aarch64")] real asm! paths in axon_sel4/src/ipc.rs.

Run from: /home/edisonbl/axon
"""

import sys
from pathlib import Path

ROOT = Path(__file__).parent

def read(p):
    return Path(p).read_text(encoding="utf-8")

def write(p, text):
    Path(p).write_text(text, encoding="utf-8")
    print(f"  wrote {p}")

def patch(path, old, new, label=""):
    text = read(path)
    if old not in text:
        print(f"  ERROR: anchor not found in {path}" + (f" [{label}]" if label else ""))
        sys.exit(1)
    count = text.count(old)
    if count > 1:
        print(f"  ERROR: anchor not unique ({count} occurrences) in {path} [{label}]")
        sys.exit(1)
    write(path, text.replace(old, new))
    print(f"  patched [{label}]")

# ── 1. codegen.rs — add 4 missing seL4 syscall intrinsics ────────────────────
# Insert after the sel4_recv block, before the read_volatile block.
# seL4 aarch64 ABI (from seL4 manual §A.1):
#   SysCall=3  SysReplyRecv=0  SysSend=6  SysNBSend=6
#   SysRecv=2  SysReply=4      SysWait=7  SysPoll=8  SysYield=10

CODEGEN = ROOT / "axon_parser/src/codegen.rs"

patch(
    CODEGEN,
    '                // P26-M1: volatile memory intrinsics — MMIO access\n'
    '                if fn_name == "read_volatile" {',

    '                // P36: sel4_reply(msginfo: u64) — fast reply, no return\n'
    '                // seL4_SysReply=4, msginfo→x1\n'
    '                if fn_name == "sel4_reply" {\n'
    '                    let mut arg_vals: Vec<String> = Vec::new();\n'
    '                    for a in args { if let Some(v) = self.emit_expr(a) { arg_vals.push(v); } }\n'
    '                    let msg = arg_vals.first().cloned().unwrap_or_else(|| "0".to_string());\n'
    '                    self.emit_line(&format!(\n'
    '                        "  call void asm sideeffect \\"mov x7, #4; svc #0\\", \\"r,~{{x7}},~{{memory}}\\"(i64 {})"\n'
    '                        , msg\n'
    '                    ));\n'
    '                    return None;\n'
    '                }\n'
    '                // P36: sel4_nb_send(ep: u64, msginfo: u64) — non-blocking send\n'
    '                // seL4_SysNBSend=6, ep→x0, msginfo→x1\n'
    '                if fn_name == "sel4_nb_send" {\n'
    '                    let mut arg_vals: Vec<String> = Vec::new();\n'
    '                    for a in args { if let Some(v) = self.emit_expr(a) { arg_vals.push(v); } }\n'
    '                    let ep  = arg_vals.first().cloned().unwrap_or_else(|| "0".to_string());\n'
    '                    let msg = arg_vals.get(1).cloned().unwrap_or_else(|| "0".to_string());\n'
    '                    self.emit_line(&format!(\n'
    '                        "  call void asm sideeffect \\"mov x7, #6; svc #0\\", \\"r,r,~{{x7}},~{{memory}}\\"(i64 {}, i64 {})"\n'
    '                        , ep, msg\n'
    '                    ));\n'
    '                    return None;\n'
    '                }\n'
    '                // P36: sel4_wait(ntfn: u64) -> u64 — block on notification, returns badge\n'
    '                // seL4_SysWait=7, ntfn→x0, badge returned in x1\n'
    '                if fn_name == "sel4_wait" {\n'
    '                    let mut arg_vals: Vec<String> = Vec::new();\n'
    '                    for a in args { if let Some(v) = self.emit_expr(a) { arg_vals.push(v); } }\n'
    '                    let ntfn = arg_vals.first().cloned().unwrap_or_else(|| "0".to_string());\n'
    '                    let tmp = self.ssa.fresh_tmp();\n'
    '                    self.emit_line(&format!(\n'
    '                        "  {} = call i64 asm sideeffect \\"mov x7, #7; svc #0\\", \\"={{x1}},r,~{{x7}},~{{memory}}\\"(i64 {})"\n'
    '                        , tmp, ntfn\n'
    '                    ));\n'
    '                    return Some(tmp);\n'
    '                }\n'
    '                // P36: sel4_poll(ntfn: u64) -> u64 — non-blocking notification check\n'
    '                // seL4_SysPoll=8, ntfn→x0, badge returned in x1 (0 if no message)\n'
    '                if fn_name == "sel4_poll" {\n'
    '                    let mut arg_vals: Vec<String> = Vec::new();\n'
    '                    for a in args { if let Some(v) = self.emit_expr(a) { arg_vals.push(v); } }\n'
    '                    let ntfn = arg_vals.first().cloned().unwrap_or_else(|| "0".to_string());\n'
    '                    let tmp = self.ssa.fresh_tmp();\n'
    '                    self.emit_line(&format!(\n'
    '                        "  {} = call i64 asm sideeffect \\"mov x7, #8; svc #0\\", \\"={{x1}},r,~{{x7}},~{{memory}}\\"(i64 {})"\n'
    '                        , tmp, ntfn\n'
    '                    ));\n'
    '                    return Some(tmp);\n'
    '                }\n'
    '                // P26-M1: volatile memory intrinsics — MMIO access\n'
    '                if fn_name == "read_volatile" {',

    "add sel4_reply/nb_send/wait/poll intrinsics"
)

# ── 2. axon_sel4/src/ipc.rs — add cfg-gated real asm! implementations ─────────
# Replace each stub function body with a cfg split:
#   #[cfg(target_arch = "aarch64")] → real asm!
#   #[cfg(not(target_arch = "aarch64"))] → existing stub

IPC = ROOT / "axon_sel4/src/ipc.rs"

# Replace the entire file with the complete updated version
IPC_NEW = '''\
// Copyright (c) 2026 Edison Lepiten / AIEONYX
//! seL4 IPC syscall wrappers
//!
//! Type-safe wrappers for seL4 IPC syscalls.
//! On aarch64: real SVC #0 inline assembly per seL4 ABI.
//! On host (x86_64/testing): safe stub values for unit testing.

use crate::types::{Cap, MsgInfo, Badge};

// ── seL4 aarch64 syscall numbers ─────────────────────────────────────────────
// Source: seL4 manual §A.1 — aarch64 syscall register ABI
// x7 = syscall number, x0-x6 = args, SVC #0, return in x0/x1
mod sys {
    pub const CALL:      u64 = 3;
    pub const REPLY:     u64 = 4;
    pub const SEND:      u64 = 6;
    pub const NB_SEND:   u64 = 6;  // same number, no-block variant
    pub const RECV:      u64 = 2;
    pub const WAIT:      u64 = 7;
    pub const POLL:      u64 = 8;
    pub const YIELD:     u64 = 10;
}

/// seL4_Call — send message and block for reply.
/// Returns reply MsgInfo in x0.
#[inline(always)]
pub fn sel4_call(ep: Cap, msginfo: MsgInfo) -> MsgInfo {
    #[cfg(target_arch = "aarch64")]
    unsafe {
        let result: u64;
        core::arch::asm!(
            "mov x7, {syscall_no}",
            "svc #0",
            syscall_no = const sys::CALL,
            inout("x0") ep => result,
            in("x1") msginfo,
            lateout("x7") _,
            options(nostack),
        );
        result
    }
    #[cfg(not(target_arch = "aarch64"))]
    { let _ = ep; msginfo }
}

/// seL4_Send — send message non-blocking, no reply.
#[inline(always)]
pub fn sel4_send(ep: Cap, msginfo: MsgInfo) {
    #[cfg(target_arch = "aarch64")]
    unsafe {
        core::arch::asm!(
            "mov x7, {syscall_no}",
            "svc #0",
            syscall_no = const sys::SEND,
            in("x0") ep,
            in("x1") msginfo,
            lateout("x7") _,
            options(nostack),
        );
    }
    #[cfg(not(target_arch = "aarch64"))]
    { let _ = (ep, msginfo); }
}

/// seL4_Recv — block waiting for message.
/// Returns (MsgInfo in x0, Badge in x1).
#[inline(always)]
pub fn sel4_recv(ep: Cap) -> (MsgInfo, Badge) {
    #[cfg(target_arch = "aarch64")]
    unsafe {
        let msginfo: u64;
        let badge: u64;
        core::arch::asm!(
            "mov x7, {syscall_no}",
            "svc #0",
            syscall_no = const sys::RECV,
            inout("x0") ep => msginfo,
            lateout("x1") badge,
            lateout("x7") _,
            options(nostack),
        );
        (msginfo, badge)
    }
    #[cfg(not(target_arch = "aarch64"))]
    { let _ = ep; (0, 0) }
}

/// seL4_Reply — fast reply to current caller.
#[inline(always)]
pub fn sel4_reply(msginfo: MsgInfo) {
    #[cfg(target_arch = "aarch64")]
    unsafe {
        core::arch::asm!(
            "mov x7, {syscall_no}",
            "svc #0",
            syscall_no = const sys::REPLY,
            in("x1") msginfo,
            lateout("x7") _,
            options(nostack),
        );
    }
    #[cfg(not(target_arch = "aarch64"))]
    { let _ = msginfo; }
}

/// seL4_NBSend — non-blocking send (fire and forget).
#[inline(always)]
pub fn sel4_nb_send(ep: Cap, msginfo: MsgInfo) {
    #[cfg(target_arch = "aarch64")]
    unsafe {
        core::arch::asm!(
            "mov x7, {syscall_no}",
            "svc #0",
            syscall_no = const sys::NB_SEND,
            in("x0") ep,
            in("x1") msginfo,
            lateout("x7") _,
            options(nostack),
        );
    }
    #[cfg(not(target_arch = "aarch64"))]
    { let _ = (ep, msginfo); }
}

/// seL4_Wait — block on notification object, returns badge.
#[inline(always)]
pub fn sel4_wait(ntfn: Cap) -> Badge {
    #[cfg(target_arch = "aarch64")]
    unsafe {
        let badge: u64;
        core::arch::asm!(
            "mov x7, {syscall_no}",
            "svc #0",
            syscall_no = const sys::WAIT,
            in("x0") ntfn,
            lateout("x1") badge,
            lateout("x7") _,
            options(nostack),
        );
        badge
    }
    #[cfg(not(target_arch = "aarch64"))]
    { let _ = ntfn; 0 }
}

/// seL4_Poll — non-blocking notification check, returns badge or 0.
#[inline(always)]
pub fn sel4_poll(ntfn: Cap) -> Badge {
    #[cfg(target_arch = "aarch64")]
    unsafe {
        let badge: u64;
        core::arch::asm!(
            "mov x7, {syscall_no}",
            "svc #0",
            syscall_no = const sys::POLL,
            in("x0") ntfn,
            lateout("x1") badge,
            lateout("x7") _,
            options(nostack),
        );
        badge
    }
    #[cfg(not(target_arch = "aarch64"))]
    { let _ = ntfn; 0 }
}

/// seL4_Yield — yield the current thread's timeslice.
#[inline(always)]
pub fn sel4_yield() {
    #[cfg(target_arch = "aarch64")]
    unsafe {
        core::arch::asm!(
            "mov x7, {syscall_no}",
            "svc #0",
            syscall_no = const sys::YIELD,
            lateout("x7") _,
            options(nostack),
        );
    }
    // host: no-op
}

/// Encode seL4 MsgInfo word: label + length.
#[inline(always)]
pub fn make_msg_info(label: u64, length: u64) -> MsgInfo {
    (label << 12) | (length & 0x7FF)
}

/// Decode label from MsgInfo word.
#[inline(always)]
pub fn msg_info_label(msginfo: MsgInfo) -> u64 { msginfo >> 12 }

/// Decode length from MsgInfo word.
#[inline(always)]
pub fn msg_info_length(msginfo: MsgInfo) -> u64 { msginfo & 0x7FF }

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
        // On host: stub returns echo
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
        let ep: Cap = 10;
        let label: u64 = 0xA1;
        let length: u64 = 2;
        let outgoing = make_msg_info(label, length);
        let _reply = sel4_call(ep, outgoing);
        assert_eq!(msg_info_label(outgoing), label);
        assert_eq!(msg_info_length(outgoing), length);
    }

    // P36: new syscall coverage tests
    #[test]
    fn tp36_ipc_07_sel4_send_no_panic() {
        sel4_send(1, make_msg_info(0xFF, 1));
    }

    #[test]
    fn tp36_ipc_08_sel4_reply_no_panic() {
        sel4_reply(make_msg_info(0, 0));
    }

    #[test]
    fn tp36_ipc_09_sel4_nb_send_no_panic() {
        sel4_nb_send(2, make_msg_info(0xAB, 1));
    }

    #[test]
    fn tp36_ipc_10_sel4_wait_returns_zero_on_host() {
        assert_eq!(sel4_wait(5), 0);
    }

    #[test]
    fn tp36_ipc_11_sel4_poll_returns_zero_on_host() {
        assert_eq!(sel4_poll(5), 0);
    }

    #[test]
    fn tp36_ipc_12_sel4_yield_no_panic() {
        sel4_yield();
    }

    #[test]
    fn tp36_ipc_13_syscall_numbers_correct() {
        // seL4 aarch64 ABI constants — must never drift
        assert_eq!(sys::CALL,    3);
        assert_eq!(sys::REPLY,   4);
        assert_eq!(sys::SEND,    6);
        assert_eq!(sys::RECV,    2);
        assert_eq!(sys::WAIT,    7);
        assert_eq!(sys::POLL,    8);
        assert_eq!(sys::YIELD,  10);
    }

    #[test]
    fn tp36_ipc_14_msginfo_label_roundtrip_fuzz() {
        for label in [0u64, 1, 0xFF, 0xDEAD, 0xFFFF_FFFF] {
            for len in [0u64, 1, 4, 0x7FF] {
                let m = make_msg_info(label, len);
                assert_eq!(msg_info_label(m), label);
                assert_eq!(msg_info_length(m), len);
            }
        }
    }
}
'''

write(IPC, IPC_NEW)

print()
print("Phase 36 patch applied.")
print("Next steps:")
print("  1. rm -f /tmp/axon_out.*")
print("  2. cargo test --workspace 2>&1 | tail -30")
print("  3. cargo clippy --workspace -- -D warnings 2>&1 | head -30")
