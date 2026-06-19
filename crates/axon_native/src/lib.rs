// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
//
// axon_native — x86_64 native codegen for AXONYX.
// Internal native backend. Not ARPi-exposed.
// External exposure via ARPi boundary defined at AWP layer.
//
// P54 landmark: AXON emits its own machine bytes.
// Closes DEFER-P53-001: if/else with conditional jumps.
// Bridge (native.rs) mirrors native.ax — excised at P55.

pub mod error;
pub mod native;
pub mod x86;

pub use error::{NativeError, NativeResult};
pub use native::{emit_program, native_codegen_source};
pub use x86::{
    call_rel32, je_rel32, jmp_rel32,
    load_rax_rbp_slot, mov_rax_imm32, mov_rax_param, mov_rbp_rsp,
    mov_rsp_rbp, pop_rbp, push_rbp, ret_byte, store_rax_rbp_slot,
    test_rax_rax,
};
