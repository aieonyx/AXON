# Getting Started with AXON

AXON is a systems programming language where the compiler formally verifies
your declared intent at compile time. This guide gets you from zero to a
working verified program in under 15 minutes.

---

## Prerequisites

**Required:**
- Linux (Ubuntu 22.04+ or Pop!_OS recommended)
- Rust toolchain (rustup)
- LLVM 18 + Clang 18

**Optional (for `axon suggest`):**
- Ollama with a local model

### Install Rust

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

### Install LLVM 18 + Clang 18

```bash
sudo apt install -y llvm-18 llvm-18-dev clang-18 lld-18
sudo ln -sf /usr/bin/llvm-config-18 /usr/bin/llvm-config
llvm-config --version   # should print 18.x.x
```

### Install Ollama (optional)

```bash
curl -fsSL https://ollama.com/install.sh | sh
ollama pull qwen2.5-coder:7b
```

---

## Install AXON

```bash
git clone https://github.com/aieonyx/AXON.git
cd AXON
cargo install --path axon_cli
axon version
```

---

## Your First AXON Program

Create a file called `hello.axon`:

```
module hello

fn greet() -> Int:
    return 42
```

Run it:

```bash
axon run hello.axon
```

Output:
```
AXON program 'hello' loaded.
```

---

## Your First Verification

This is where AXON becomes different from any other language.

Create `verify_demo.axon`:

```
module demo

@ensures("result >= 0")
fn safe_abs(x : Int) -> Int:
    if x < 0:
        return 0 - x
    return x

@ensures("result >= 1")
fn always_positive() -> Int:
    return 42
```

Run the verifier:

```bash
axon verify verify_demo.axon
```

Output:
```
axon verify: verify_demo.axon
  ? fn safe_abs      — unknown (cannot fully prove on all paths)
  ✓ fn always_positive — @ensures verified on all paths

axon verify: verify_demo.axon — OK (1 verified, 1 unknown)
```

`always_positive` returns `42` — the compiler proves `42 >= 1` on every
path. Verified.

`safe_abs` returns `x` on the non-negative path — `x` is an unknown
variable, so the compiler says Unknown (conservative and correct).

---

## Catching a Real Bug

Now intentionally write a broken function.

Create `broken.axon`:

```
module broken

@ensures("result >= 0")
fn bad_abs(x : Int) -> Int:
    return 0 - 1
```

Run the verifier:

```bash
axon verify broken.axon
```

Output:
```
axon verify: broken.axon

error[E411]: @ensures constraint violated
  → fn bad_abs declares: result >= 0
  → violating path: return path #1
  → hint: ensure all code paths return a value >= 0

axon verify: broken.axon — 1 violation(s) found
```

The compiler caught it. `0 - 1 = -1` which violates `result >= 0`.
The program did not compile. No tests needed. No runtime crash.
The violation was caught at compile time.

---

## AI-Assisted Formal Specs

If you have Ollama running, AXON can read your intent in plain English
and propose formal `@ensures` annotations.

Create `intent_demo.axon`:

```
module demo

@ai.intent("always returns a non-negative value")
fn abs_value(x : Int) -> Int:
    if x < 0:
        return 0 - x
    return x

@ai.intent("pure function, no side effects")
fn square(x : Int) -> Int:
    return x * x

@ai.intent("classifies severity — always returns 0, 1, or 2")
fn classify(severity : Int) -> Int:
    match severity:
        0 => return 0
        1 => return 1
        _ => return 2
```

Run the suggestion engine:

```bash
axon suggest intent_demo.axon
```

Output:
```
axon suggest: intent_demo.axon
Scanning for @ai.intent annotations...

── fn abs_value ──────────────────────────────
  @ai.intent: "always returns a non-negative value"
  Proposed: @ensures("result >= 0")

── fn square ──────────────────────────────
  @ai.intent: "pure function, no side effects"
  Proposed: @effect("pure function (no side effects)")

── fn classify ──────────────────────────────
  @ai.intent: "classifies severity — always returns 0, 1, or 2"
  Proposed: @ensures("result >= 0")
             @ensures("result <= 2")

Next steps:
  1. Add the proposed @ensures annotations to your source file
  2. Run: axon verify intent_demo.axon
  3. Fix any violations the verifier finds
```

Add the proposed annotations then run `axon verify` to confirm.

---

## Compile to Native Binary

AXON compiles directly to native machine code via LLVM 18.

```bash
axon build --native verify_demo.axon
```

