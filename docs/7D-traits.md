# AXON Phase 7D — Trait System + Coherence + Static Dispatch

**Tests:** 189  
**Crates:** `axon_hir_common`, `axon_ai`, `axon_codegen`  
**Status:** CLOSED

---

## Overview

Phase 7D implements AXON's trait system — the mechanism for defining shared behavior across types. AXON traits are similar to Rust traits: they define a set of method signatures, and types implement them to gain the associated behavior. Phase 7D covers:

1. Trait definition and implementation HIR types
2. Coherence rules (no conflicting implementations)
3. Method resolution (which impl to use at a call site)
4. Static dispatch (zero-cost, monomorphized at compile time)
5. Object safety rules (required for dyn Trait in 7F)

---

## Core Types

### TraitDef

```rust
pub struct TraitDef {
    pub def_id:    DefId,
    pub name:      Symbol,
    pub methods:   Vec<TraitMethodDef>,
    pub type_params: Vec<TypeParam>,   // trait Foo<T>
    pub span:      Span,
}

pub struct TraitMethodDef {
    pub method_id:      MethodId,
    pub name:           Symbol,
    pub params:         Vec<ParamDef>,
    pub return_type:    Option<Type>,
    pub receiver_mode:  Option<ReceiverMode>,
    pub span:           Span,
}
```

### TraitImpl

```rust
pub struct TraitImpl {
    pub impl_id:      ImplId,
    pub trait_def_id: DefId,
    pub self_type:    Type,         // impl Foo for MyStruct
    pub methods:      Vec<FnDef>,
    pub span:         Span,
}
```

### ReceiverMode

```rust
pub enum ReceiverMode {
    Value,       // fn foo(self)
    SharedRef,   // fn foo(&self)
    MutableRef,  // fn foo(&mut self)
}
```

---

## Coherence Rules

AXON enforces the orphan rule: either the trait or the implementing type must be defined in the current crate.

The coherence checker verifies:

1. **No duplicate implementations:** Two impls of the same trait for the same type → `CoherenceError::DuplicateImpl`
2. **Orphan rule:** `impl ForeignTrait for ForeignType` → `CoherenceError::OrphanViolation`
3. **All methods implemented:** If a trait requires method `foo()` and the impl doesn't define it → `CoherenceError::MissingMethod`
4. **No extra methods:** An impl may not define methods not in the trait → `CoherenceError::UnknownMethod`

---

## Method Resolution

At a static call site `value.method()`:

1. Determine the type of `value`
2. Look up all TraitImpls for that type in the module
3. Find the TraitImpl that defines `method`
4. If exactly one found → resolved; emit a direct call to that impl's FnDef
5. If zero found → `MethodNotFound`
6. If more than one found → `AmbiguousMethod` (coherence should have caught this)

---

## Static Dispatch

Static dispatch generates a direct function call — no runtime overhead. The compiler resolves the method at compile time and emits:

```llvm
call ret_ty @TraitName_ImplType_method(args...)
```

### Build Method Symbol

The unified `build_method_symbol()` function (in `axon_codegen/src/symbols.rs`) generates the LLVM symbol name:

```rust
fn build_method_symbol(trait_name: Symbol, self_type: Symbol, method_name: Symbol) -> Symbol {
    Symbol::intern(&format!("{}_{}_{}", trait_name.as_str(), self_type.as_str(), method_name.as_str()))
}
```

---

## Object Safety Rules

A trait is **object-safe** (can be used as `dyn Trait`) if all methods satisfy:

1. Receiver is `&self` or `&mut self` (not `self` by value)
2. No type parameters on the method itself (`fn foo<T>()` is not object-safe)
3. No `where Self: Sized` bounds
4. Return type is not `Self`

Traits that violate these rules can still be used for static dispatch but cannot be used with `dyn Trait`.

---

## SubtypingStatus

Every impl method carries a `SubtypingStatus` updated by the 7G LSP checker:

```rust
pub enum SubtypingStatus {
    Unverified,                         // not yet checked
    Verified,                           // LSP satisfied
    Failed { reason: Symbol },          // LSP violated
    NoTraitContract,                    // trait has no @requires/@ensures
}
```

---

## Integration Points

- **7C (Generics):** Trait bounds on TypeParams use TraitDef lookup
- **7F (Dispatch):** Object-safe traits are used for dyn Trait vtable construction
- **7G (Verification):** TraitContract and ImplContract stored in MetadataStore for LSP checking

---

## Exit Conditions Verified (189 tests)

- TraitDef and TraitImpl HIR types correct
- Coherence: duplicate impl caught, orphan rule enforced
- Method resolution: exact match, missing method, ambiguous method
- Static dispatch: correct LLVM symbol generated
- Object safety: all 4 rules enforced
- SubtypingStatus transitions: Unverified → Verified/Failed
- Integration with 7C generics

---

*Phase 7D — CLOSED*
