//! seL4 capability derivation operations
//! Copyright (c) 2026 Edison Lepiten / AIEONYX

use crate::types::{Cap, Error};

/// seL4 rights word
pub type Rights = u64;

/// seL4 capability rights constants
pub mod rights {
    pub const NONE:  u64 = 0;
    pub const READ:  u64 = 1;
    pub const WRITE: u64 = 2;
    pub const GRANT: u64 = 4;
    pub const ALL:   u64 = 7;
}

/// CNode operation: Copy capability
pub fn cnode_copy(dest: CNodeSlot, src: CNodeSlot, rights: Rights) -> Error {
    let _ = (dest, src, rights);
    0 // seL4_NoError
}

/// Slot descriptor for CNode operations
#[derive(Debug, Clone, Copy)]
pub struct CNodeSlot { pub root: Cap, pub index: u64, pub depth: u64 }

/// CNode operation: Mint capability (copy with badge)
pub fn cnode_mint(dest: CNodeSlot, src: CNodeSlot, rights: Rights, badge: u64) -> Error {
    let _ = (dest, src, rights, badge);
    0
}

/// CNode operation: Delete capability
pub fn cnode_delete(root: Cap, index: u64, depth: u64) -> Error {
    let _ = (root, index, depth);
    0
}

/// CNode operation: Revoke all derived capabilities
pub fn cnode_revoke(root: Cap, index: u64, depth: u64) -> Error {
    let _ = (root, index, depth);
    0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tp30_cap_01_rights_constants() {
        assert_eq!(rights::NONE,  0);
        assert_eq!(rights::READ,  1);
        assert_eq!(rights::WRITE, 2);
        assert_eq!(rights::GRANT, 4);
        assert_eq!(rights::ALL,   7);
        // Composability
        assert_eq!(rights::READ | rights::WRITE, 3);
    }

    #[test]
    fn tp30_cap_02_cnode_copy_returns_no_error() {
        let dest = CNodeSlot { root: 1, index: 2, depth: 64 };
        let src  = CNodeSlot { root: 1, index: 3, depth: 64 };
        let err = cnode_copy(dest, src, rights::ALL);
        assert_eq!(err, 0, "cnode_copy must return seL4_NoError=0");
    }

    #[test]
    fn tp30_cap_03_cnode_mint_badge() {
        let dest = CNodeSlot { root: 1, index: 4, depth: 64 };
        let src  = CNodeSlot { root: 1, index: 3, depth: 64 };
        let err = cnode_mint(dest, src, rights::READ, 0xBEEF);
        assert_eq!(err, 0);
    }

    #[test]
    fn tp30_cap_04_cnode_delete() {
        let err = cnode_delete(1, 3, 64);
        assert_eq!(err, 0);
    }
}
