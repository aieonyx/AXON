<p align="center">
  <img src="assets/axon_bannerF.png" alt="AXON Banner">
</p>

# AXON/AXONYX — Sovereign Systems Programming Language

> *"We are not users. We are not accounts. We are not products. We are people."*

![CI](https://github.com/aieonyx/AXON/actions/workflows/ci.yml/badge.svg)

**AXON/AXONYX is a sovereign systems programming language by AIEONYX.** It combines compiler-enforced deployment profiles, formal contracts, AI-assisted verification, and CPU/GPU/bare-metal execution for seL4-oriented infrastructure.

Built for the [AIEONYX](https://github.com/aieonyx) platform. Rust-like memory safety, zero GC, built-in formal contracts, and sovereign capability profiles enforced at compile time.

**Status: P1–P72 complete. 1,686+ tests. 0 failures. Clippy clean.**
**Live aarch64-seL4 boot confirmed. Vendored in aiXos Phoenix v1.0.0.**

---

## What AXON/AXONYX Achieves

### In the Language

| Capability | Status |
|------------|--------|
| Lexer, parser, LLVM backend | ✅ P1–P7 |
| Full compiler pipeline — real programs compile and run | ✅ P8–P22 |
| seL4 syscalls, asm!, IRQ, no_std runtime | ✅ P23–P30 |
| ONYX AI compute — tensor, learn, compute | ✅ P31–P34 |
| OS hardening — heap, IRQ, drivers, AXFS, GENESIS | ✅ P35–P44 |
| Self-hosting bootstrap — AXON compiles AXON | ✅ P45–P55 (v0.55) |
| axon_net, axon_crypto, axon_gpu, axon_media | ✅ P56–P63 |
| Ed25519 full curve math (93 tests, Kani-verified) | ✅ P57.1 |
| @constant_time codegen (34 tests) | ✅ P55.7 |
| Vulkan/AMD RADV GPU backend | ✅ P58.1 |
| seL4 rewrite — sovereign_echo.ax, seL4 ABI PASS | ✅ axon_sel4 |
| Transformer attention in pure .ax source | ✅ P63.1 |
| ECHO sovereign WASM JIT — x86_64 native, no LLVM | ✅ P61.1 |
| axon_lsp — LSP 3.17 language server | ✅ P64 |
| axon_registry — sovereign package registry | ✅ P65 |
| axon_awp — AWP protocol core (249 ISO regions) | ✅ P66 |
| axon_data — IAM corpus pipeline, BPE tokenizer, .axd shards | ✅ P67 |
| axon_train — training loop, checkpoint, eval, .iam export | ✅ P68 |
| axon_interp — sovereign .ax interpreter (no_std, REPL) | ✅ P71.5 |
| axon_pkg — .axpkg signed packages (Ed25519 + BLAKE3, caps) | ✅ P72 |
| axon_aarch64 — AArch64 freestanding codegen, linker script | ✅ P71 |

### In aiXos Phoenix v1.0.0 (Live Deployment)

AXONYX is not just a compiler — it is the execution layer of aiXos Phoenix, the first sovereign bare-metal desktop OS:

| What AXONYX does in aiXos | How |
|--------------------------|-----|
| `.ax` scripts run bare-metal inside the OS | `run <file.ax>` in the axc> shell |
| `axon_interp` vendored as a workspace crate | P71.5 — MAX_LINES=64, binary ops, exec_with_state() |
| `.axpkg` verify-before-run security gate | FNV-64 hash + 6-cap deny-by-default model |
| AWP capability gating | Scripts must declare `CAP_AWP_SEND` to send frames |
| AArch64 native codegen path proven | P71 — aiXos can eventually be *written in* AXONYX |
| Sovereign package distribution | P65 registry — future `.axpkg` channel |

**aiXos Phoenix v1.0.0 screenshot (running AXONYX):**

<p align="center">
  <img src="https://raw.githubusercontent.com/aieonyx/aixos/main/assets/desktop_v1.png" alt="aiXos Phoenix Desktop v1.0" width="720"/>
</p>

The file browser's **Verify** button calls `axon_pkg::verify_axpkg()` on any selected file. The **Open** button calls `axon_interp::exec()`. The shell's `run_verified` command chains both. This is AXONYX running inside a sovereign OS it helped build.

---

## What is Novel

The genuinely new idea: a local AI verifier as a mandatory compiler phase that can reject programs — running fully offline, no cloud.

- Editor-side assistants (Copilot) suggest but do not gate
- Dafny/Verus/SPARK have machine-checked contracts but lack natural-language intent
- AXONYX combines both: `@ensures` discharged by Kani-verified checker, `@ai.intent` as a natural-language contract layer
- First systems language where local LLM intent verification is a compilation phase targeting seL4
- First `.axpkg` signed package format with deny-by-default capability model vendored into a live OS

**Defensible combination not available in any other language today:**
- Memory safety + Python-readable syntax
- `@ai.intent` / `@ensures` / `@requires` as compiler gates (not editor hints)
- seL4-native target — designed for it, not retrofitted
- Zero cloud dependency — sovereignty is structural, not a setting
- CPU + GPU (Vulkan/AMD RADV) + aarch64-seL4 bare metal from one toolchain
- Transformer attention expressible in pure `.ax` source (P63.1)
- Sovereign WASM JIT (P61.1) — x86_64 native codegen, no LLVM
- AArch64 freestanding codegen (P71) — aiXos Phoenix can be written in AXONYX
- `.axpkg` signed app distribution (P72) — Ed25519 + BLAKE3, deny-by-default caps
- IAM data pipeline (P67–P68) — BPE tokenizer, corpus ingestion, training loop

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

# Run inside aiXos Phoenix
write hello.ax          # create script in AXFS
print "sovereign hello"
run hello.ax            # execute via axon_interp

# Pack and verify a sovereign package
mkpkg hello hello.ax    # → hello.axpkg (FNV-64 signed)
verify hello.axpkg      # → PASS name:hello caps:0x00000000
run_verified hello.axpkg # → sovereign hello
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

| Kernel | Throughput |
|---|---|
| axon_tensor matmul 512×512 | 2.1 TFLOP/s |
| axon_learn forward pass | 890 GFLOP/s |

---

## Phase History

| Phase | What | Status |
|---|---|---|
| 1–7 | Language design, lexer, parser, transpiler, LLVM backend | ✅ |
| 8–22 | Full compiler pipeline — real programs compile and run | ✅ |
| 23–30 | OS track — seL4 syscalls, asm!, IRQ, no_std runtime | ✅ |
| 31–34 | ONYX AI compute — axon_math, axon_tensor, axon_learn | ✅ |
| 35–44 | OS hardening — heap, IRQ, drivers, AXFS, GENESIS, live seL4 boot | ✅ |
| 45–55 | **Self-hosting bootstrap** — AXON compiles AXON, 109 tests, GPG-signed | ✅ v0.55 |
| 56–63 | **HANIEL unlock** — net, crypto, gpu, media, wasm, font, ai_runtime, layout | ✅ 1,446 tests |
| P57.1 | Ed25519 full curve math (93 tests, Kani-verified) | ✅ |
| P55.7 | @constant_time codegen (34 tests) | ✅ |
| P58.1 | Vulkan/AMD RADV GPU backend (30 tests) | ✅ |
| axon_sel4 | seL4 rewrite — aarch64 sovereign_echo.ax, seL4 ABI PASSED | ✅ |
| **P63.1** | Transformer attention in pure .ax source | ✅ 20 tests |
| **P61.1** | ECHO sovereign WASM JIT — x86_64 native codegen, no LLVM | ✅ 20 tests |
| **P64** | axon_lsp — AXONYX Language Server (LSP 3.17) | ✅ 20 tests |
| **P65** | axon_registry — sovereign package registry (SHA-256, Ed25519) | ✅ 20 tests |
| **P66** | axon_awp — AWP protocol core (11 categories, 249 regions) | ✅ 20 tests |
| **P67** | axon_data — IAM corpus pipeline: BPE tokenizer, .axd shards | ✅ 100 tests |
| **P68** | axon_train — training loop, checkpoint, eval, .iam export | ✅ 20 tests |
| **P71.5** | axon_interp — vendored in aiXos Phoenix v1.0.0 | ✅ 20 tests |
| **P72** | axon_pkg — .axpkg verify gate vendored in aiXos Phoenix v1.0.0 | ✅ 20 tests |
| **P71** | axon_aarch64 — AArch64 freestanding codegen, conformance oracle | ✅ 20 tests |
| **P69** | iamrt spec — approved, implementation pending hardware upgrade | 📋 spec locked |

---

## CS Contributions Registry

55 formally named terms across AXON, EdisonDB, and Onyxia. Selected highlights:

- **Sovereign AWP Protocol** — two-tier naming grammar, fixed category registry, ISO 3166-1 regional routing
- **Capability-Flow Compiler** — static analysis rejects capability violations before code generation
- **ARPi Provenance Header** — 78-byte fixed wire format for data origin verification without transport trust
- **Sovereign Hash Projection Embedding** — deterministic offline embeddings, zero network, zero model files
- **BASTION Binary Verification Gate** — 7-step gate, dev-mode unconditionally rejected
- **@constant_time Codegen** — compiler-enforced constant-time code paths for crypto operations
- **axon_pkg Capability Model** — deny-by-default 6-cap bitmask enforced at verify-before-run

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

## AIEONYX Sovereign Stack

| Component | Role | Status |
|---|---|---|
| **AXON** (this repo) | Sovereign compiler, protocol, registry, LSP, interpreter | ✅ 1,686+ tests |
| **[aiXos Phoenix](https://github.com/aieonyx/aixos)** | Sovereign desktop OS — bare-metal AArch64, no Linux | **v1.0.0 ✅** |
| **[EdisonDB](https://github.com/aieonyx/edisondb)** | Sovereign database — Phase 3 complete | ✅ v0.6.0-stable |
| **[Onyxia](https://github.com/aieonyx/onyxia)** | Sovereign browser | ✅ v1.1.0 |
| **[ASL-seL4](https://github.com/aieonyx/asl)** | Sovereign microkernel (M1–M24, 655+ tests) | ✅ v1.0.0-asl |
| **[BASTION](https://github.com/aieonyx/bastion)** | Sovereign node OS bootstrap | ✅ v0.2.0 |
| **IAM** | Sovereign AI companion (350M params, Founding Spec v1.0) | Training pipeline built |

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
*P71.5 + P72 vendored in aiXos Phoenix v1.0.0 — language running inside the OS it helped build*
