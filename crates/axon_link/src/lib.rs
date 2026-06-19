// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
//
// axon_link — ELF assembler, layout resolver, seL4 image, sovereign signing.
// Internal linker layer. Not ARPi-exposed.
// External exposure via ARPi boundary defined at AWP layer.
//
// Closes: DEFER-P45-001 (seL4 IPC wiring complete via sel4_image::sel4_ipc_wire)

pub mod elf;
pub mod emit;
pub mod error;
pub mod layout;
pub mod sel4_image;
pub mod signer;

pub use elf::{elf_add_section, elf_new, ElfBinary, ElfSection, EM_AARCH64, EM_X86_64};
pub use emit::elf_emit;
pub use error::{LinkError, LinkResult};
pub use layout::{elf_resolve_layout, MemoryLayout};
pub use sel4_image::{sel4_image_verify, sel4_image_wrap, sel4_ipc_wire};
pub use signer::{sig_append, sig_verify_present, sign_stub, SovereignSig};
