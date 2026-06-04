# AXON Phase 7C — Generic Type Parameters + Monomorphization

**Tests:** 56  
**Crates:** `axon_hir_common`, `axon_codegen`  
**Status:** CLOSED

---

## Overview

Phase 7C adds generic type parameters to AXON, enabling parametric polymorphism. The compiler uses monomorphization (like Rust/C++) rather than type erasure (like Java/Go) — each concrete instantiation of a generic function produces a separate specialized copy.

---

## Core Types

### TypeParam

```rust
pub struct TypeParam {
    pub name:   Symbol,           // "T", "K", "V", etc.
    pub bounds: Vec<TraitBound>,  // T: Display + Send
    pub span:   Span,
}

pub struct TraitBound {
    pub trait_name: Symbol,
    pub span:       Span,
}
```

### GenericFnDef

```rust
pub struct GenericFnDef {
    pub base: FnDef,
    pub type_params: Vec<TypeParam>,
}
```

### MonoTable

The monomorphization table maps (DefId, substitution) pairs to concrete monomorphized FnDefs:

```rust
pub struct MonoTable {
    entries: HashMap<MonoKey, MonoDef>,
}

pub struct MonoKey {
    pub generic_def_id:   DefId,
    pub substitutions:    Vec<(Symbol, Type)>,  // "T" → i32, etc.
}

pub struct MonoDef {
    pub specialized_def_id: DefId,
    pub fn_def:             FnDef,
}
```

---

## Monomorphization Algorithm

For each call site where a generic function is invoked with concrete type arguments:

1. **Build substitution map:** Map each TypeParam name to the concrete Type at the call site
2. **Look up MonoTable:** If this (generic_def_id, substitutions) pair already exists, reuse it
3. **Specialize:** Apply substitutions to produce a concrete FnDef with all TypeParams replaced
4. **Register:** Add the new MonoDef to the MonoTable with a fresh DefId
5. **Recurse:** If the specialized body itself contains generic calls, monomorphize those too (cycle detection required)

### Cycle Detection

```rust
pub struct MonoStack {
    in_progress: HashSet<MonoKey>,
}
```

If a MonoKey is already in `in_progress` during monomorphization, it indicates a recursive generic instantiation. The compiler emits a `MonomorphizationCycleError` rather than infinitely recursing.

---

## Type Bounds Checking

Before monomorphization, the compiler verifies that the concrete type satisfies all bounds on the TypeParam:

```
fn max<T: Ord>(a: T, b: T) -> T

Call site: max(5_i32, 3_i32)
  → Check: i32: Ord ✓ (i32 implements Ord via trait registry)
  → Proceed with monomorphization
```

If the bound is not satisfied, the compiler emits `BoundNotSatisfied { type_name, trait_name, span }`.

---

## Integration Points

- **7D (Traits):** TypeParam bounds reference TraitDefs; bound checking uses the trait registry
- **7E (Ownership):** Specialized FnDefs go through the same ownership analysis as non-generic functions
- **7F (Dispatch):** Generic types can include `dyn Trait` — monomorphization handles the fat pointer correctly
- **7G (Verification):** Contract expressions in generic functions are specialized alongside the types

---

## Known Limitations (Phase 8)

- No higher-kinded types (HKT)
- No associated types on traits (planned for 8A)
- No const generics
- Monomorphization explosion for large generic call graphs (no optimization)

---

## Exit Conditions Verified (56 tests)

- TypeParam and TraitBound defined correctly
- MonoTable lookup/insert idempotent
- Cycle detection prevents infinite monomorphization
- Bound checking catches unsatisfied bounds
- Substitution applies recursively through nested types
- Generic functions integrated into the full pipeline

---

*Phase 7C — CLOSED*
