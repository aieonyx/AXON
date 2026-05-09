# AXON Language Specification v0.3

> Formal specification of the AXON programming language.
> Copyright © 2026 Edison Lepiten — AIEONYX

---

## Overview

AXON is a statically typed, indentation-sensitive systems programming language
with first-class AI verification support. It compiles to native machine code
via LLVM and targets seL4 bare-metal as its primary sovereign infrastructure
deployment platform.

### Design Goals

1. **Sovereign** — zero cloud dependency, all inference runs locally
2. **Verifiable** — declared intent is formally checked at compile time
3. **Fast** — native performance via LLVM, equivalent to Rust
4. **Safe** — capability-based memory model, no implicit allocation
5. **Legible** — Python-like syntax, English-first annotations

---

## Syntax

### Indentation

AXON is indentation-sensitive. Blocks are delimited by consistent indentation
(4 spaces recommended). Tabs are not permitted.

```
fn example(x : Int) -> Int:
    if x > 0:
        return x
    return 0
```

### Comments

```
# This is a single-line comment
```

### Module Declaration

Every AXON source file begins with an optional module declaration:

```
module aieonyx.aegis.monitor
```

Module paths use dot notation and correspond to the project namespace.

---

## Types

### Primitive Types

| Type | Description | Size |
|---|---|---|
| `Int` | Signed 64-bit integer | 8 bytes |
| `Int32` | Signed 32-bit integer | 4 bytes |
| `Int8` | Signed 8-bit integer | 1 byte |
| `UInt` | Unsigned 64-bit integer | 8 bytes |
| `Float` | 64-bit floating point | 8 bytes |
| `Float32` | 32-bit floating point | 4 bytes |
| `Bool` | Boolean | 1 byte |
| `Char` | Unicode scalar value | 4 bytes |
| `Str` | UTF-8 string | pointer + length |
| `Bytes` | Raw byte sequence | pointer + length |

### Composite Types

```
# Option
let x : Option<Int> = None
let y : Option<Int> = Some(42)

# Result
let r : Result<Int, Str> = Ok(42)
let e : Result<Int, Str> = Err("failed")

# List
let xs : List<Int> = [1, 2, 3]
```

### User-Defined Types

**Structs:**
```
struct Point:
    x : Int
    y : Int
```

**Enums:**
```
enum ThreatLevel:
    Clear
    Advisory
    Critical
```

---

## Functions

### Declaration

```
fn name(param : Type, ...) -> ReturnType:
    body
```

### Tasks (Async Functions)

Tasks are async functions that may perform I/O. They declare their
capabilities via the `uses` clause:

```
task monitor() uses [ipc.read, collective.emit]:
    ...
```

### Return

```
fn abs(x : Int) -> Int:
    if x < 0:
        return 0 - x
    return x
```

---

## Statements

### Variable Binding

```
let x = 42          # immutable binding
mut y = 0           # mutable binding
```

### Assignment

```
y = y + 1
```

### Conditionals

```
if condition:
    ...
elif other_condition:
    ...
else:
    ...
```

### Match

```
match value:
    0 => return 0
    1 => return 1
    _ => return 2
```

### Loops

```
# While loop
while condition:
    ...

# For loop
for item in collection:
    ...
```

### Defer

Deferred statements execute at scope exit in LIFO order:

```
let channel = ipc.open_channel()?
defer channel.close()
# channel.close() runs when the enclosing scope exits
```

---

## Decorators

Decorators appear above function or module declarations and begin with `@`.

### `@ai.intent`

Natural language description of the function's intended behavior.
Used by `axon suggest` to propose formal annotations.
Does not gate compilation on its own.

```
@ai.intent("always returns a non-negative value")
fn abs(x : Int) -> Int: ...
```

### `@ensures`

Formal postcondition. Verified by `axon verify` at compile time.
Compilation fails if the constraint is violated.

