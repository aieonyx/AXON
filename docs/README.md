# AXON Compiler — Phase 7 Specification Archive

**Project:** AIEONYX / AXON Compiler  
**Author:** Edison Lepiten (Lead Architect via Claude)  
**Status:** FORMALLY CLOSED  
**Date Closed:** 2026-06-02  
**Classification:** Internal Engineering — Confidential

---

## What Is This?

This directory is the permanent specification archive for **Phase 7** of the AXON compiler. Phase 7 was conducted as a design/specification exercise in a dedicated Claude session, producing detailed architecture documents, Rust pseudocode, DeepSeek audit trails, and comprehensive exit condition verification.

Phase 7 code exists as **specification only** — it was not committed to the repository as working source. Phase 8 will implement this architecture as real, compilable, Aider-committed Rust code.

---

## Phase 7 Scope

Phase 7 extended AXON from a Phase 6 compiler (bio-inspired stdlib features, 263 tests) to a full formally-verified, ownership-safe, contract-enforcing compiler specification.

### Sub-stages

| Stage | Scope | Tests | Status |
|-------|-------|-------|--------|
| 7A | CCP Capability Profiles | — (spec) | ✅ LOCKED |
| 7B | ASP Patchable System | — (spec) | ✅ LOCKED |
| 7C | Generic Type Params + Monomorphization | 56 | ✅ CLOSED |
| 7D | Trait System + Coherence + Static Dispatch | 189 | ✅ CLOSED |
| 7E | Ownership (Move / Borrow / Drop / Copy) | 185 | ✅ CLOSED |
| 7F | Dynamic Dispatch (vtables + dyn Trait) | 80 | ✅ CLOSED |
| 7G | Formal Verification (Contracts + LSP + E7E-008) | 60 | ✅ CLOSED |
| 7H | LLVM IR Emission + Close Gate | 82 | ✅ CLOSED |

**Total workspace tests at close: 618**  
**DeepSeek QA audits completed: 14**  
**Tasks completed: 27**

---

## Workspace Crates

```
axon/
  axon_hir_common/   ← Shared HIR types (Place, BorrowId, ContractExpr, etc.)
  axon_parser/       ← AXON grammar + ContractParser (@requires/@ensures)
  axon_ai/           ← MetadataStore: TraitContract, ImplContract, SubtypingStatus
  axon_codegen/      ← Full compiler pipeline: ownership, dispatch, verification, LLVM
  axon_rt/           ← Runtime support
  axon_cli/          ← axon CLI frontend
```

---

## Key Architectural Decisions

### Memory Safety
AXON uses a Rust-like NLL (Non-Lexical Lifetime) ownership model enforced at the HIR/CFG level. Ownership violations are caught by the move checker and borrow checker before any LLVM emission occurs.

### Formal Contracts
AXON has first-class `@requires` (preconditions) and `@ensures` (postconditions) as part of the language syntax. These are parsed by `ContractParser`, stored in `MetadataStore`, and verified at every call site and return point by `verify_all_contracts()`.

### Behavioral Subtyping
Every trait impl is verified to satisfy the Liskov Substitution Principle via `check_lsp()` and `implies()`. A conservative structural implication checker (depth-bounded at 64) with integer comparison subsumption verifies that impl @requires are weaker and @ensures are stronger than their trait counterparts.

### Capability Profiles (CCP)
Four profiles are defined: `seL4-strict`, `sovereign-offline`, `mesh-node`, `dev-mode`. BASTION rejects `dev-mode` by default. Profile enforcement gates which system capabilities are available at compile time.

### LLVM IR Emission
AXON targets LLVM 15+ opaque pointer model. Local variables are stack-allocated via `alloca` (named `%pN`). SSA temporaries use a per-block monotonic counter (`%tN`). Drop glue functions are called only for types where `place_needs_drop()` is true.

---

## Security Guarantees Delivered

1. **No use-after-move** — enforced by move checker (7E)
2. **No dangling borrows** — enforced by borrow checker (7E)
3. **No double-free** — enforced by drop elaboration (7E)
4. **Behavioral contract compliance** — enforced by verification engine (7G)
5. **LSP correctness for all trait impls** — enforced by LSP checker (7G)
6. **No receiver lifetime escape** — enforced by E7E-008 checker (7G)
7. **Dynamic dispatch soundness** — enforced by vtable verifier (7F)
8. **Capability-gated compilation** — enforced by CCP profiles (7A)

---

## Files in This Archive

| File | Contents |
|------|----------|
| `README.md` | This file — master overview |
| `7A-7B-profiles.md` | CCP profiles + ASP patchable system |
| `7C-generics.md` | Generic type params + monomorphization |
| `7D-traits.md` | Trait system + coherence + dispatch |
| `7E-ownership.md` | Ownership model (move/borrow/drop/Copy) |
| `7F-dispatch.md` | Dynamic dispatch (vtables + dyn Trait) |
| `7G-verification.md` | Formal verification engine |
| `7H-llvm-emission.md` | LLVM IR type + instruction mapping + emission |
| `AUDIT-TRAIL.md` | All 14 DeepSeek audit cycles |
| `PHASE8-PROMPT.md` | Complete Phase 8 session opening prompt |

---

## What Phase 7 Does NOT Include

- A parser for full AXON source programs (only contract syntax)
- Type inference (declared types only)
- A working binary — no `.ll`, `.o`, or executable output
- A standard library (`axon_std`)
- Lifetime annotations (structural analysis only)

These are Phase 8 deliverables.

---

## Phase 8 Preview

Phase 8 will convert this specification into a working AXON compiler:

| Stage | Scope |
|-------|-------|
| 8A | Full AXON source parser (.axon → HIR) |
| 8B | Type inference (bidirectional, Hindley-Milner style) |
| 8C | LLVM integration (llc + clang → object code) |
| 8D | Standard library core (axon_std: Vec, String, Option, Result) |
| 8E | Profile enforcement CLI (--profile flag, BASTION integration) |
| 8F | Phase 8 close gate → Profile Stage opens |

---

*AIEONYX — "To build a new Civilization where every Human is Supreme and Sovereign over their own Digital Existence."*
