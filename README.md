<p align="center">
  <img src="assets/axon_bannerF.png" alt="AXON Banner">
</p>

# AXON — Sovereign Systems Programming Language

> *"We are not users. We are not accounts. We are not products. We are people."*

![CI](https://github.com/aieonyx/AXON/actions/workflows/ci.yml/badge.svg)

**AIEONYX AXON is a sovereign systems programming language.** It combines compiler-enforced deployment profiles, formal contracts, AI-assisted verification, and CPU/GPU execution for seL4-oriented infrastructure.

Built for the [AIEONYX](https://github.com/aieonyx) platform. Rust-like memory safety, zero GC, built-in formal contracts, and sovereign capability profiles enforced at compile time.

**Status: Tier 1 + Tier 2 + Tier 3 + P67–P72 complete.**
**1,686+ tests passing. 0 failures. Clippy clean. Live aarch64-seL4 boot confirmed.**

---

## What is Novel

The genuinely new idea is the placement: a local AI verifier as a mandatory compiler phase that can reject programs. It runs fully offline, with no cloud.

Editor-side assistants like Copilot suggest but do not gate. Dafny, Verus, and SPARK have machine-checked contracts but lack a natural-language intent layer. AXON combines both: `@ensures` is discharged by a sound checker (Kani-verified core), with `@ai.intent` as a natural-language contract layer. It is the first systems language where local LLM intent verification is a compilation phase targeting seL4.

This defensible combination is not available in any other language today:

- Memory safety + Python-readable syntax
- `@ai.intent` / `@ensures` / `@requires` as compiler gates (not editor hints)
- seL4-native target — designed for it, not retrofitted
- Zero cloud dependency; sovereignty is structural, not a setting
- CPU + GPU (Vulkan/AMD RADV) + aarch64-seL4 bare metal from one toolchain
- Transformer attention expressible in pure `.ax` source (P63.1)
- Sovereign WASM JIT: x86_64 native code generation, no LLVM (P61.1)
- AArch64 freestanding native codegen — aiXos Phoenix can be written in AXONYX (P71)
- Sovereign .axpkg signed app distribution — Ed25519 + BLAKE3, deny-by-default capabilities (P72)
- IAM data pipeline — BPE tokenizer, corpus ingestion, .axd shards, training loop (P67–P68)

---

## Quick Start

```axon
fn main() -> i32 {
    let x: i32 = 20;
    let y: i32 = 22;
    let z = x + y;
    return z;
}
```

```bash
# Compile for CPU
axon build --profile sovereign-offline -o add add.ax
./add; echo $?   # 42

# Compile for Vulkan GPU (AMD RADV)
axon build --profile sovereign-offline --target vulkan -o kernel kernel.ax

# Compile for aarch64-seL4 bare metal
axon build --profile seL4-strict --target aarch64-sel4 -o node node.ax
```

---

## Benchmark Results

### Compiler Throughput (Phase 36 — Official Record)

Test: full compiler pipeline — 5,000 runs, AMD Ryzen 7, LLVM 18, Pop!_OS

| Workload | Latency | Throughput |
|---|---|---|
| IR emission (simple fn) | 25µs/compile | ~40,000 compiles/s |
| IR emission (arithmetic) | 38µs/compile | ~26,315 compiles/s |
| IR emission (multi-function) | 52µs/compile | ~19,230 compiles/s |
| Full pipeline (IR + llc + link) | ~72ms | native binary out |

### GPU — NVIDIA T4 (Google Colab)

- Vector addition: 1,000,000 elements × 20 runs
- Throughput: **16.64 billion ops/sec**
- Pipeline: AXON → LLVM 18 → PTX → NVIDIA T4 (sm_75)

> Numbers published as-is. Credibility comes from honesty, not flattery.

---

## Capability Profiles

Every AXON program compiles under a sovereign capability profile.
Violations abort compilation — not a runtime check, not a policy file.

| Profile | Use Case | BASTION Safe |
|---|---|:---:|
| `seL4-strict` | Maximum isolation. Production. | ✅ |
| `sovereign-offline` | No network. Local node. | ✅ |
| `mesh-node` | Controlled network. Mesh participant. | ✅ |
| `dev-mode` | Development only. | ❌ |

---

## Formal Contracts

```axon
@requires(x > 0)
@ensures(result > 0)
fn positive(x: i32) -> i32 {
    return x;
}
```

Contracts are checked at compile time via the HIR lowerer and ContractExpr system.
Unverifiable contracts emit compiler errors — never silently accepted.

---

## Current Capabilities

| Domain | Component | Tests | Status |
|---|---|---|---|
| **Compiler** | Lexer → Parser → HIR → LLVM 18 → native binary | 342 | ✅ |
| **Compiler** | Hindley-Milner type inference | — | ✅ |
| **Compiler** | Self-hosting bootstrap (P45–P55) | 109 | ✅ v0.55-bootstrap |
| **Compiler** | @constant_time codegen (P55.7) | 34 | ✅ |
| **Compiler** | aarch64-seL4 asm! intrinsics | — | ✅ |
| **Crypto** | axon_crypto — Ed25519, X25519, ChaCha20, SHA-256 | 93 | ✅ P57.1 |
| **GPU** | axon_gpu — Vulkan/AMD RADV backend | 30 | ✅ P58.1 |
| **AI Runtime** | axon_ai_runtime — matmul, softmax, relu | 22 | ✅ |
| **AI Runtime** | Transformer attention in pure .ax (P63.1) | 20 | ✅ |
| **WASM** | axon_wasm — binary parser, validator, interpreter | 17 | ✅ |
| **WASM** | Sovereign WASM JIT — x86_64 native codegen (P61.1) | 20 | ✅ |
| **LSP** | axon_lsp — LSP 3.17 language server (P64) | 20 | ✅ |
| **Registry** | axon_registry — sovereign package registry (P65) | 20 | ✅ |
| **Protocol** | axon_awp — AWP protocol core (P66) | 20 | ✅ |
| **seL4** | axon_sel4 — aarch64-seL4 sovereign echo | — | ✅ merged |
| **OS** | GENESIS root task, CapabilityBroker CB-01–CB-10 | — | ✅ |
| **OS** | Live aarch64-seL4 boot on QEMU, axon_main()=42 | — | ✅ |
| **Drivers** | USB HID, CDC-ECM, HDA, VESA/GOP, Mass Storage | — | ✅ |
| **Std** | axon_std — sync, mem, net, font, media | 116 | ✅ |
| **Data** | axon_data — corpus ingestion, BPE tokenizer, .axd shards (P67) | 100 | ✅ |
| **Training** | axon_train — IAM training loop, checkpoint, .iam export (P68) | 20 | ✅ |
| **Interpreter** | axon_interp — sovereign .ax interpreter, REPL, no_std (P71.5) | 20 | ✅ |
| **Packages** | axon_pkg — .axpkg signed package format, Ed25519+BLAKE3 (P72) | 20 | ✅ |
| **AArch64** | axon_aarch64 — freestanding native codegen, linker script (P71) | 20 | ✅ |
| **Workspace** | **Total** | **1,686+** | **0 failures** |

---

## What Makes AXON Different

| Feature | Rust | C++ | Go | AXON |
|---|:---:|:---:|:---:|:---:|
| Memory safety | ✅ | ❌ | ⚠️ | ✅ |
| No GC | ✅ | ✅ | ❌ | ✅ |
| `@requires` / `@ensures` | ❌ | ❌ | ❌ | ✅ |
| `@ai.intent` compiler gate | ❌ | ❌ | ❌ | ✅ |
| Capability profiles | ❌ | ❌ | ❌ | ✅ |
| GPU compilation (Vulkan) | ⚠️ | ⚠️ | ❌ | ✅ |
| seL4 bare-metal target | ❌ | ⚠️ | ❌ | ✅ |
| Sovereign WASM JIT | ❌ | ❌ | ❌ | ✅ |
| Built-in AWP protocol | ❌ | ❌ | ❌ | ✅ |
| Language server (LSP 3.17) | — | — | — | ✅ |
| Sovereign package registry | — | — | — | ✅ |
| Zero cloud dependency | ❌ | ✅ | ❌ | ✅ |
| Live seL4 boot confirmed | ❌ | ❌ | ❌ | ✅ |
| AArch64 freestanding codegen | ❌ | ⚠️ | ❌ | ✅ |
| Signed app packages (.axpkg) | ❌ | ❌ | ❌ | ✅ |
| Sovereign interpreter (no_std) | ❌ | ❌ | ❌ | ✅ |

---

## Phase History

| Phase | What | Status |
|---|---|---|
| 1–7 | Language design, lexer, parser, transpiler, LLVM backend, AI inference | ✅ |
| 8–22 | Full compiler pipeline — real programs compile and run | ✅ |
| 23–30 | OS track — seL4 syscalls, asm!, IRQ, no_std runtime | ✅ |
| 31–34 | ONYX AI compute — axon_math, axon_tensor, axon_learn, axon_compute | ✅ |
| 35–44 | OS hardening — heap, IRQ, drivers, AXFS, GENESIS, live seL4 boot | ✅ |
| 45–55 | **Self-hosting bootstrap** — AXON compiles AXON, 109 tests, GPG-signed | ✅ v0.55-bootstrap |
| 56–63 | **HANIEL unlock** — axon_net, axon_crypto, axon_gpu, axon_media, axon_wasm, axon_font, axon_ai_runtime, axon_layout | ✅ 1,446 tests |
| P57.1 | Ed25519 full curve math (93 tests, Kani-verified) | ✅ |
| P55.7 | @constant_time codegen (34 tests) | ✅ |
| P58.1 | Vulkan/AMD RADV GPU backend (30 tests) | ✅ |
| axon_sel4 | seL4 rewrite — aarch64 sovereign_echo.ax, seL4 ABI PASSED | ✅ merged |
| **P63.1** | Transformer attention in pure .ax source | ✅ 20 tests |
| **P61.1** | ECHO sovereign WASM JIT — x86_64 native codegen, no LLVM | ✅ 20 tests |
| **P64** | axon_lsp — AXONYX Language Server (LSP 3.17) | ✅ 20 tests |
| **P65** | axon_registry — sovereign package registry (SHA-256, Ed25519) | ✅ 20 tests |
| **P66** | axon_awp — AWP protocol core (11 categories, 249 regions, C-ABI FFI) | ✅ 20 tests |
| **P67** | axon_data — IAM corpus pipeline: ingestion, BPE tokenizer, .axd shards | ✅ 100 tests |
| **P68** | axon_train — IAM training loop, checkpoint, eval, .iam export | ✅ 20 tests |
| **P69** | iamrt spec — approved, implementation pending hardware upgrade | 📋 spec locked |
| **P71.5** | axon_interp — sovereign .ax interpreter upstreamed, REPL mode, no_std | ✅ 20 tests |
| **P72** | axon_pkg — .axpkg signed package format (Ed25519 + BLAKE3, capabilities) | ✅ 20 tests |
| **P71** | axon_aarch64 — AArch64 freestanding codegen, linker script, conformance oracle | ✅ 20 tests |

---

## CS Contributions Registry

55 formally named terms across AXON, EdisonDB, and Onyxia. Selected highlights:

- **Sovereign AWP Protocol** — two-tier naming grammar, fixed category registry, ISO 3166-1 regional routing
- **Capability-Flow Compiler** — static analysis rejects capability violations before code generation
- **ARPi Provenance Header** — 78-byte fixed wire format for data origin verification without transport trust
- **Sovereign Hash Projection Embedding** — deterministic offline embeddings, zero network, zero model files
- **BASTION Binary Verification Gate** — 7-step gate, dev-mode unconditionally rejected
- **@constant_time Codegen** — compiler-enforced constant-time code paths for crypto operations

Full registry: [EXHIBIT.md](https://github.com/aieonyx/onyxia/blob/main/EXHIBIT.md)
arXiv submission: cs.AR — slot 7680982

---

## Building

```bash
git clone https://github.com/aieonyx/AXON
cd AXON
cargo test --workspace -- --test-threads=1
```

Individual crate tests:
```bash
cargo test -p axon_crypto -- --test-threads=1
cargo test -p axon_awp --test p66_awp_tests -- --test-threads=1
cargo test -p axon_lsp --test p64_lsp_tests -- --test-threads=1
cargo test -p axon_interp --test p71_5_interp_tests -- --test-threads=1
cargo test -p axon_pkg --test p72_pkg_tests -- --test-threads=1
cargo test -p axon_aarch64 --test p71_aarch64_tests -- --test-threads=1
```

---

## Part of the AIEONYX sovereign stack

| Component | Role | Status |
|---|---|---|
| **AXON** (this repo) | Sovereign compiler, protocol, registry, LSP | ✅ 1,606+ tests |
| **[EdisonDB](https://github.com/aieonyx/edisondb)** | Sovereign database — Phase 3 complete | ✅ v0.6.0-stable |
| **[Onyxia](https://github.com/aieonyx/onyxia)** | Sovereign browser | ✅ v1.0.0 |
| **[BASTION](https://github.com/aieonyx/bastion)** | Sovereign node OS bootstrap | ✅ v0.2.0 |
| **aiXos Phoenix** | Sovereign desktop OS (bare-metal AArch64, no Linux) | 🔵 PL-60+ |

---

## License

Apache 2.0 — permanently and irrevocably.
Community Promise II: the core will never become proprietary.

## Author

Edison Lepiten — Solo founder, AIEONYX
Prague, Czech Republic.
For ordinary people. Not corporations.

---

*AIEONYX: github.com/aieonyx*
*NLNet NGI Zero grant application submitted May 2026*
*CS Contributions Registry: 55 formally named terms — arXiv submission in preparation*
*P67–P72 complete: IAM pipeline, .ax interpreter, .axpkg packages, AArch64 freestanding codegen*