```
@ensures("result >= 0")
fn abs(x : Int) -> Int: ...
```

### `@requires`

Formal precondition. What the caller must guarantee.

```
@requires("x >= 0")
fn sqrt(x : Float) -> Float: ...
```

### `@effect`

Side effect declaration for the function.

```
@effect("pure")
fn add(x : Int, y : Int) -> Int: ...
```

### `@program_intent`

Module-level intent declaration. Documents what the module does
as a whole — what it reads, writes, and guarantees.

```
@program_intent
"""
ONLY reads threat signals from seL4 IPC endpoints.
Does NOT modify system state.
ALWAYS logs every classification to the audit trail.
"""
module aieonyx.aegis.monitor
```

---

## Memory Model

### Memory Modes

Every value in AXON has an associated memory mode:

| Mode | Meaning |
|---|---|
| `own` | Owned value — single owner, freed on drop |
| `borrow` | Immutable reference — does not take ownership |
| `mutborrow` | Mutable reference — exclusive mutable access |
| `copy` | Value is copied on assignment (primitives) |
| `ephemeral` | Temporary — must not outlive current scope |

### Capability-Based Allocation

Functions declare what they are permitted to allocate:

```
task monitor() uses [ipc.read, collective.emit]:
    ...
```

Functions without a `uses` clause may not perform I/O or
access external resources.

### Defer and RAII

Resources are managed via `defer` for RAII-style cleanup:

```
let file = fs.open("data.bin")?
defer file.close()
# file.close() guaranteed to run at scope exit
```

---

## Imports

```
import axon.sys.sel4.ipc as ipc
import axon.mesh.collective as collective
```

---

## Error Handling

AXON uses `Result<T, E>` for fallible operations and the `?` operator
for propagation:

```
task read_data() -> Result<Bytes, Str>:
    let file = fs.open("data.bin")?   # propagates Err
    let data = file.read_all()?
    return Ok(data)
```

---

## Verification Semantics

### Constraint Domain

The AXON verifier uses abstract interpretation over the following domains:

- **Interval domain** — tracks numeric ranges `[lo, hi]`
- **Constant domain** — tracks exact constant values
- **Nullability domain** — tracks null/non-null status
- **Reachability domain** — tracks whether code paths are reachable

### Verification States

| State | Meaning |
|---|---|
| `Verified` | All constraints proven on all return paths |
| `Unknown` | Verifier cannot determine — conservative, not a failure |
| `Violated` | At least one path definitively violates a constraint |
| `NotVerifiable` | No formal constraints to check |

### Soundness Guarantee

The verifier is sound within its abstract domain:
- If it reports `Verified`, the constraint holds on all analyzed paths
- If it reports `Unknown`, it cannot prove OR disprove
- If it reports `Violated`, a definite counterexample was found
- The LLM is never in the Trusted Computing Base

---

## Standard Library

The AXON standard library (`axon_std`) provides:

| Module | Contents |
|---|---|
| `axon.sys.sel4.ipc` | seL4 IPC channel primitives |
| `axon.mesh.collective` | Collective mesh communication |
| `axon.io` | Basic I/O (guarded by capability) |
| `axon.mem` | Memory allocation primitives |
| `axon.crypto` | Cryptographic primitives |

---

## Targets

| Target string | Platform |
|---|---|
| `x86_64-unknown-linux-gnu` | x86-64 Linux (default) |
| `aarch64-unknown-linux-gnu` | ARM64 Linux |
| `aarch64-unknown-none-elf` | ARM64 seL4 bare-metal |

---

## Package Format

AXON packages use the `.aix` format with three trust modes:

| Mode | Description |
|---|---|
| `Locked` | Fully verified, signed, immutable |
| `Developer` | Source available, unverified |
| `Open` | Community package, use with caution |

---

*Version 0.3 — May 2026*
*Copyright © 2026 Edison Lepiten — AIEONYX*
