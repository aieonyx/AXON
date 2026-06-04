# AXON Phase 7F — Dynamic Dispatch (vtables + dyn Trait)

**Tests:** 80  
**Crates:** `axon_hir_common`, `axon_codegen`  
**Status:** CLOSED

---

## Overview

Phase 7F implements dynamic dispatch — the ability to call trait methods through a pointer to a runtime-dispatched vtable. This enables runtime polymorphism via `dyn Trait`. Phase 7F also handles loop CFG construction (for back-edge detection) and the `MaybeAlias` borrow conflict case.

---

## Core Types (`axon_hir_common/src/dispatch.rs`)

### VtableId

```rust
pub struct VtableId(u32);

impl VtableId {
    /// Sentinel for an unset/unresolved vtable ID.
    pub const UNSET: VtableId = VtableId(u32::MAX);
}
```

### VtableEntry

```rust
pub struct VtableEntry {
    pub method_name:    Symbol,
    pub impl_def_id:    DefId,      // concrete method implementation
    pub method_index:   usize,      // position in vtable array
}
```

### Vtable

```rust
pub struct Vtable {
    pub vtable_id:    VtableId,
    pub trait_def_id: DefId,
    pub self_type:    Type,
    pub entries:      Vec<VtableEntry>,
}
```

### DynTraitType and DynObject

```rust
pub struct DynTraitType {
    pub trait_def_id: DefId,
    pub bounds:       Vec<TraitBound>,
}

pub struct DynObject {
    pub data:    PlaceId,    // pointer to concrete data
    pub vtable:  PlaceId,    // pointer to vtable
}
```

### Type::Dyn

```rust
// In axon_hir_common Type enum:
Type::Dyn(DynTraitType)   // dyn Trait — maps to LlvmType::Ptr in LLVM 15+
```

---

## Vtable Construction

`VtableRegistry` is a lazy cached registry of all vtables built during compilation.

### build_vtable() — 4-step process

1. **Resolve trait methods:** Look up the TraitDef to get all method signatures
2. **Find impl:** Look up the TraitImpl for the concrete `self_type`
3. **Build entries:** For each trait method, find the corresponding impl FnDef and create a VtableEntry with the correct `method_index`
4. **Register:** Assign a unique VtableId (UNSET sentinel prevents double-registration)

### VtableId::UNSET Invariant

When a vtable is first created, its ID is `UNSET = VtableId(u32::MAX)`. The `set_id()` method assigns a real ID exactly once (guarded by an assertion). This prevents double-registration bugs.

---

## DynCall Site

```rust
pub struct DynDispatchSite {
    pub vtable_id:   VtableId,
    pub method_name: Symbol,
    pub args:        Vec<PlaceId>,
    pub result:      Option<PlaceId>,
    pub span:        Span,
}
```

`CfgStatement::DynCall { site: DynDispatchSite }` represents a dynamic method call. At LLVM emission:

```
// Production (deferred to Phase 8):
%vtable_ptr = load ptr, ptr %dyn_obj.vtable_slot
%fn_ptr = getelementptr inbounds %VtableTy, ptr %vtable_ptr, i32 0, i32 <method_idx>
%fn = load ptr, ptr %fn_ptr
call ret_ty %fn(args...)

// Phase 7 spec simplification:
; DynCall: method_name
call ret_ty %method_name(args...)   ; resolved at link time via vtable registry
```

---

## Loop CFG

Loop CFGs require special handling for the borrow checker — a `continue` in a while loop must target the loop **header** block, not the loop **body** block.

### push_loop() API

```rust
impl LoopContext {
    pub fn push_loop(&mut self, header_bb: BasicBlockId, exit_bb: BasicBlockId)
    pub fn pop_loop(&mut self)
    pub fn current_header(&self) -> Option<BasicBlockId>
    pub fn current_exit(&self) -> Option<BasicBlockId>
}
```

**Critical fix (C-001, 7F-4):** Early implementation had `continue` targeting `body_bb` instead of `header_bb`. Fixed by explicitly passing `(header, exit)` to `push_loop()`.

---

## MaybeAlias

For dereference projections, the borrow checker cannot always determine whether two places alias. `PlaceConflict::MaybeAlias` is emitted conservatively when:

1. Both places involve a `Deref` projection
2. The base pointers cannot be proven distinct

This is sound (never allows unsafe aliasing) but conservative (may reject safe code). Full alias analysis is deferred to Phase 8.

---

## DynDispatchResult (output of run_7f_passes)

```rust
pub struct DynDispatchResult {
    pub vtable_registry:    VtableRegistry,
    pub dyn_coerce_count:   usize,
    pub dyn_call_count:     usize,
    pub resolved_vtable_ids: HashMap<PlaceId, VtableId>,
}
```

### resolve_vtable_ids() Pass

After vtable construction, a final pass synchronizes vtable IDs between the HIR lowering phase and the registry. This ensures that `DynCall.vtable_id` matches `VtableRegistry.get(id)` for all dispatch sites.

---

## Object Safety Enforcement

Traits used as `dyn Trait` must be object-safe (see 7D). The `build_vtable()` function rejects non-object-safe traits with `ObjectSafetyViolation`. This prevents `Box<dyn Trait>` where `Trait` has non-object-safe methods.

---

## predecessor_cache

Added to `FunctionCfg` during 7F-4:

```rust
pub struct FunctionCfg {
    pub blocks:            Vec<BasicBlock>,
    pub param_place_ids:   Vec<PlaceId>,
    pub place_table:       PlaceTable,
    pub place_types:       HashMap<PlaceId, Symbol>,
    pub predecessor_cache: HashMap<BasicBlockId, Vec<BasicBlockId>>,  // O(1) lookup
}
```

Built lazily on first access by scanning all block terminators.

---

## Exit Conditions Verified (80 tests)

- VtableId::UNSET sentinel prevents double-registration
- vtable construction: 4-step process, method index correct
- DynCoerce: lender ID tracked in CfgStatement
- DynCall: correct IndirectCall emission
- Loop CFG: continue targets header_bb (C-001 fix confirmed)
- MaybeAlias: deref projections flagged conservatively
- resolve_vtable_ids() pass: sync confirmed
- predecessor_cache: O(1) lookup verified

---

*Phase 7F — CLOSED*
