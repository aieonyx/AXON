# AArch64 Attestation — Evidence Exhibit

**Copyright (c) 2026 Edison Lepiten / AIEONYX — SPDX-License-Identifier: Apache-2.0**

---

## Claim

The AXONYX compiler emits AArch64 relocatable objects that are linkable as
seL4 Protection Domains and that execute correctly. Output is byte-identical
across independent machines.

Every push to `main` re-establishes this from source. Nothing here requires
trusting a report: the compiler is built from the commit under test, the
object it produces is published, and the result is visible in a public log.

## Verify it

**Without installing anything** — open the latest run of the
[AArch64 Attestation workflow](../../actions/workflows/attest.yml). The run
summary contains the full manifest. The compiled object, its SHA-256, and
the build log are attached as artefacts.

**On your own machine:**

```bash
git clone https://github.com/aieonyx/AXON && cd AXON
sudo apt-get install -y binutils-aarch64-linux-gnu gcc-aarch64-linux-gnu \
                        qemu-user clang llvm
bash scripts/attest-build.sh    # compile and verify the object
bash scripts/attest-exec.sh     # execute it, assert the invariant
```

The object hash you obtain should match the one in the published manifest.

## What is checked

| # | Assertion | Established by |
|---|---|---|
| 1 | Compiler emits an object under `seL4-strict` | compiler ABI gate |
| 2 | `seL4-strict` profile is enforced, not merely requested | compiler ABI gate |
| 3 | Output is an ELF 64-bit LSB relocatable | `file` |
| 4 | Architecture is aarch64 | `file` |
| 5 | ELF header `Machine` is `AArch64` | `readelf -h` |
| 6 | `axon_main` is a global text symbol | `nm` |
| 7 | No dynamic dependencies — freestanding | `readelf -d` |
| 8 | No `.interp` section — no loader expected | `readelf -S` |
| 9 | Compiled code returns 16723 | `qemu-aarch64` |
| 10 | Value carries the sovereign proof invariant `0x4153` | `qemu-aarch64` |
| 11 | Harness exits 0 | shell |
| 12 | Attested object unmodified by the execution proof | `sha256sum` |

Assertions 1–2 are the compiler's own verdict. Assertions 3–12 are
independent of it, by design — see LIMIT-005 below.

## Method

The fixture `attest/sovereign_attest.ax` is **frozen**. It is compiled on
every push and never edited; editing it would invalidate every previously
published hash. It computes `16000 + 723 = 16723 = 0x4153` using arithmetic
rather than a literal, so code generation must emit real instructions.

```
attest/sovereign_attest.ax
   │  axon build --target aarch64-sel4 --profile seL4-strict
   ▼
sovereign_attest.o                      920 bytes, AArch64 relocatable
   │  objcopy --globalize-symbol=axon_main
   ▼
verify (assertions 3–8) ──► SHA-256 ──► published manifest
   │
   │  a copy is linked into a minimal C harness
   ▼
qemu-aarch64 ──► axon_main() = 16723 (0x4153)
```

The execution proof links a **copy**. The attested object's hash is
re-checked afterwards to prove it was untouched.

## Observed results

| Run | Machine | Object SHA-256 | Result |
|---|---|---|---|
| local | Pop!_OS, LLVM 18 | `84603b38…c77180` | 8/8 shape, 5/5 execution |
| CI (PR) | ubuntu-latest | `84603b38…c77180` | pass |
| CI (main) | ubuntu-latest | `84603b38…c77180` | pass |

Object: 920 bytes. Fixture SHA-256: `32ea61c6…21b3d`.
Compiler: AXON 0.8.0-phase8.

## Limits

Stated so that the claim is not read as broader than it is.

**LIMIT-001** — A single fixture exercising integer arithmetic. This is not a
proof of general code-generation correctness. Compiler correctness is covered
separately by the project's test suite.

**LIMIT-002** — Execution is under user-mode QEMU, not on physical AArch64
hardware and not as a live seL4 Protection Domain. Boot-level proof is tracked
separately.

**LIMIT-003** — Byte-identical output is observed across three runs on two
machines. This is empirical evidence of deterministic code generation, not a
formal guarantee across all toolchain versions.

**LIMIT-004** — Symbol globalisation is performed by `objcopy`, external to
the compiler. The compiler emits `axon_main` as a local symbol.

**LIMIT-005** — Assertions 1–2 are self-reported by the compiler under test.
They are retained because the compiler's refusal to emit a non-conforming
object is itself meaningful, but they are not independent. Assertions 3–12
exist to provide verification that does not depend on the compiler's own
judgement.

## Context

AXONYX is self-hosting: its parser (`crates/axon_parse/axon/parse.ax`),
code emitter (`crates/axon_codegen/axon/emit.ax`), and type unification
(`crates/axon_infer/axon/unify.ax`) are implemented in AXONYX itself.

The `aarch64-sel4` target exists because AXONYX-compiled code runs as
Protection Domains under seL4 in the aiXos Phoenix operating system.
This exhibit establishes the compiler-side half of that claim.
