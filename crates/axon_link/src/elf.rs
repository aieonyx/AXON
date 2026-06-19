// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
//
// ELF64 sovereign assembler.
// Hand-written ELF64 header and section assembly — zero GNU ld dependency.
// Targets: x86_64-linux-gnu (POSIX dev) and aarch64-sel4 (sovereign).

use axon_std_string::AxString;

// ELF64 constants
pub const ELFMAG:        [u8; 4] = [0x7f, b'E', b'L', b'F'];
pub const ELFCLASS64:    u8      = 2;
pub const ELFDATA2LSB:   u8      = 1;   // little-endian
pub const ET_EXEC:       u16     = 2;   // executable
pub const EM_X86_64:     u16     = 62;
pub const EM_AARCH64:    u16     = 183;
pub const EV_CURRENT:    u8      = 1;
pub const PT_LOAD:       u32     = 1;
pub const PF_X:          u32     = 0x1; // execute
pub const PF_W:          u32     = 0x2; // write
pub const PF_R:          u32     = 0x4; // read
pub const SHT_NULL:      u32     = 0;
pub const SHT_PROGBITS:  u32     = 1;
pub const SHT_NOBITS:    u32     = 8;   // .bss
pub const SHF_ALLOC:     u64     = 0x2;
pub const SHF_EXECINSTR: u64     = 0x4;
pub const SHF_WRITE:     u64     = 0x1;
pub const ALIGN_4K:      u64     = 0x1000;

// Section flags
pub const SEC_TEXT:   u32 = 0x01;
pub const SEC_RODATA: u32 = 0x02;
pub const SEC_DATA:   u32 = 0x04;
pub const SEC_BSS:    u32 = 0x08;
pub const SEC_SIG:    u32 = 0x10;   // .axsig — AIEONYX sovereign signature

#[derive(Debug, Clone)]
pub struct ElfSection {
    pub name:  AxString,
    pub data:  Vec<u8>,
    pub flags: u32,
}

impl ElfSection {
    pub fn new(name: &str, data: Vec<u8>, flags: u32) -> Self {
        ElfSection {
            name: AxString::ax_from_str(name),
            data,
            flags,
        }
    }

    pub fn is_bss(&self) -> bool {
        self.flags & SEC_BSS != 0
    }
}

#[derive(Debug)]
pub struct ElfBinary {
    pub sections: Vec<ElfSection>,
    pub entry:    u64,
    pub arch:     u16,   // EM_X86_64 or EM_AARCH64
}

/// Create a new empty ELF binary with given entry point.
pub fn elf_new(entry: u64, arch: u16) -> ElfBinary {
    ElfBinary { sections: Vec::new(), entry, arch }
}

/// Add a section to the binary.
pub fn elf_add_section(bin: &mut ElfBinary, section: ElfSection) {
    bin.sections.push(section);
}

/// Emit the ELF64 header as raw bytes.
pub fn elf_header_bytes(bin: &ElfBinary, phoff: u64, shoff: u64, phnum: u16, shnum: u16) -> Vec<u8> {
    let mut h = vec![0u8; 64];

    // ELF magic
    h[0..4].copy_from_slice(&ELFMAG);
    h[4]  = ELFCLASS64;
    h[5]  = ELFDATA2LSB;
    h[6]  = EV_CURRENT;
    h[7]  = 0; // OS/ABI = System V
    // bytes 8..16 = padding (zero)

    // e_type = ET_EXEC
    h[16..18].copy_from_slice(&ET_EXEC.to_le_bytes());
    // e_machine
    h[18..20].copy_from_slice(&bin.arch.to_le_bytes());
    // e_version
    h[20..24].copy_from_slice(&(EV_CURRENT as u32).to_le_bytes());
    // e_entry
    h[24..32].copy_from_slice(&bin.entry.to_le_bytes());
    // e_phoff
    h[32..40].copy_from_slice(&phoff.to_le_bytes());
    // e_shoff
    h[40..48].copy_from_slice(&shoff.to_le_bytes());
    // e_flags
    h[48..52].copy_from_slice(&0u32.to_le_bytes());
    // e_ehsize = 64
    h[52..54].copy_from_slice(&64u16.to_le_bytes());
    // e_phentsize = 56
    h[54..56].copy_from_slice(&56u16.to_le_bytes());
    // e_phnum
    h[56..58].copy_from_slice(&phnum.to_le_bytes());
    // e_shentsize = 64
    h[58..60].copy_from_slice(&64u16.to_le_bytes());
    // e_shnum
    h[60..62].copy_from_slice(&shnum.to_le_bytes());
    // e_shstrndx = shnum - 1 (last section = .shstrtab)
    h[62..64].copy_from_slice(&(shnum.saturating_sub(1)).to_le_bytes());

    h
}

/// Emit a program header (PT_LOAD) entry — 56 bytes.
pub fn program_header_bytes(
    vaddr: u64, paddr: u64, filesz: u64, memsz: u64, flags: u32, align: u64,
) -> Vec<u8> {
    let mut ph = vec![0u8; 56];
    ph[0..4].copy_from_slice(&PT_LOAD.to_le_bytes());
    ph[4..8].copy_from_slice(&flags.to_le_bytes());
    ph[8..16].copy_from_slice(&0u64.to_le_bytes());  // offset filled by caller
    ph[16..24].copy_from_slice(&vaddr.to_le_bytes());
    ph[24..32].copy_from_slice(&paddr.to_le_bytes());
    ph[32..40].copy_from_slice(&filesz.to_le_bytes());
    ph[40..48].copy_from_slice(&memsz.to_le_bytes());
    ph[48..56].copy_from_slice(&align.to_le_bytes());
    ph
}
