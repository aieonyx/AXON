# AXON Memory Model v0.2.1

> Memory ownership, lifetime, and allocation semantics for AXON.
> Copyright © 2026 Edison Lepiten — AIEONYX

---

## Overview

AXON uses a capability-based memory model inspired by Rust's ownership
system, simplified for systems programming in sovereign environments.
The core principle: **every value has a known owner, a known lifetime,
and a declared allocation capability.**

---

## Memory Modes

Every binding in AXON has an associated memory mode. The mode determines
how the value can be used, passed, and freed.

### `own` — Ownership

The binding owns the value. When the binding goes out of scope,
the value is freed. Ownership can be transferred (moved) but not copied.

```
let data = Buffer.alloc(1024)   # owns the buffer
process(data)                    # moves ownership to process()
# data is no longer accessible here
```

### `borrow` — Immutable Reference

A temporary read-only view of a value. Does not take ownership.
Multiple borrows can coexist. The value must outlive the borrow.

```
fn inspect(data : &Buffer) -> Int:
    return data.length()   # borrows, does not move
```

### `mutborrow` — Mutable Reference

A temporary exclusive mutable view. No other borrows can exist
while a mutborrow is active.

```
fn zero_fill(data : &mut Buffer):
    data.fill(0)
```

### `copy` — Copy Semantics

Primitive types (`Int`, `Bool`, `Float`, `Char`) use copy semantics.
Assignment creates a new independent copy.

```
let x = 42
let y = x    # y is a copy of x, both are valid
```

### `ephemeral` — Scope-Bound

An ephemeral binding must not outlive the current scope.
Used for resources that must be explicitly released.

```
let@ channel = ipc.open_channel()?   # ephemeral
defer channel.close()
# channel cannot escape this scope
```

---

## Allocation Capabilities

Functions in AXON must declare what resources they are permitted
to access via the `uses` clause on task declarations.

### Declaring Capabilities

```
task monitor() uses [ipc.read, collective.emit]:
    let channel = ipc.open_channel()?
    for signal in channel.signals:
        collective.emit(classify(signal))
```

### Pure Functions

Functions declared without a `uses` clause are implicitly pure —
they may not perform I/O, allocate heap memory beyond their parameters,
or access external state.

```
fn classify(severity : Int) -> Int:
    match severity:
        0 => return 0
        1 => return 1
        _ => return 2
```

### Capability Inheritance

A task may only grant capabilities it itself possesses.
A task with `uses [ipc.read]` cannot call a function that requires
`uses [ipc.write]`.

---

## Lifetime Rules

1. **A value must outlive all its borrows.**
2. **An ephemeral binding (`let@`) must not escape its scope.**
3. **A mutable borrow is exclusive — no other borrows during its lifetime.**
4. **Ownership transfer (move) invalidates the original binding.**

---

## Defer and RAII

AXON uses `defer` for deterministic resource cleanup, similar to
Go's defer or C++ RAII destructors.

```
task read_file(path : Str) -> Result<Bytes, Str>:
    let file = fs.open(path)?
    defer file.close()             # guaranteed cleanup
    let data = file.read_all()?
    return Ok(data)
    # file.close() runs here, even if an error occurred
```

### Defer Ordering

Multiple defers execute in LIFO (last-in, first-out) order:

```
let a = resource_a.open()
defer a.close()          # runs third
let b = resource_b.open()
defer b.close()          # runs second
let c = resource_c.open()
defer c.close()          # runs first
```

---

## Stack vs Heap

### Stack Allocation

All primitive types and fixed-size structs are stack allocated by default.
Stack allocation is implicit and requires no capability declaration.

```
let x : Int = 42           # stack
let p : Point = Point(1, 2) # stack if Point is fixed-size
```

### Heap Allocation

Heap allocation requires an explicit allocator call and a capability
declaration on the enclosing task.

```
task process() uses [mem.alloc]:
    let buf = Buffer.alloc(4096)   # heap
    defer buf.free()
```

Functions without `mem.alloc` capability cannot heap-allocate.
This makes heap allocation auditable at the function signature level.

---

## seL4 Integration

On seL4, AXON's memory model maps directly to seL4 capabilities:

| AXON concept | seL4 concept |
|---|---|
| `own` binding | Capability with full rights |
| `borrow` | Capability with read rights |
| `mutborrow` | Capability with write rights |
| `uses [ipc.read]` | IPC endpoint capability |
| `defer free()` | Capability revocation |

This mapping is intentional. AXON programs running on seL4 inherit
the microkernel's formal isolation guarantees at the language level.

---

## Formal Effect Annotations

The `@effect` decorator documents memory behavior for the formal verifier:

```
@effect("no_allocate")
fn classify(severity : Int) -> Int:
    ...
```

| Annotation | Meaning |
|---|---|
| `@effect("pure")` | No side effects, no allocation |
| `@effect("readonly")` | Reads but does not write |
| `@effect("no_allocate")` | No heap allocation |
| `@effect("may_allocate")` | May heap-allocate |
| `@effect("writes_audit_log")` | Writes to audit trail |

Effect checking is enforced by the AXON verifier (P5-05).
In the current release, effects are declared and advisory.

---

## Memory Safety Guarantees

AXON's memory model provides the following guarantees when used correctly:

- **No use-after-free** — ownership rules prevent access after move
- **No double-free** — single owner, freed once at scope exit via defer
- **No dangling references** — lifetime rules prevent borrows outliving owners
- **No data races** — exclusive mutborrow prevents concurrent mutation
- **Deterministic cleanup** — defer guarantees resource release order

These guarantees hold for code that follows AXON's mode annotations.
Unsafe operations (for seL4 capability manipulation) bypass these
guarantees and must be explicitly marked.

---

*Version 0.2.1 — May 2026*
*Copyright © 2026 Edison Lepiten — AIEONYX*
