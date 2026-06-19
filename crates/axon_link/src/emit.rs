// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
//
// Binary emission — assembles ELF64 binary and writes to disk.
// Uses axon_std_io for all file operations.

use crate::elf::{
    elf_header_bytes, program_header_bytes,
    ElfBinary, SEC_TEXT, SEC_RODATA, SEC_DATA,
    PF_R, PF_X, PF_W, ALIGN_4K,
};
use crate::layout::MemoryLayout;
use crate::error::{LinkError, LinkResult};
use axon_std_string::AxString;

/// Assemble and emit the ELF64 binary to disk.
pub fn elf_emit(bin: &ElfBinary, layout: &MemoryLayout, path: &str) -> LinkResult<()> {
    let mut output: Vec<u8> = Vec::new();

    // Collect section data
    let text_data: Vec<u8> = bin.sections.iter()
        .filter(|s| s.flags & SEC_TEXT != 0)
        .flat_map(|s| s.data.iter().copied())
        .collect();

    let rodata_data: Vec<u8> = bin.sections.iter()
        .filter(|s| s.flags & SEC_RODATA != 0)
        .flat_map(|s| s.data.iter().copied())
        .collect();

    let data_data: Vec<u8> = bin.sections.iter()
        .filter(|s| s.flags & SEC_DATA != 0)
        .flat_map(|s| s.data.iter().copied())
        .collect();

    // Layout:
    // [ELF header 64B] [PH text 56B] [PH data 56B] [.text] [.rodata] [.data]
    let elf_hdr_size:  u64 = 64;
    let ph_entry_size: u64 = 56;
    let ph_count:      u16 = 2;
    let phoff:         u64 = elf_hdr_size;
    let data_offset:   u64 = elf_hdr_size + ph_entry_size * ph_count as u64;

    let text_file_offset   = data_offset;
    let rodata_file_offset = text_file_offset + text_data.len() as u64;
    let data_file_offset   = rodata_file_offset + rodata_data.len() as u64;

    let text_memsz   = layout.text_size;
    let rodata_memsz = layout.rodata_size;
    let data_memsz   = layout.data_size;
    let bss_memsz    = layout.bss_size;

    // Section header table offset (after all section data)
    let shoff = data_offset
        + text_data.len() as u64
        + rodata_data.len() as u64
        + data_data.len() as u64;

    // shnum: null + text + rodata + data + bss = 5
    let shnum: u16 = 5;

    // ELF header
    let header = elf_header_bytes(bin, phoff, shoff, ph_count, shnum);
    output.extend_from_slice(&header);

    // Program header 1: PT_LOAD text+rodata (R+X)
    let mut ph1 = program_header_bytes(
        layout.text_addr,
        layout.text_addr,
        text_memsz + rodata_memsz,
        text_memsz + rodata_memsz,
        PF_R | PF_X,
        ALIGN_4K,
    );
    // Fix up file offset
    ph1[8..16].copy_from_slice(&text_file_offset.to_le_bytes());
    output.extend_from_slice(&ph1);

    // Program header 2: PT_LOAD data+bss (R+W)
    let mut ph2 = program_header_bytes(
        layout.data_addr,
        layout.data_addr,
        data_memsz,
        data_memsz + bss_memsz,
        PF_R | PF_W,
        ALIGN_4K,
    );
    ph2[8..16].copy_from_slice(&data_file_offset.to_le_bytes());
    output.extend_from_slice(&ph2);

    // Section data
    output.extend_from_slice(&text_data);
    output.extend_from_slice(&rodata_data);
    output.extend_from_slice(&data_data);

    // Section header table (minimal — 5 entries x 64 bytes)
    // SHT_NULL
    output.extend_from_slice(&[0u8; 64]);
    // .text
    output.extend_from_slice(&section_header_bytes(
        1, crate::elf::SHT_PROGBITS,
        crate::elf::SHF_ALLOC | crate::elf::SHF_EXECINSTR,
        layout.text_addr, text_file_offset, text_data.len() as u64, ALIGN_4K,
    ));
    // .rodata
    output.extend_from_slice(&section_header_bytes(
        2, crate::elf::SHT_PROGBITS,
        crate::elf::SHF_ALLOC,
        layout.rodata_addr, rodata_file_offset, rodata_data.len() as u64, ALIGN_4K,
    ));
    // .data
    output.extend_from_slice(&section_header_bytes(
        3, crate::elf::SHT_PROGBITS,
        crate::elf::SHF_ALLOC | crate::elf::SHF_WRITE,
        layout.data_addr, data_file_offset, data_data.len() as u64, ALIGN_4K,
    ));
    // .bss
    output.extend_from_slice(&section_header_bytes(
        4, crate::elf::SHT_NOBITS,
        crate::elf::SHF_ALLOC | crate::elf::SHF_WRITE,
        layout.bss_addr, 0, layout.bss_size, ALIGN_4K,
    ));

    std::fs::write(path, &output).map_err(|e| {
        LinkError::IoError(AxString::ax_from_str(&e.to_string()))
    })
}

/// Emit a minimal section header entry (64 bytes).
fn section_header_bytes(
    name_idx: u32, sh_type: u32, sh_flags: u64,
    addr: u64, offset: u64, size: u64, align: u64,
) -> Vec<u8> {
    let mut sh = vec![0u8; 64];
    sh[0..4].copy_from_slice(&name_idx.to_le_bytes());   // sh_name
    sh[4..8].copy_from_slice(&sh_type.to_le_bytes());    // sh_type
    sh[8..16].copy_from_slice(&sh_flags.to_le_bytes());  // sh_flags
    sh[16..24].copy_from_slice(&addr.to_le_bytes());     // sh_addr
    sh[24..32].copy_from_slice(&offset.to_le_bytes());   // sh_offset
    sh[32..40].copy_from_slice(&size.to_le_bytes());     // sh_size
    sh[40..44].copy_from_slice(&0u32.to_le_bytes());     // sh_link
    sh[44..48].copy_from_slice(&0u32.to_le_bytes());     // sh_info
    sh[48..56].copy_from_slice(&align.to_le_bytes());    // sh_addralign
    sh[56..64].copy_from_slice(&0u64.to_le_bytes());     // sh_entsize
    sh
}
