# AXONYX Bootstrap — P55
**Copyright (c) 2026 Edison Lepiten / AIEONYX**
**Date:** 2026-06-19
**Tag:** `v0.55-bootstrap` (GPG-signed)

---

## Bootstrap Condition
Let S = axonc.ax  (the unified AXON source of the compiler itself)
Pass 1:  axonc-rust  compiles S → binary B1

Pass 2:  B1          compiles S → binary B2

Pass 3:  B2          compiles S → binary B3
ACHIEVED iff: sha256(B1) == sha256(B2) == sha256(B3)
---

## Stage 0: axonc-rust (Active at P55)

The Rust bridge implements all ten sovereign pipeline crates.
Rust exits the chain when `axonc.ax` compiles itself.

| Phase | Crate | Tests | Role |
|---|---|---|---|
| P45 | `axon_std_io` | 9/9 | I/O substrate |
| P46 | `axon_std_string` | 12/12 | Sovereign strings + SSO |
| P47 | `axon_build` | 10/10 | Build system + DAG |
| P48 | `axon_link` | 10/10 | ELF assembler + seL4 image |
| P49 | `axon_lex` | 14/14 | AXONYX lexer |
| P50 | `axon_parse` | 12/12 | Recursive descent parser |
| P51 | `axon_hir` | 10/10 | HIR lowering |
| P52 | `axon_infer` | 10/10 | Type inference |
| P53 | `axon_codegen` | 10/10 | LLVM IR emission |
| P54 | `axon_native` | 10/10 | x86_64 machine bytes |
| **P55** | **`axonc`** | **8/8** | **Compiler driver** |

---

## Running the Bootstrap Proof

```bash
# Build stage 0 (Rust bridge)
cargo build --release -p axonc

# Step 1: Compile axonc.ax with axonc-rust
./target/release/axonc axonc.ax -o axonc-stage1

# Step 2: Compile axonc.ax with stage 1
./axonc-stage1 axonc.ax -o axonc-stage2

# Step 3: Compile axonc.ax with stage 2
./axonc-stage2 axonc.ax -o axonc-stage3

# Verify — all three hashes must be identical
sha256sum axonc-stage1 axonc-stage2 axonc-stage3
```

---

## Determinism Proof (P55 Test T4)

At P55, determinism of `axonc-rust` is verified by:
compile(S) == compile(S) == compile(S)  [3 consecutive calls]
This is a necessary condition for bootstrap. It is proven by `test_3pass_simulation`.

---

## What Comes Next

1. **`.ax` Unification** — merge P49–P54 `.ax` specification files into `axonc.ax`
2. **True Bootstrap** — run the 3-pass procedure above
3. **Rust Retirement** — once sha256 matches, the Rust bridges are archived
4. **Sovereign Epoch** — AXON compiles AXON; the chain is complete

---

## Known Deferrals

| ID | Description |
|---|---|
| DEFER-P54-001 | `idiv` for integer division |
| DEFER-P54-002 | Float in SSE2 registers |

These are resolved during the `.ax` unification pass.

---

*"The compiler that describes itself."*
*— Edison Lepiten / AIEONYX, 2026*
