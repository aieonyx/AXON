//! seL4 sovereign types
//! Copyright (c) 2026 Edison Lepiten / AIEONYX

/// seL4 capability slot (u64 index into CNode)
pub type Cap = u64;
/// seL4 message info word (label + length encoded)
pub type MsgInfo = u64;
/// seL4 badge value carried on IPC
pub type Badge = u64;
/// seL4 error code (0 = success)
pub type Error = u64;
/// seL4 untyped memory object size bits
pub type SizeBits = u64;

/// seL4 syscall numbers (aarch64)
pub mod syscall {
    pub const SEND:    u64 = 6;
    pub const RECV:    u64 = 2;
    pub const CALL:    u64 = 3;
    pub const REPLY:   u64 = 4;
    pub const NOTIFY:  u64 = 6;
    pub const WAIT:    u64 = 7;
    pub const POLL:    u64 = 8;
    pub const YIELD:   u64 = 10;
}

/// seL4 object types for Untyped_Retype
pub mod obj_type {
    pub const UNTYPED:      u64 = 0;
    pub const TCB:          u64 = 1;
    pub const ENDPOINT:     u64 = 2;
    pub const NOTIFICATION: u64 = 3;
    pub const CNODE:        u64 = 4;
    pub const SMALL_PAGE:   u64 = 15;
    pub const LARGE_PAGE:   u64 = 16;
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn tp30_types_01_syscall_numbers() {
        assert_eq!(syscall::CALL, 3);
        assert_eq!(syscall::RECV, 2);
        assert_eq!(syscall::SEND, 6);
        assert_eq!(syscall::WAIT, 7);
        assert_eq!(syscall::POLL, 8);
    }
    #[test]
    fn tp30_types_02_obj_types() {
        assert_eq!(obj_type::ENDPOINT, 2);
        assert_eq!(obj_type::NOTIFICATION, 3);
        assert_eq!(obj_type::TCB, 1);
    }
}
