# EXHIBIT.md — NLNet Evidence Package
## AIEONYX / AXON Compiler — axon_sel4 Rewrite Track
### Prepared: 2026-06-21 | Branch: sel4-rewrite | Commit: 01c4b1e | Tag: v0.sel4-m3

---

## CLAIM
AXON (AXONYX) is a sovereign systems programming language with its own compiler
that produces verified ELF objects targeting seL4 on aarch64 bare-metal.
This exhibit provides reproducible evidence of that claim.

---

## ARTIFACT 1 — Source: axon_sel4/ax/sovereign_echo.ax

AXON source file implementing a seL4 IPC round-trip proof:
- Structs: CPtr, MessageInfo, IpcResult
- FFI stubs resolved by axon_sel4 Rust shim at link time
- Logic: sovereign_send → axon_sel4_call → verify_echo
- Entry point: main() returns 1 on verified echo, 0 on failure

Parser constraints discovered and documented during development:
- Equality comparisons require literal RHS when structs are in scope
- Variable-to-variable equality expressed via diff() helper

## ARTIFACT 2 — Compiler Invocation
cargo run -p axon_cli -- build 
--target aarch64-sel4 
--profile seL4-strict 
axon_sel4/ax/sovereign_echo.ax
Output:
axon build: axon_sel4/ax/sovereign_echo.ax [profile: seL4-strict]

axon: targeting seL4 (aarch64-unknown-none-elf)

axon: seL4 object ready: sovereign_echo.o

axon: seL4 ABI check PASSED

axon: profile seL4-strict enforced

axon: binary ready for BASTION signing

## ARTIFACT 3 — Object File
ABI check confirms:
- Machine type: AArch64 (EM_AARCH64 = 183)
- No dynamic dependencies (bare-metal seL4 target)
- No forbidden libc symbols (printf, malloc, free, exit, syscall)
- Profile seL4-strict enforced throughout

## ARTIFACT 4 — Codegen Hardening (CF5–CF15)

During this track, 11 LLVM IR codegen bugs were discovered and fixed
in axon_parser/src/codegen.rs. These represent genuine compiler
hardening work surfaced by the seL4 target's stricter type requirements:

| Fix | Description |
|-----|-------------|
| CF5 | Infer/catch-all default i32→i64 (word-size correctness) |
| CF6 | Struct let-binding: memcpy not scalar store |
| CF7 | Named types → ptr/%%struct.Name in type emitters |
| CF8 | Let-binding type inferred from init expression kind |
| CF9 | Call arg types resolved via place_type_map |
| CF10 | Param place_struct_map for field access on struct params |
| CF11 | Struct-returning fn signatures emit ptr; ret uses current_fn_ret |
| CF12 | Call site return type resolved via fn_ret_map |
| CF13 | BinOp operand type via place_type_map |
| CF14 | Match scrutinee type via place_type_map |
| CF15 | Phi result type via current_fn_ret; BinOp init in Let handler |

## ARTIFACT 5 — Test Suite

All workspace tests pass post-hardening:
- 0 failed, 5 ignored (pre-existing platform skips)
- axon_parser codegen tests: tc_e2e_return_42, tc_e2e_arithmetic,
  tc_p14_integration, tc_match_int, tc11_ir_to_object — all PASS

## ARTIFACT 6 — Git Evidence

Repository: https://github.com/aieonyx/AXON
Branch: sel4-rewrite
Commit: 01c4b1e
Tag: v0.sel4-m3 (GPG-signed)
Base backup tag: v0.pre-sel4-rewrite

---

Copyright (c) 2026 Edison Lepiten / AIEONYX
