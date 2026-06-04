# AXON Phase 7G — Formal Verification Engine

**Tests:** 60  
**Crates:** `axon_ai`, `axon_codegen`  
**Status:** CLOSED

---

## Overview

Phase 7G delivers AXON's formal verification engine — a compile-time system that checks behavioral contracts (`@requires`/`@ensures`), behavioral subtyping (Liskov Substitution Principle), and receiver lifetime escape (E7E-008). This is the capability that distinguishes AXON from all existing systems-programming languages.

---

## Sub-stages

| Stage | Scope |
|-------|-------|
| 7G-1 | Contract system foundations (ContractExpr, ContractSpec, MetadataStore) |
| 7G-2 | Contract verification engine (check_requires, check_ensures, DynCall) |
| 7G-3 | Behavioral subtyping / LSP (implies, check_lsp, verify_all_lsp) |
| 7G-4 | E7E-008 receiver lifetime escape |
| 7G-5 | 7G integration + close gate (run_7g_passes) |

---

## Contract Expression AST

```rust
pub enum ContractExpr {
    True,
    False,
    Compare { lhs: Symbol, op: CompareOp, rhs: ContractValue, span: Span },
    Predicate { name: Symbol, args: Vec<Symbol>, span: Span },
    And(Box<ContractExpr>, Box<ContractExpr>),
    Or(Box<ContractExpr>, Box<ContractExpr>),
    Not(Box<ContractExpr>),
}

pub enum CompareOp { Eq, Ne, Lt, Le, Gt, Ge }

pub enum ContractValue {
    Integer(i64),
    Bool(bool),
    Null,
    Param(Symbol),   // reference to a parameter name
}
```

### ContractSpec

```rust
pub struct ContractSpec {
    pub clauses: Vec<ContractClause>,
    pub span:    Span,
}

pub enum ContractClause {
    Requires { expr: ContractExpr, span: Span },
    Ensures  { expr: ContractExpr, span: Span },
}
```

**Key invariant:** `ContractSpec` has no `Default` derive. Use `ContractSpec::empty(span)` for empty specs. This prevents accidentally creating specs without span information.

**Key invariant:** `ContractExpr::PartialEq` is for structural identity only. All logical checking uses `implies()`, never `==`.

---

## MetadataStore (axon_ai/src/store.rs)

```rust
pub struct MetadataStore {
    pub trait_contracts: HashMap<DefId, TraitContract>,
    pub impl_contracts:  HashMap<ImplId, ImplContract>,
}

pub struct TraitContract {
    pub trait_def_id: DefId,
    pub methods:      BTreeMap<MethodId, TraitMethodContract>,
}

pub struct TraitMethodContract {
    pub method_id:   MethodId,
    pub method_name: Symbol,
    pub spec:        ContractSpec,
}

pub struct ImplMethodContract {
    pub method_id:   MethodId,
    pub method_name: Symbol,
    pub spec:        ContractSpec,
    pub subtyping:   SubtypingStatus,
}
```

### MethodId::PLACEHOLDER

`MethodId(u32::MAX)` is the sentinel for unresolved method IDs. `populate_from_module()` asserts no PLACEHOLDER in production. `resolve_placeholder_method_ids()` returns a list of unresolved entries (emitted as warnings, not errors).

---

## implies() — Conservative Structural Implication

```rust
/// Depth-bounded to prevent stack overflow.
/// Returns false at depth >= MAX_CONTRACT_DEPTH (64).
pub fn implies(a: &ContractExpr, b: &ContractExpr) -> bool {
    implies_bounded(a, b, 0)
}
```

### Handled Cases

| Pattern | Rule |
|---------|------|
| `_ ⊢ True` | Always true |
| `False ⊢ _` | Always true |
| `a ⊢ a` | Structural identity |
| `And(x,y) ⊢ b` | `implies(x,b) ∨ implies(y,b)` |
| `a ⊢ And(x,y)` | `implies(a,x) ∧ implies(a,y)` |
| `Or(x,y) ⊢ b` | `implies(x,b) ∧ implies(y,b)` |
| `a ⊢ Or(x,y)` | `implies(a,x) ∨ implies(a,y)` |
| `Not(x) ⊢ Not(y)` | `implies(y,x)` (contrapositive) |
| `Compare(lhs, op_a, va) ⊢ Compare(lhs, op_b, vb)` | `compare_implies(op_a, va, op_b, vb)` |

### compare_implies() — Integer Subsumption (13 cases)

