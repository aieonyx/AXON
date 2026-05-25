//! PalProcess for seL4 — TCB + capability space operations.
use axon_core::prelude::*;
use axon_core::types::U32;
use axon_pal::traits::PalProcess;
use crate::Sel4Pal;

impl PalProcess for Sel4Pal {
    fn args() -> AxonResult<&'static [&'static str]> {
        // seL4 bare metal: no argv. Args passed via IPC from root server.
        AxonResult::Ok(&[])
    }
    fn env_var(_key: &str) -> AxonResult<&'static str> {
        // seL4 has no environment variables. Config via IPC or boot image.
        AxonResult::Err(AxonError::not_found("seL4: no environment variables"))
    }
    fn exit(_code: U32) -> ! {
        // seL4: suspend root TCB. In practice, halt the processor.
        loop { unsafe { core::arch::asm!("wfe", options(nostack, nomem)); } }
    }
    fn pid() -> AxonResult<U32> {
        // seL4 has no PID — identity is the TCB capability.
        // Return the initial TCB cap slot index as a proxy.
        AxonResult::Ok(2) // CPtr::INIT_TCB.0 as u32
    }
}