Output:
```
axon build --native: verify_demo.axon → verify_demo (x86_64-unknown-linux-gnu)
  ✓ LLVM IR  → verify_demo.ll
  ✓ Object   → verify_demo.o
  ✓ Library .o ready (link manually or add fn main)
```

### Cross-compile for seL4 ARM64

```bash
axon build --native --target aarch64-sel4 verify_demo.axon
```

Output:
```
  ✓ LLVM IR  → verify_demo.ll   (target: aarch64-unknown-none-elf)
  ✓ Object   → verify_demo.o
  ✓ Cross-target .o ready. Link with: aarch64-linux-gnu-gcc
```

---

## The Full Aegis Monitor Demo

This is the integration test — the Aegis Monitor from the AXON test suite,
fully annotated and formally verified.

```bash
axon suggest axon_ai/tests/p509_aegis_monitor.axon
axon verify  axon_ai/tests/p509_aegis_monitor.axon
```

Expected output from verify:
```
  ✓ fn classify      — @ensures verified on all paths
  ✓ fn audit_priority — @ensures verified on all paths

axon verify: p509_aegis_monitor.axon — OK (2 verified, 0 unknown)
```

The `classify` function is proven to always return a value between 0 and 2.
The `audit_priority` function is proven to always return a value >= 1.
Formally. At compile time.

---

## Command Reference

| Command | What it does |
|---|---|
| `axon version` | Print version |
| `axon check <file>` | Parse and syntax-check an AXON source file |
| `axon run <file>` | Transpile to Rust and execute |
| `axon build <file>` | Transpile to Rust, generate Cargo project |
| `axon build --native <file>` | Compile via LLVM to native binary |
| `axon build --native --target arm64 <file>` | Cross-compile for ARM64 Linux |
| `axon build --native --target aarch64-sel4 <file>` | Cross-compile for seL4 bare-metal |
| `axon verify <file>` | Formally verify `@ensures` / `@requires` annotations |
| `axon suggest <file>` | AI reads `@ai.intent`, proposes `@ensures` annotations |

---

## Annotation Reference

```
@ai.intent("plain English description")
```
Natural language intent. Used by `axon suggest` to propose formal specs.
Advisory — does not gate compilation on its own.

```
@ensures("result >= 0")
```
Postcondition. The compiler verifies this holds on every return path.
Compilation fails if violated.

Supported constraint strings:
- `"result >= 0"` — result is non-negative
- `"result > 0"` — result is positive
- `"result >= N"` — result is at least N
- `"result <= N"` — result is at most N
- `"result == N"` — result equals exactly N
- `"result != null"` — result is never null
- `"no_allocation"` — function does not heap-allocate
- `"no_io"` — function performs no I/O

```
@effect("pure")
```
Side effect declaration. Supported effects:
- `"pure"` — no observable side effects
- `"readonly"` — reads but does not write system state
- `"writes_audit_log"` — writes to audit log
- `"no_allocate"` — no heap allocation permitted

---

## What `Unknown` Means

When `axon verify` reports `?` (Unknown), it means the verifier could
not prove OR disprove the constraint on that path. This is conservative
and correct — not a failure.

Unknown happens when:
- A function returns a variable (`return x`) rather than a constant
- A constraint involves paths the interval verifier cannot fully analyze

Unknown is not a violation. Your program still compiles.
Only `error[E411]` is a violation.

---

## Troubleshooting

**`axon: command not found`**
```bash
cargo install --path axon_cli --force
source $HOME/.cargo/env
```

**`llvm-config not found`**
```bash
sudo ln -sf /usr/bin/llvm-config-18 /usr/bin/llvm-config
```

**`axon suggest` returns "AI unavailable"**
Ollama is not running or the model is not pulled.
```bash
ollama serve &
ollama pull qwen2.5-coder:7b
```
`axon suggest` falls back to rule-based analysis automatically —
compilation and verification still work without Ollama.

**`axon verify` returns Unknown for everything**
This is expected for functions that return variables rather than
constants. Unknown means the verifier is being conservative.
It is not an error.

---

## Next Steps

- Read the [Language Specification](language-spec.md)
- Read the [Grammar Reference](grammar.md)
- Read the [Memory Model](memory-model.md)
- Read the [Compiler Pipeline](compiler-pipeline.md)
- Explore the [AIEONYX ecosystem](https://github.com/aieonyx)

---

*Copyright © 2026 Edison Lepiten — AIEONYX*
*AXON is open source under the MIT License*