| Rule | Condition |
|------|-----------|
| `x > va ⊢ x >= vb` | `va >= vb` |
| `x > va ⊢ x > vb` | `va >= vb` |
| `x >= va ⊢ x >= vb` | `va >= vb` |
| `x >= va ⊢ x > vb` | `va > vb` (NEW in 7H-2) |
| `x < va ⊢ x <= vb` | `va <= vb` |
| `x < va ⊢ x < vb` | `va <= vb` |
| `x <= va ⊢ x <= vb` | `va <= vb` |
| `x <= va ⊢ x < vb` | `va < vb` (NEW in 7H-2) |
| `x == va ⊢ x >= vb` | `va >= vb` |
| `x == va ⊢ x <= vb` | `va <= vb` |
| `x == va ⊢ x > vb` | `va > vb` |
| `x == va ⊢ x < vb` | `va < vb` |
| `x != va ⊢ x != vb` | `va == vb` |

**MAX_CONTRACT_DEPTH = 64** (applies to `implies()`, `implies_bounded()`, and `collect_param_names()`)

---

## check_requires() + check_ensures()

Called at every call site and return point respectively:

```
check_requires(call_site_span, callee_def_id, method_name, contract, call_args, move_state, cfg, module)
check_ensures(return_span, fn_def_id, method_name, contract, return_place, move_state, cfg, module)
```

Both support: `True`, `False`, `And(a,b)` (both checked), `Compare`, `Predicate`, `Unknown`.

**Parent span threading:** `And` recursion uses the parent contract's span — no `Span::DUMMY` in production.

---

## LSP Verification

### LSP Rules

- **@requires rule:** `trait.requires ⊢ impl.requires` — impl accepts *at least* everything trait promises
- **@ensures rule:** `impl.ensures ⊢ trait.ensures` — impl guarantees *at least* everything trait promises

### check_lsp() (per-method)

Returns `Vec<LspViolation>` — both violations reported even if both rules are violated simultaneously.

```rust
pub enum LspViolation {
    RequiresStrengthened { impl_id, trait_def_id, method_name, span, trait_requires, impl_requires },
    EnsuresWeakened      { impl_id, trait_def_id, method_name, span, impl_ensures, trait_ensures },
}
```

### verify_all_lsp() (module-level)

- Validates `TraitImpl` existence before checking
- Validates `trait_def_id` in module trait registry
- Marks methods with `SubtypingStatus::NoTraitContract` if no trait contract exists
- Marks methods with `SubtypingStatus::Failed { reason }` on violation
- Collects ALL violations before returning (no early exit)

---

## E7E-008 Receiver Lifetime Escape

Fires when a method with `&self` or `&mut self` receiver returns a reference derived from `self`.

### 3-Condition Gate

1. Method has a reference receiver (`ReceiverMode::SharedRef` or `MutableRef`)
2. Return type is or contains `Type::Reference` (including via generic type params — conservative)
3. CFG data-flow trace: return value was produced by borrowing the receiver or a field of it

### is_derived_from_receiver() — Full-Scan

Scans **ALL** blocks (not first-match). Returns `true` if ANY assignment to `return_place` is receiver-derived. Bounded by `MAX_CONTRACT_DEPTH`.

### is_receiver_or_field() — PlaceId-Based

Uses `PlaceTable::root_place_of(place) == Some(receiver_place)` for field projection detection. Not Symbol-based (robust to shadowing).

### Generic Return Type Conservatism

If `return_type` contains an unconstrained type parameter from `fn_def.type_params`, `type_contains_reference()` returns `true` conservatively. E7E-008 fires. Full monomorphization-based analysis deferred to Phase 8.

---

## run_7g_passes()

```
Pass 1: MetadataStore::populate_from_module()
Pass 2: resolve_placeholder_method_ids() (warnings, not errors)
Pass 3: verify_all_contracts() (check_requires + check_ensures + E7E-008)
Pass 4: verify_all_lsp() (LSP checking — always runs even if Pass 3 found errors)
```

Returns `Ok(VerificationResult)` or `Err(VerificationErrors)`. Both passes always run.

### MoveStateMap Integration (7H-2 resolution)

`verify_all_contracts()` receives `move_results: &HashMap<DefId, MoveCheckResult>` from `OwnershipResult`. Per-block move state is fetched from `MoveCheckResult::move_state_at(block.id)`.

---

## Production Hardening Obligations (Phase 8)

- SMT solver for full logical implication (replaces conservative `implies()`)
- Full monomorphization for generic return type analysis
- GEP-based vtable dispatch in LLVM

---

## Exit Conditions Verified (60 tests, 6 categories)

- Contract foundation: ContractSpec, conjunction, PLACEHOLDER cleanup
- Contract verification: check_requires, check_ensures, DynCall, unknown params
- Behavioral subtyping: implies all cases, compare_implies all 13 cases, LSP rules
- Receiver lifetime: E7E-008 all scenarios including multi-branch and generics
- Dynamic dispatch contracts: DynCall @requires checking
- Integration: run_7g_passes() clean/violation/both-passes

---

*Phase 7G — CLOSED*
