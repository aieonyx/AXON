// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
// P48 QA — axon_link test suite
// Pass bar: 10/10 before P49 begins.

use axon_link::{
    elf_add_section, elf_emit, elf_new, elf_resolve_layout,
    sel4_ipc_wire,
    sig_append, sig_verify_present, sign_stub,
    ElfSection, LinkError, EM_X86_64,
};
use axon_link::elf::{SEC_TEXT, SEC_RODATA, SEC_DATA, ELFMAG};
use tempfile::tempdir;

// T1: empty ELF binary created with correct entry point
#[test]
fn test_elf_new() {
    let bin = elf_new(0x400000, EM_X86_64);
    assert_eq!(bin.entry, 0x400000);
    assert_eq!(bin.arch, EM_X86_64);
    assert!(bin.sections.is_empty());
}

// T2: section added with correct name and data
#[test]
fn test_elf_add_section() {
    let mut bin = elf_new(0x400000, EM_X86_64);
    let sec = ElfSection::new(".text", vec![0x48, 0x31, 0xc0, 0xc3], SEC_TEXT);
    elf_add_section(&mut bin, sec);
    assert_eq!(bin.sections.len(), 1);
    assert_eq!(bin.sections[0].name.as_str(), ".text");
    assert_eq!(bin.sections[0].data, vec![0x48, 0x31, 0xc0, 0xc3]);
}

// T3: .text section placed at base_addr
#[test]
fn test_elf_layout_text() {
    let mut bin = elf_new(0x400000, EM_X86_64);
    elf_add_section(&mut bin, ElfSection::new(".text", vec![0u8; 64], SEC_TEXT));
    let layout = elf_resolve_layout(&bin, 0x400000).unwrap();
    assert_eq!(layout.text_addr, 0x400000);
    assert_eq!(layout.text_size, 64);
}

// T4: .data section placed after .text + .rodata (page-aligned)
#[test]
fn test_elf_layout_data() {
    let mut bin = elf_new(0x400000, EM_X86_64);
    elf_add_section(&mut bin, ElfSection::new(".text",   vec![0u8; 64],  SEC_TEXT));
    elf_add_section(&mut bin, ElfSection::new(".rodata", vec![0u8; 32],  SEC_RODATA));
    elf_add_section(&mut bin, ElfSection::new(".data",   vec![0u8; 16],  SEC_DATA));
    let layout = elf_resolve_layout(&bin, 0x400000).unwrap();
    // .data must be at a higher address than .text + .rodata
    assert!(layout.data_addr > layout.rodata_addr);
    // Must be 4K aligned
    assert_eq!(layout.data_addr % 0x1000, 0);
}

// T5: binary written to disk
#[test]
fn test_elf_emit() {
    let dir = tempdir().unwrap();
    let out = dir.path().join("test.elf");
    let mut bin = elf_new(0x400000, EM_X86_64);
    elf_add_section(&mut bin, ElfSection::new(".text", vec![0x90u8; 16], SEC_TEXT));
    let layout = elf_resolve_layout(&bin, 0x400000).unwrap();
    elf_emit(&bin, &layout, out.to_str().unwrap()).unwrap();
    assert!(out.exists());
    assert!(out.metadata().unwrap().len() > 0);
}

// T6: emitted binary starts with ELF magic
#[test]
fn test_elf_magic() {
    let dir = tempdir().unwrap();
    let out = dir.path().join("magic.elf");
    let mut bin = elf_new(0x400000, EM_X86_64);
    elf_add_section(&mut bin, ElfSection::new(".text", vec![0x90u8; 16], SEC_TEXT));
    let layout = elf_resolve_layout(&bin, 0x400000).unwrap();
    elf_emit(&bin, &layout, out.to_str().unwrap()).unwrap();
    let bytes = std::fs::read(&out).unwrap();
    assert_eq!(&bytes[0..4], &ELFMAG);
}

// T7: sign_stub is deterministic — same input same output
#[test]
fn test_sign_stub() {
    let data = b"axon sovereign binary";
    let sig1 = sign_stub(data);
    let sig2 = sign_stub(data);
    assert_eq!(sig1, sig2, "signing must be deterministic");
    assert_eq!(sig1.0.len(), 64);
    // Signature must not be all zeros
    assert!(sig1.0.iter().any(|&b| b != 0));
}

// T8: signature appended to binary file
#[test]
fn test_sig_append() {
    let dir = tempdir().unwrap();
    let out = dir.path().join("signed.elf");
    let mut bin = elf_new(0x400000, EM_X86_64);
    elf_add_section(&mut bin, ElfSection::new(".text", vec![0x90u8; 16], SEC_TEXT));
    let layout = elf_resolve_layout(&bin, 0x400000).unwrap();
    elf_emit(&bin, &layout, out.to_str().unwrap()).unwrap();

    let orig_size = std::fs::metadata(&out).unwrap().len();
    let data = std::fs::read(&out).unwrap();
    let sig = sign_stub(&data);
    sig_append(out.to_str().unwrap(), &sig).unwrap();

    let new_size = std::fs::metadata(&out).unwrap().len();
    assert_eq!(new_size, orig_size + 68); // 4 magic + 64 sig
    assert!(sig_verify_present(out.to_str().unwrap()).unwrap());
}

// T9: sel4_ipc_wire returns Ok — DEFER-P45-001 closed
#[test]
fn test_sel4_ipc_wire() {
    assert!(sel4_ipc_wire().is_ok());
}

// T10: all LinkError variants produce non-empty display message
#[test]
fn test_link_error_display() {
    use axon_std_string::AxString;
    let errors = vec![
        LinkError::IoError(AxString::ax_from_str("test")),
        LinkError::ElfFormatError(AxString::ax_from_str("test")),
        LinkError::LayoutError(AxString::ax_from_str("test")),
        LinkError::SigningError(AxString::ax_from_str("test")),
        LinkError::Sel4Error(AxString::ax_from_str("test")),
    ];
    for e in errors {
        let msg = format!("{}", e);
        assert!(!msg.is_empty(), "error display must not be empty");
        assert!(msg.contains("test"), "error must include context");
    }
}
