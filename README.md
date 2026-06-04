<p align="center">
  <img src="assets/axon-banner.png" alt="AxonBanner">

# AXON — Sovereign Systems Programming Language

> *"We are not users. We are not accounts. We are not products. We are people."*

AXON is a sovereign systems programming language built for the
[AIEONYX](https://github.com/aieonyx) platform. It combines
Rust-like memory safety, zero GC, built-in formal contracts,
and sovereign capability profiles that are enforced at compile time.

**Status: Phase 8 complete. Real programs compile, run, and execute on GPU.**

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
```
```
fn main() -> i32 { let x: i32 = 20; let y: i32 = 22; return x + y; }
→ Compiles and runs: exit code 42
→ Pipeline: AXON → LLVM 18 → native binary
```
---

## Benchmark Results

### CPU — x86_64 (Pop OS, AMD Ryzen 7)

### GPU — NVIDIA T4 (Google Colab)

<p align="center">
  <img src="assets/junebenchmark.png" alt="AxonBenchmark">
    ---


RESULT**:

- Vector addition: 1,000,000 elements × 20 runs
- Throughput:      16.64 billion ops/sec
- Correctness:     True (verified)
- Pipeline:        AXON → LLVM 18 → PTX → NVIDIA T4 (sm_75)


    
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
axon build --profile seL4-strict program.axon      # enforces strict caps
axon build --profile sovereign-offline program.axon # default sovereign
axon build --profile dev-mode program.axon          # BASTION will reject
```

---

## Formal Contracts

```axon
@requires(x > 0)
@ensures(result > 0)
fn positive(x: i32) -> i32 {
    return x;
}
```

Contracts are checked at compile time via the HIR lowerer and
ContractExpr system. Unverifiable contracts emit compiler errors —
never silently accepted.

---

## What Makes AXON Different

| Feature | Rust | C++ | Go | AXON |
|---------|------|-----|----|------|
| Memory safety | ✅ | ❌ | ⚠️ | ✅ |
| No GC | ✅ | ✅ | ❌ | ✅ |
| @requires/@ensures | ❌ | ❌ | ❌ | ✅ |
| Capability profiles | ❌ | ❌ | ❌ | ✅ |
| GPU compilation | ⚠️ | ⚠️ | ❌ | ✅ |
| Sovereign enforcement | ❌ | ❌ | ❌ | ✅ |
| One-command GPU build | ❌ | ❌ | ❌ | ✅ |

---

## Compiler Architecture
**137 tests. 0 failures.**

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
| 8 | **Working compiler — real programs compile and run** | ✅ |
| Profile Stage | BASTION integration, EdisonDB in AXON | 🔄 |

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
