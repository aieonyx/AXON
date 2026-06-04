# AXON Phase 7E — Ownership Model (Move / Borrow / Drop / Copy)

**Tests:** 185  
**Crates:** `axon_hir_common`, `axon_codegen`  
**Status:** CLOSED

---

## Overview

Phase 7E implements AXON's ownership and borrowing system — the core memory safety mechanism. Inspired by Rust's NLL (Non-Lexical Lifetimes) model, AXON's ownership analysis operates on the elaborated CFG (Control Flow Graph) after HIR lowering.

---

## Core HIR Types (`axon_hir_common/src/ownership.rs`)

### Place and PlaceId

A `Place` represents a memory location (local variable, field, or dereferenced pointer):

```rust
pub struct Place {
    pub root:          Symbol,    // variable name
    pub root_place_id: PlaceId,   // PlaceId of the root variable (for field projection lookup)
    pub projection:    Vec<PlaceElem>,
}

pub enum PlaceElem {
    Field(Symbol),   // .field_name
    Deref,           // *ptr
    Index(PlaceId),  // [idx]
}

pub struct PlaceId(u32);
```

**Naming convention:** `place_alloca(p) = %p{index}` in LLVM IR.

### BorrowId and RegionId

```rust
pub struct BorrowId(u32);
pub struct RegionId(u32);

pub struct Borrow {
    pub borrow_id:  BorrowId,
    pub lender:     PlaceId,
    pub result:     PlaceId,
    pub mutability: Mutability,
    pub region:     RegionId,
    pub span:       Span,
}
```

### PlaceState (for move checking)

```rust
pub enum PlaceState {
    Initialized,                  // value is valid and owned
    Moved { at: Span },           // value has been moved out
    PartiallyMoved { fields: Vec<Symbol> },  // some fields moved
    Uninitialized,                // declared but not yet assigned
    BorrowedShared { borrows: Vec<BorrowId> },
    BorrowedMut { borrow: BorrowId },
}
```

### BorrowViolation

```rust
pub struct BorrowViolation {
    pub kind:  BorrowViolationKind,
    pub place: PlaceId,
    pub span:  Span,
}

pub enum BorrowViolationKind {
    ConflictBorrow { existing: BorrowId },
    DroppedWhileBorrowed { active_borrow: BorrowId },
    LenderMovedWhileBorrowed { active_borrow: BorrowId },
    BorrowEscapesReceiver { method_name: Symbol, receiver_mode: ReceiverMode },
    TwoPhaseRequired,
}
```

---

## CFG Structure

### BasicBlock

```rust
pub struct BasicBlock {
    pub id:          BasicBlockId,
    pub statements:  Vec<CfgStatement>,
    pub terminator:  Terminator,
}
```

### CfgStatement Variants

| Variant | Purpose |
|---------|---------|
| `StorageLive { place }` | Declare a local variable; allocate stack slot |
| `StorageDead { place }` | End of variable scope; hint to optimizer |
| `Assign { lhs, rhs }` | Move or copy a value |
| `Borrow { result, lender, mutability }` | Create a borrow of `lender` into `result` |
| `BorrowExpires { borrow_id }` | Lifetime hint: this borrow is no longer active |
| `Use { place }` | Read from a place (for move checking) |
| `Drop { place }` | Explicit user-invoked drop |
| `DropElaborated { place }` | Compiler-inserted drop (only when `place_needs_drop()` is true) |
| `ConditionalDrop { place, flag }` | Drop only if flag is true (for partially-initialized paths) |
| `SetDropFlag { flag, value }` | Set the conditional drop flag |
| `DynCoerce { src, dst }` | Coerce a concrete type to dyn Trait |
| `DynCall { site }` | Call through a vtable |

### Terminator Variants

| Variant | Purpose |
|---------|---------|
| `Return { value: Option<PlaceId> }` | Return from function |
| `Goto { target: BasicBlockId }` | Unconditional jump |
| `SwitchBool { discriminant, true_target, false_target }` | Conditional branch |
| `Unreachable` | Marks unreachable code |

---

## Pass Structure

### Run Order (run_7e_passes)

1. **Move checker** — Forward dataflow; detects use-after-move and move-while-borrowed
2. **Borrow checker** — Detects aliasing violations, dangling borrow attempts
3. **Drop elaboration** — Inserts `DropElaborated` statements for all types needing drop
4. **Copy integration** — Exempts `T: Copy` types from move rules

### Move Check State Machine

```
Uninitialized → Initialized (after first assignment)
Initialized → Moved { at } (after move)
Initialized → BorrowedShared / BorrowedMut (after borrow)
BorrowedShared → Initialized (after borrow expires)
BorrowedMut → Initialized (after mutable borrow expires)
Moved → ERROR on next use
```

### Drop Elaboration

`type_needs_drop()` returns true if:
1. Type implements the `Drop` trait explicitly, OR
2. Any field of the type transitively needs dropping (with cycle protection)

Primitives (`i32`, `bool`, etc.) and `T: Copy` types never need dropping.

---

## E7E Error Codes

| Code | Description |
|------|-------------|
| E7E-001 | Use of moved value |
| E7E-002 | Move while borrowed |
| E7E-003 | Borrow of moved value |
| E7E-004 | Mutable borrow while shared borrow active |
| E7E-005 | Shared borrow while mutable borrow active |
| E7E-006 | Lender moved while borrow active |
| E7E-007 | Double move |
| E7E-008 | Receiver lifetime escape (**see 7G**) |
| E7E-009 | Drop of borrowed value |
| E7E-010 | Partial move — field already moved |
| E7E-011 | Use of uninitialized value |

---

## Key Architectural Decisions

### PlaceConflict::MaybeAlias

For dereference projections (`*ptr`), AXON conservatively treats two derefs as potentially aliasing unless proven otherwise. This prevents false acceptance of unsafe aliasing patterns at the cost of some false positives.

### BorrowExpires Placement

`BorrowExpires` is inserted at the `StorageDead` of the borrow result in `lower_body()`. This ensures the borrow region ends deterministically at the end of the borrow result's lifetime, not at the end of the function.

### predecessor_cache

`FunctionCfg` maintains a `predecessor_cache: HashMap<BasicBlockId, Vec<BasicBlockId>>` for O(1) predecessor lookup during borrow checking. Built lazily on first access.

---

## OwnershipResult (output of run_7e_passes)

```rust
pub struct OwnershipResult {
    pub cfgs:         HashMap<DefId, FunctionCfg>,
    pub move_results: HashMap<DefId, MoveCheckResult>,
    pub borrow_errors: Vec<BorrowViolation>,
    pub drop_elaborations: HashMap<DefId, Vec<PlaceId>>,
}
```

---

## Exit Conditions Verified (185 tests)

- Move checker: all 11 E7E error codes detected
- Borrow checker: aliasing violations, dangling borrow, lender-moved
- Drop elaboration: transitive drop, cycle protection, primitive exclusion
- Copy integration: Copy types exempt from move errors
- BorrowExpires placed at StorageDead of borrow result
- Full integration: run_7e_passes() returns OwnershipResult

---

*Phase 7E — CLOSED*
