# AXON-QA-P61.1-001 — ECHO WASM JIT QA Audit
<!-- Copyright (c) 2026 Edison Lepiten / AIEONYX -->
<!-- INTERNAL ONLY -->

## Phase
P61.1 — ECHO sovereign WASM JIT: x86_64 native code generation

## Deliverables
- `crates/axon_wasm/src/jit.rs` — CodeBuffer (mmap RWX), X64Emitter, jit_compile, LEB128
- `crates/axon_wasm/src/module.rs` — WasmFunc struct added
- `crates/axon_wasm/src/lib.rs` — pub mod jit
- `crates/axon_wasm/tests/p61_1_jit_tests.rs` — 20 tests

## Opcodes supported
| Opcode | Mnemonic       |
|--------|----------------|
| 0x00   | unreachable    |
| 0x01   | nop            |
| 0x0B   | end            |
| 0x0F   | return         |
| 0x1A   | drop           |
| 0x20   | local.get      |
| 0x21   | local.set      |
| 0x22   | local.tee      |
| 0x41   | i32.const      |
| 0x42   | i64.const      |
| 0x6A   | i32.add        |
| 0x6B   | i32.sub        |
| 0x6C   | i32.mul        |
| 0x71   | i32.and        |
| 0x72   | i32.or         |
| 0x7C   | i64.add        |
| 0x7D   | i64.sub        |
| 0x7E   | i64.mul        |
| 0x83   | i64.and        |
| 0x84   | i64.or         |

## Calling convention
- Sovereign ABI: fn(locals_ptr: *mut i64) -> i64
- rdi = pointer to locals[] array (i64 flat)
- rax = return value
- Value stack: shadow slots at [rbp-16] downward (max 64 values)
- No external JIT library — raw Linux mmap syscall, pure x86_64 machine bytes

## Test matrix
T1  CodeBuffer alloc non-null
T2  emit_u8 increments written()
T3  LEB128 u32 single byte
T4  LEB128 u32 multi-byte (300)
T5  LEB128 i32 positive
T6  LEB128 i32 negative (-1 = 0x7F)
T7  LEB128 i64 large (624485)
T8  empty body returns 0
T9  i32.const 42
T10 i32.const -1
T11 i32.add (10+32=42)
T12 i32.sub (100-58=42)
T13 i32.mul (6*7=42)
T14 local.get (pre-loaded 99)
T15 local.set then local.get (77)
T16 local.tee (55, stack+local)
T17 nop passthrough
T18 drop (99 stays after 42 dropped)
T19 compound (3+4)*6=42
T20 unsupported opcode → Err

## 3P Doctrine gate
- P1 Purpose: S4+i Speed — JIT eliminates interpreter overhead for HANIEL ECHO
- P2 Pattern (internal): engine-mechanical — single-pass compiler, pistons fire once
- P3 Practice: Law of Complementarity — JIT fast path, existing interpreter fallback

## Post Doctrine 5-check
- [ ] Attribution Scrub
- [ ] Internal Language Scrub
- [ ] Spec Confidentiality
- [ ] Clean Commit
- [ ] Copyright: "Copyright (c) 2026 Edison Lepiten / AIEONYX"

## Deferred (P61.2 scope)
- i32.div_s / i32.rem_s (require idiv + divide-by-zero trap)
- f32/f64 opcodes (require SSE2 emitter)
- br / br_if / block / loop (require label patching)
- Memory opcodes (i32.load, i32.store)
- Multi-value returns
