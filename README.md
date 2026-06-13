<p align="center">
  <img src="assets/axon-banner.png" alt="AXON Banner">
</p>

# AXON — Sovereign Systems Programming Language

> *"We are not users. We are not accounts. We are not products. We are people."*

**AXON is the world's first sovereign systems programming language — unifying compiler-enforced deployment profiles, formal contracts, AI-assisted verification, and CPU/GPU execution for seL4-oriented infrastructure.**

Built for the [AIEONYX](https://github.com/aieonyx) platform. Rust-like memory safety, zero GC, built-in formal contracts, and sovereign capability profiles enforced at compile time.

**Status: Phase 36 complete. Full compiler pipeline, OS track, ONYX AI compute, aarch64-seL4 intrinsics. 430+ tests passing.**

---

## What is Novel

The genuinely new idea is the *placement*: a local AI verifier as a mandatory compiler phase that can reject programs — running fully offline, zero cloud.

Editor-side assistants (Copilot etc.) suggest; they do not gate. Dafny/Verus/SPARK have machine-checked contracts but no natural-language intent layer. AXON combines both: `@ensures` discharged by a sound checker (Kani-verified core), with `@ai.intent` as a natural-language contract layer — the first systems language where local LLM intent-verification is a compilation phase targeting seL4.

The defensible combination no other language ships today:
- Memory safety + Python-readable syntax
- `@ai.intent` / `@ensures` / `@requires` as compiler gates (not editor hints)
- seL4-native target — designed for it, not retrofitted
- Zero cloud dependency — sovereignty is structural, not a setting
- CPU + GPU (PTX) + aarch64-seL4 bare metal from one toolchain

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
axon build --profile sovereign-offline -o add add.axon
./add; echo $?   # 42

# Compile for NVIDIA GPU (T4, A100, RTX)
axon build --profile sovereign-offline --target nvptx64 -o kernel kernel.axon
# → produces kernel.ptx, validated by ptxas

# Compile for aarch64-seL4 bare metal
axon build --profile seL4-strict --target aarch64-sel4 -o node node.axon
```

---

## Benchmark Results

### Phase 4 — Official Record

<p align="center">
  <img src="assets/phase4_benchmark.png" alt="Phase 4 Benchmark">
</p>

---

### Phase 36 — Official Record
---

### Compiler Throughput — Phase 36 (Detailed)
*Parse → HIR → LLVM IR → llc-18 → clang → native binary*
*Machine: Pop OS, AMD Ryzen 7, ~32GB RAM, LLVM 18*

| Workload | Per Compile | Rate |
|----------|------------|------|
| `fn main() -> i32 { return 42; }` | 25µs | 40,000/sec |
| Arithmetic (3 ops) | 38µs | 26,315/sec |
| Multi-function (2 fns) | 52µs | 19,230/sec |
| **Average (5,000 runs)** | **33µs** | **30,303/sec** |
| Full pipeline (IR+llc-18+clang) | **~72ms** | native binary out |

All compiled binaries verified correct — exit code 42 across all workloads. ✅

---

### GPU — NVIDIA T4 (Google Colab)

<p align="center">
  <img src="assets/junebenchmark.png" alt="GPU Benchmark">
</p>

- Vector addition: 1,000,000 elements × 20 runs
- Throughput: **16.64 billion ops/sec**
- Pipeline: AXON → LLVM 18 → PTX → NVIDIA T4 (sm_75)

> Numbers published as-is. Credibility comes from honesty, not flattery.

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

## Capability Profiles

Every AXON program compiles under a sovereign capability profile.
Violations abort compilation — not a runtime check, not a policy file.

| Profile | Use Case | BASTION Safe |
|---------|----------|---|
| `seL4-strict` | Maximum isolation. Production. | ✅ |
| `sovereign-offline` | No network. Local node. | ✅ |
| `mesh-node` | Controlled network. Mesh participant. | ✅ |
| `dev-mode` | Development only. | ❌ |

```bash
axon build --profile seL4-strict program.axon       # enforces strict caps
axon build --profile sovereign-offline program.axon  # default sovereign
axon build --profile dev-mode program.axon           # BASTION will reject
```

---

## What Makes AXON Different

| Feature | Rust | C++ | Go | AXON |
|---------|------|-----|----|------|
| Memory safety | ✅ | ❌ | ⚠️ | ✅ |
| No GC | ✅ | ✅ | ❌ | ✅ |
| @requires/@ensures | ❌ | ❌ | ❌ | ✅ |
| @ai.intent compiler gate | ❌ | ❌ | ❌ | ✅ |
| Capability profiles | ❌ | ❌ | ❌ | ✅ |
| GPU compilation | ⚠️ | ⚠️ | ❌ | ✅ |
| seL4 bare-metal target | ❌ | ⚠️ | ❌ | ✅ |
| Built-in AI compute (ONYX) | ❌ | ❌ | ❌ | ✅ |
| Sovereign enforcement | ❌ | ❌ | ❌ | ✅ |
| Zero cloud dependency | ❌ | ✅ | ❌ | ✅ |

---

## Compiler Architecture

**430+ tests. 0 failures. Clippy clean.**

Full pipeline: Lexer → Parser → HIR → HM Type Inference → LLVM 18 → Native binary / PTX / aarch64-seL4 ELF

Kani-verified core (`axon_verify_core`): 17 harnesses, 0 failures — constitutional verification kernel.

---

## Possible Contributions to the World

- **A new compiler category** — verification-gated compilation with natural-language contracts. Citable, nameable, first-mover.
- **A sovereign high-assurance toolchain for seL4** — no language was *designed* for seL4 until AXON. Real value for embedded, defense, medical, and election-integrity work.
- **A teaching bridge into formal methods** — `@ensures` in readable syntax lowers the barrier dramatically versus Dafny or SPARK.
- **Research artifacts** — arXiv paper, CS term registry (46 formally named terms), Kani-verified core, reproducible build manifests.
- **ONYX sovereign AI compute** — inference on BASTION nodes without cloud dependency. Local tensor engine, autodiff, GPU dispatch.

---

## Status

| Phase | What | Status |
|-------|------|--------|
| 1 | Language design & S4+i spec | ✅ |
| 2 | Lexer + Parser | ✅ |
| 3 | Rust transpiler | ✅ |
| 4 | LLVM native backend | ✅ |
| 5 | AI inference engine | ✅ |
| 6 | Stage 3 compiler features (263 tests) | ✅ |
| 7 | Compiler architecture spec (CCP, ASP, ownership) | ✅ |
| 8–22 | Full compiler pipeline — real programs compile and run | ✅ |
| 23–30 | OS Development Track — seL4 syscalls, asm!, IRQ, no_std runtime | ✅ |
| 31 | axon_math — ONYX core math stdlib (FFT, linalg, stats) | ✅ |
| 32 | axon_tensor — Tensor engine + SIMD | ✅ |
| 33 | axon_learn — Autodiff, neural layers, SGD/Adam | ✅ |
| 34 | axon_compute — GPU dispatch, AWP mesh, ONYX checkpoint | ✅ |
| 35 | Result<T,E> error payload — E type stored, ? operator fixed | ✅ |
| 36 | aarch64-seL4 asm! intrinsics — sel4_reply/wait/poll/nb_send | ✅ |
| 37 | axon_alloc — sovereign heap allocator | 🔜 |
| 38 | IRQ dispatch layer | 🔜 |
| 39 | Driver PAL — UART, GPIO, timer | 🔜 |
| 40 | AXFS — sovereign file system layer | 🔜 |
| 41 | GENESIS root task | 🔜 |
| 42 | Live aarch64-seL4 boot | 🔜 |
| 43–44 | Phoenix generic drivers | 🔜 |

---

## Building

```bash
git clone https://github.com/aieonyx/axon
cd axon
cargo install --path axon_cli
axon version
```

---

## License

Apache 2.0 — permanently and irrevocably.
Community Promise II: the core will never become proprietary.

## Author

Edison Lepiten — solo founder, AIEONYX
Built after work hours in Prague, Czech Republic.
For ordinary people. Not corporations.

---

*AIEONYX: github.com/aieonyx*
*NLNet NGI Zero grant application submitted May 2026*
*CS Contributions Registry: 46 formally named terms — arXiv submission in preparation*
