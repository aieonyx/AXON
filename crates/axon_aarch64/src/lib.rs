// Copyright (c) 2026 Edison Lepiten / AIEONYX
// axon_aarch64 P71 — AArch64 freestanding native codegen
//
// Turns aiXos Phoenix from "hosting AXONYX scripts" to "written in AXONYX."
// Emits AArch64 assembly (GNU syntax) for bare-metal aiXos — no OS, no libc.
//
// Conformance oracle: output of exec() on a script must match the behavior
// of the compiled native binary on the same script (P71 acceptance criterion).
//
// Pipeline:
//   .ax source → AxIr (IR) → AArch64 asm → .s file → assembled by GNU as
//
// Freestanding invariants:
//   - _start entry point (no CRT, no libc init)
//   - BSS zero loop before axon_main
//   - Stack pointer initialized from linker symbol
//   - No dynamic linking, no PLT, no GOT
//   - Position-independent: PIC-free (-fno-pic)

pub mod ir;
pub mod emit;
pub mod linker;
pub mod conformance;
