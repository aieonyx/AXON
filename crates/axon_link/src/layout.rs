// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
//
// Memory layout resolver.
// Places ELF sections at correct virtual addresses given a base address.
// Enforces 4K page alignment between segments.

use crate::elf::{ElfBinary, SEC_BSS, SEC_DATA, SEC_RODATA, SEC_TEXT, ALIGN_4K};
use crate::error::{LinkError, LinkResult};
use axon_std_string::AxString;

#[derive(Debug, Clone)]
pub struct MemoryLayout {
    pub text_addr:   u64,
    pub text_size:   u64,
    pub rodata_addr: u64,
    pub rodata_size: u64,
    pub data_addr:   u64,
    pub data_size:   u64,
    pub bss_addr:    u64,
    pub bss_size:    u64,
}

fn align_up(addr: u64, align: u64) -> u64 {
    (addr + align - 1) & !(align - 1)
}

/// Resolve section placement given a base virtual address.
pub fn elf_resolve_layout(bin: &ElfBinary, base_addr: u64) -> LinkResult<MemoryLayout> {
    let mut cursor = base_addr;

    // .text
    let text_addr = cursor;
    let text_size: u64 = bin.sections.iter()
        .filter(|s| s.flags & SEC_TEXT != 0)
        .map(|s| s.data.len() as u64)
        .sum();
    if text_size == 0 {
        return Err(LinkError::LayoutError(
            AxString::ax_from_str("no .text section found")
        ));
    }
    cursor = align_up(text_addr + text_size, ALIGN_4K);

    // .rodata
    let rodata_addr = cursor;
    let rodata_size: u64 = bin.sections.iter()
        .filter(|s| s.flags & SEC_RODATA != 0)
        .map(|s| s.data.len() as u64)
        .sum();
    cursor = align_up(rodata_addr + rodata_size.max(1), ALIGN_4K);

    // .data
    let data_addr = cursor;
    let data_size: u64 = bin.sections.iter()
        .filter(|s| s.flags & SEC_DATA != 0)
        .map(|s| s.data.len() as u64)
        .sum();
    cursor = align_up(data_addr + data_size.max(1), ALIGN_4K);

    // .bss
    let bss_addr = cursor;
    let bss_size: u64 = bin.sections.iter()
        .filter(|s| s.flags & SEC_BSS != 0)
        .map(|s| s.data.len() as u64)
        .sum();

    Ok(MemoryLayout {
        text_addr, text_size,
        rodata_addr, rodata_size,
        data_addr, data_size,
        bss_addr, bss_size,
    })
}
