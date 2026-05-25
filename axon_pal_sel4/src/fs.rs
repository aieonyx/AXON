//! PalFs for seL4 — IPC to a filesystem server protection domain.
//!
//! seL4 has no native filesystem. The AXON root task communicates with
//! a filesystem server PD via IPC. The FS server endpoint is at CPtr(17).

use axon_core::prelude::*;
use axon_pal::{traits::PalFs, types::{AxonPath, FileStat, OpenFlags, RawFd}};
use crate::Sel4Pal;

impl PalFs for Sel4Pal {
    fn open(_path: &AxonPath, _flags: OpenFlags) -> AxonResult<RawFd> {
        AxonResult::Err(AxonError::not_implemented("seL4 FS: requires filesystem server IPC"))
    }
    fn close(_fd: RawFd) -> AxonResult<()> {
        AxonResult::Err(AxonError::not_implemented("seL4 FS: requires filesystem server IPC"))
    }
    fn stat(_path: &AxonPath) -> AxonResult<FileStat> {
        AxonResult::Err(AxonError::not_implemented("seL4 FS: requires filesystem server IPC"))
    }
    fn mkdir(_path: &AxonPath, _mode: u32) -> AxonResult<()> {
        AxonResult::Err(AxonError::not_implemented("seL4 FS: requires filesystem server IPC"))
    }
    fn remove(_path: &AxonPath) -> AxonResult<()> {
        AxonResult::Err(AxonError::not_implemented("seL4 FS: requires filesystem server IPC"))
    }
    fn rename(_from: &AxonPath, _to: &AxonPath) -> AxonResult<()> {
        AxonResult::Err(AxonError::not_implemented("seL4 FS: requires filesystem server IPC"))
    }
}
