// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
//
// axonc compiler driver — orchestrates the ten sovereign pipeline stages.
// This is axonc-rust (stage 0). Retires when axonc.ax is self-compiled.
//
// Pipeline:
//   source → P49 lex → P50 parse → P51 hir → P52 infer
//          → P53 codegen → P54 native → Vec<u8>
//
// ELF output wraps native bytes with a minimal ELF64 PT_LOAD header.
// Verified against the ELF-64 Object File Format Specification.

use axon_native::native_codegen_source;
use axon_std_string::AxString;

// ── Error types ───────────────────────────────────────────────────────────────

#[derive(Debug)]
pub enum CompileError {
    Native(AxString),
    Io(AxString),
}

impl core::fmt::Display for CompileError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            CompileError::Native(e) => write!(f, "axonc native: {}", e.as_str()),
            CompileError::Io(e)     => write!(f, "axonc io: {}", e.as_str()),
        }
    }
}

pub type CompileResult<T> = Result<T, CompileError>;

// ── Public API ────────────────────────────────────────────────────────────────

/// Compile an AXON source string to raw x86_64 machine bytes.
/// Runs all ten pipeline stages (P45–P54) in sequence.
/// Deterministic: identical input always produces identical output.
pub fn compile(source: &str) -> CompileResult<Vec<u8>> {
    native_codegen_source(source).map_err(|e| {
        CompileError::Native(AxString::ax_from_str(&format!("{}", e)))
    })
}

/// Compile an AXON source string and write a minimal ELF64 executable.
/// The output binary is x86_64-linux-gnu, System V AMD64 ABI.
pub fn compile_elf(source: &str, output_path: &str) -> CompileResult<()> {
    let code = compile(source)?;
    let elf  = wrap_elf64(&code);
    std::fs::write(output_path, &elf).map_err(|e| {
        CompileError::Io(AxString::ax_from_str(&format!("{}", e)))
    })
}

// ── ELF64 wrapper ─────────────────────────────────────────────────────────────

/// Wrap raw x86_64 machine bytes in a minimal ELF64 executable.
///
/// Layout:
///   [ELF header 64B][PT_LOAD header 56B][code N bytes]
///
/// One PT_LOAD segment covers the entire file.
/// Entry point = LOAD_ADDR + 120 (= 0x401078).
fn wrap_elf64(code: &[u8]) -> Vec<u8> {
    const LOAD_ADDR: u64 = 0x0040_1000;
    const ELF_HDR:   u64 = 64;
    const PHDR_SIZE: u64 = 56;
    const HEADERS:   u64 = ELF_HDR + PHDR_SIZE;

    let entry     = LOAD_ADDR + HEADERS;
    let file_size = HEADERS + code.len() as u64;

    let mut buf: Vec<u8> = Vec::with_capacity(file_size as usize);

    // ── ELF header (64 bytes) ─────────────────────────────────────────────
    // e_ident[16]
    buf.extend_from_slice(&[0x7f, b'E', b'L', b'F']); // magic
    buf.push(2);                                               // ELFCLASS64
    buf.push(1);                                               // ELFDATA2LSB
    buf.push(1);                                               // EV_CURRENT
    buf.push(0);                                               // ELFOSABI_NONE
    buf.extend_from_slice(&[0u8; 8]);                         // padding
    // e_type … e_shstrndx
    buf.extend_from_slice(&2u16.to_le_bytes());    // ET_EXEC
    buf.extend_from_slice(&0x3eu16.to_le_bytes()); // EM_X86_64
    buf.extend_from_slice(&1u32.to_le_bytes());    // e_version
    buf.extend_from_slice(&entry.to_le_bytes());   // e_entry
    buf.extend_from_slice(&64u64.to_le_bytes());   // e_phoff
    buf.extend_from_slice(&0u64.to_le_bytes());    // e_shoff (none)
    buf.extend_from_slice(&0u32.to_le_bytes());    // e_flags
    buf.extend_from_slice(&64u16.to_le_bytes());   // e_ehsize
    buf.extend_from_slice(&56u16.to_le_bytes());   // e_phentsize
    buf.extend_from_slice(&1u16.to_le_bytes());    // e_phnum
    buf.extend_from_slice(&64u16.to_le_bytes());   // e_shentsize (unused)
    buf.extend_from_slice(&0u16.to_le_bytes());    // e_shnum
    buf.extend_from_slice(&0u16.to_le_bytes());    // e_shstrndx

    // ── PT_LOAD header (56 bytes) ─────────────────────────────────────────
    buf.extend_from_slice(&1u32.to_le_bytes());           // PT_LOAD
    buf.extend_from_slice(&5u32.to_le_bytes());           // PF_R | PF_X
    buf.extend_from_slice(&0u64.to_le_bytes());           // p_offset
    buf.extend_from_slice(&LOAD_ADDR.to_le_bytes());      // p_vaddr
    buf.extend_from_slice(&LOAD_ADDR.to_le_bytes());      // p_paddr
    buf.extend_from_slice(&file_size.to_le_bytes());      // p_filesz
    buf.extend_from_slice(&file_size.to_le_bytes());      // p_memsz
    buf.extend_from_slice(&0x1000u64.to_le_bytes());      // p_align

    // ── Code ─────────────────────────────────────────────────────────────
    buf.extend_from_slice(code);

    buf
}
