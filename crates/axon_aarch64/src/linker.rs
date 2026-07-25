// Copyright (c) 2026 Edison Lepiten / AIEONYX
// axon_aarch64 P71 — Linker script generator for aiXos bare-metal

/// Generate a freestanding linker script for aiXos Phoenix (AArch64).
/// Load address: 0x40080000 (after QEMU UEFI stub, matches aiXos PL-13+)
pub fn generate_linker_script(load_addr: u64, stack_size: usize) -> String {
    format!(r#"/* Copyright (c) 2026 Edison Lepiten / AIEONYX */
/* axon_aarch64 P71 — Sovereign freestanding linker script */
/* Target: AArch64 bare-metal aiXos Phoenix */

ENTRY(_start)

MEMORY {{
  FLASH (rx) : ORIGIN = {:#010x}, LENGTH = 64M
  RAM   (rwx): ORIGIN = 0x48000000, LENGTH = 32M
}}

SECTIONS {{
  . = {:#010x};

  .text : {{
    KEEP(*(.text._start))
    *(.text*)
  }} > FLASH

  .rodata : {{
    *(.rodata*)
  }} > FLASH

  .data : {{
    *(.data*)
  }} > RAM

  .bss : {{
    __bss_start = .;
    *(.bss*)
    *(COMMON)
    __bss_end = .;
  }} > RAM

  /* Stack at top of RAM */
  . = ORIGIN(RAM) + LENGTH(RAM) - {:#x};
  __stack_bottom = .;
  . += {:#x};
  __stack_top = .;
}}
"#, load_addr, load_addr, stack_size, stack_size)
}

/// Default aiXos Phoenix load address (after UEFI stub)
pub const AIXOS_LOAD_ADDR: u64 = 0x40080000;
/// Default stack size (256KB)
pub const DEFAULT_STACK_SIZE: usize = 256 * 1024;

/// Generate build commands for a freestanding .ax compilation
pub fn freestanding_build_commands(
    asm_file: &str,
    obj_file: &str,
    elf_file: &str,
    linker_script: &str,
) -> Vec<String> {
    vec![
        format!("aarch64-linux-gnu-as -o {} {}", obj_file, asm_file),
        format!("aarch64-linux-gnu-ld -T {} -o {} {}",
            linker_script, elf_file, obj_file),
    ]
}
